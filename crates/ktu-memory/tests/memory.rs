//! Integração de `ktu-memory`: criação, recall (ponteiros), dedup idempotente,
//! persistência entre instâncias e projecções de leitura.

use ktu_core::memory::{Memory, MemoryDraft, MemoryKind, MemoryWrite};
use ktu_core::memory_config::{EmbeddingProvider, Embeddings};
use ktu_core::ports::{
    EmbedError, Embedder, EmbeddingMeta, FixedClock, MemFs, Timestamp, UnavailableEmbedder,
};
use ktu_memory::drain::digest;
use ktu_memory::{JsonlMemory, Rebuild, export_markdown, health};

const ROOT: &str = "/proj";
const MEM: &str = "/proj/.agents/memory";

fn memory<'a>(fs: &'a MemFs, clock: &'a FixedClock) -> JsonlMemory<'a> {
    JsonlMemory::new(fs, clock, ROOT, MEM)
}

fn draft(kind: MemoryKind, statement: &str) -> MemoryDraft {
    MemoryDraft::new(kind, statement)
}

#[test]
fn write_creates_then_recall_finds_a_pointer() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1_700_000_000_000));
    let memory = memory(&fs, &clock);
    let outcome = memory.write(&draft(
        MemoryKind::Fact,
        "o guard confina paths ao root resolvido",
    ));
    assert!(matches!(outcome, Ok(MemoryWrite::Created { .. })));

    let hints = memory.recall("guard confina paths", 5).unwrap_or_default();
    assert_eq!(hints.len(), 1);
    let Some(hint) = hints.first() else { return };
    assert!(hint.statement.contains("guard"));
    assert_ne!(hint.id, "");
    assert!(hint.score > 0.0);
}

#[test]
fn writing_the_same_statement_is_idempotent() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1));
    let memory = memory(&fs, &clock);
    let first = memory.write(&draft(MemoryKind::Fact, "a memoria vive em jsonl"));
    assert!(matches!(first, Ok(MemoryWrite::Created { .. })));
    let second = memory.write(&draft(MemoryKind::Fact, "a memoria vive em jsonl"));
    assert!(matches!(second, Ok(MemoryWrite::Unchanged { .. })));
}

#[test]
fn near_duplicates_are_not_created_twice() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1));
    let memory = memory(&fs, &clock);
    let _first = memory.write(&draft(
        MemoryKind::Fact,
        "o provider fala SSE com o llama server",
    ));
    let second = memory.write(&draft(
        MemoryKind::Fact,
        "o provider fala SSE com o llama server local",
    ));
    let not_created = matches!(
        second,
        Ok(MemoryWrite::Merged { .. } | MemoryWrite::Rejected { .. })
    );
    assert!(not_created, "esperava merge/rejeicao, veio {second:?}");
    let notes = memory.load_notes().unwrap_or_default();
    assert_eq!(notes.len(), 1);
}

#[test]
fn notes_persist_across_adapters() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1));
    {
        let memory = memory(&fs, &clock);
        let _written = memory.write(&draft(MemoryKind::Decision, "o JSONL e a fonte da verdade"));
    }
    // Nova instância sobre o mesmo `Fs` (equivalente a nova sessão/processo).
    let reopened = memory(&fs, &clock);
    let hints = reopened.recall("fonte da verdade", 5).unwrap_or_default();
    assert_eq!(hints.len(), 1);
}

#[test]
fn export_and_health_describe_the_corpus() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1));
    let memory = memory(&fs, &clock);
    let mut with_anchor = draft(MemoryKind::Risk, "ha um risco de latencia em CPU");
    with_anchor.anchors = vec!["does/not/exist.rs".to_owned()];
    let _written = memory.write(&with_anchor);

    let notes = memory.load_notes().unwrap_or_default();
    let markdown = export_markdown(&notes);
    assert!(markdown.contains("latencia em CPU"));

    let health = health(&memory).unwrap_or_default();
    assert_eq!(health.notes, 1);
    assert_eq!(health.malformed, 0);
    assert_eq!(health.broken_anchors.len(), 1);
    assert!(!health.is_healthy());
}

// --- H8: canal semântico (opcional) -------------------------------------------------------

/// Embedder de teste: mapeia palavras-chave para vetores ortogonais fixos.
///
/// Prova que o canal **vetorial** entra na fusão RRF: a consulta não partilha nenhum
/// termo com a nota, pelo que só o canal semântico a pode recuperar.
struct KeywordEmbedder {
    meta: EmbeddingMeta,
}

impl KeywordEmbedder {
    fn new(dimensions: usize) -> Self {
        Self {
            meta: EmbeddingMeta::new("test", "keyword", "1", dimensions),
        }
    }
}

impl Embedder for KeywordEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        Ok(texts
            .iter()
            .map(|text| {
                let lower = text.to_ascii_lowercase();
                if lower.contains("dedup") || lower.contains("duplicad") {
                    vec![1.0, 0.0, 0.0]
                } else if lower.contains("guard") {
                    vec![0.0, 1.0, 0.0]
                } else {
                    vec![-1.0, -1.0, -1.0]
                }
            })
            .collect())
    }
}

/// Configuração do canal semântico para os testes (sem prefixo: documento == consulta).
fn semantic_config() -> Embeddings {
    Embeddings {
        provider: EmbeddingProvider::Http,
        dimensions: 3,
        prefix: "none".to_owned(),
        ..Embeddings::default()
    }
}

#[test]
fn semantic_channel_recovers_a_paraphrase_that_lexical_misses() -> Result<(), String> {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1_700_000_000_000));
    let config = semantic_config();
    let embedder = KeywordEmbedder::new(config.dimensions);
    let memory =
        JsonlMemory::new(&fs, &clock, ROOT, MEM).with_embeddings(&embedder, config.clone());
    let _written = memory.write(&draft(MemoryKind::Fact, "o dedup usa o body_hash"));

    // Sem drenar não há índice: o canal semântico desliga sem aviso.
    let cold = memory
        .recall("como se detectam duplicados", 5)
        .map_err(|error| error.to_string())?;
    assert_eq!(cold.len(), 0, "lexical nao encontra a parafrase");

    let notes = memory.load_notes().map_err(|error| error.to_string())?;
    let outcome = digest(
        &fs,
        memory.memory_dir(),
        &config,
        &notes,
        &embedder,
        Rebuild::Keep,
    )
    .map_err(|error| error.to_string())?;
    assert_eq!(outcome.indexed, 1);
    assert_eq!(outcome.failure, None);

    let warm = memory
        .recall("como se detectam duplicados", 5)
        .map_err(|error| error.to_string())?;
    assert_eq!(warm.len(), 1, "o canal semantico entra na fusao RRF");
    assert!(
        warm.first()
            .is_some_and(|hint| hint.statement.contains("dedup")),
        "{warm:?}"
    );

    // Determinismo: duas execuções produzem o mesmo resultado.
    let again = memory
        .recall("como se detectam duplicados", 5)
        .map_err(|error| error.to_string())?;
    assert_eq!(warm, again);
    assert_eq!(embedder.meta().dimensions, 3);
    Ok(())
}

#[test]
fn unavailable_provider_degrades_to_lexical_without_error() -> Result<(), String> {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(1));
    let embedder = UnavailableEmbedder::new();
    let memory =
        JsonlMemory::new(&fs, &clock, ROOT, MEM).with_embeddings(&embedder, semantic_config());
    let _written = memory.write(&draft(MemoryKind::Fact, "o guard confina paths ao root"));
    let hints = memory
        .recall("guard confina paths", 5)
        .map_err(|error| error.to_string())?;
    assert_eq!(hints.len(), 1, "o recall lexical continua a funcionar");
    Ok(())
}
