//! Dreno da **fila derivada** de embeddings (H8): estado, digestão e `--force`.
//!
//! A fila não é armazenada: é a diferença entre as notas e o índice vetorial
//! (`indexed` / `pending` / `stale`, classificado pelo `body_hash`). O dreno é
//! **off-path** e em lotes, grava uma vez por lote e **nunca descarta nota**: uma
//! falha do provedor deixa-a pendente (degradação graciosa).

use std::path::Path;

use knudge_core::embeddings::{EmbeddingIndex, EmbeddingState, Prefix};
use knudge_core::schema::Value;
use knudge_core::store::Note;
use ktu_core::memory::MemoryResult;
use ktu_core::memory_config::Embeddings;
use ktu_core::ports::{Embedder, Fs};

use crate::embed::{engine_meta, load, save};

/// Modo de reconstrução do derivado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rebuild {
    /// Digere o que falta, reaproveitando o índice existente.
    Keep,
    /// Ignora o índice existente e re-embeda **todas** as notas (`--force`).
    Wipe,
}

impl Rebuild {
    /// `true` se descarta o índice existente.
    #[must_use]
    pub const fn wipes(self) -> bool {
        matches!(self, Self::Wipe)
    }
}

/// Canal do recall: lexical (sem provedor) ou lexical **+** semântico.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// Só lexical (BM25 + âncoras); o canal semântico está desligado.
    Lexical,
    /// Lexical + semântico (há provedor configurado).
    Semantic,
}

impl Channel {
    /// `true` se o canal semântico está ligado.
    #[must_use]
    pub const fn is_semantic(self) -> bool {
        matches!(self, Self::Semantic)
    }
}

/// Presença do índice vetorial derivado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexState {
    /// Sem índice (nunca digerido, invalidado ou ilegível).
    Absent,
    /// Índice reaproveitável (identidade compatível).
    Present,
}

impl IndexState {
    /// `true` se o índice existe e é reaproveitável.
    #[must_use]
    pub const fn is_present(self) -> bool {
        matches!(self, Self::Present)
    }
}

/// Estado do canal semântico (derivado; **nunca** fonte).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrainStatus {
    /// Provedor configurado.
    pub provider: String,
    /// Modelo configurado.
    pub model: String,
    /// Dimensão configurada.
    pub dimensions: usize,
    /// Canal do recall.
    pub channel: Channel,
    /// Presença do índice derivado.
    pub index: IndexState,
    /// Vetores indexados.
    pub indexed: usize,
    /// Notas sem vetor.
    pub pending: usize,
    /// Notas com vetor desactualizado (o corpo mudou).
    pub stale: usize,
}

/// Resultado de um dreno.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DrainOutcome {
    /// Vetores gravados neste dreno.
    pub indexed: usize,
    /// Notas ainda pendentes no fim.
    pub pending: usize,
    /// Notas desactualizadas observadas.
    pub stale: usize,
    /// Lotes processados.
    pub batches: usize,
    /// Falha do provedor (a fila fica pendente; o recall continua lexical).
    pub failure: Option<String>,
}

/// Nota por embedar (id + `body_hash` + texto canónico).
struct PendingNote {
    id: String,
    body_hash: String,
    text: String,
}

/// Estado da fila (read-only: nunca toca no índice).
///
/// # Errors
/// Propaga erros de I/O e de configuração (prefixo inválido).
pub fn status(
    fs: &dyn Fs,
    memory_dir: &Path,
    config: &Embeddings,
    notes: &[Note],
) -> MemoryResult<DrainStatus> {
    let mut status = DrainStatus {
        provider: config.provider.as_str().to_owned(),
        model: config.model.clone(),
        dimensions: config.dimensions,
        channel: if config.enabled() {
            Channel::Semantic
        } else {
            Channel::Lexical
        },
        index: IndexState::Absent,
        indexed: 0,
        pending: 0,
        stale: 0,
    };
    if !config.enabled() {
        return Ok(status);
    }
    let expected = engine_meta(config)?;
    let Some(index) = load(fs, memory_dir, &expected)? else {
        status.pending = notes.len();
        return Ok(status);
    };
    let (queue, stale) = pending(notes, &index, expected.prefix);
    status.index = IndexState::Present;
    status.indexed = index.len();
    status.pending = queue.len();
    status.stale = stale;
    Ok(status)
}

/// Digere a fila em lotes até esvaziar (ou até o provedor falhar).
///
/// # Errors
/// Propaga erros de I/O, de configuração e de serialização do índice.
#[allow(
    clippy::too_many_arguments,
    reason = "dreno explicito: portas + configuracao + notas; agrupar esconderia a injecao"
)]
pub fn digest(
    fs: &dyn Fs,
    memory_dir: &Path,
    config: &Embeddings,
    notes: &[Note],
    embedder: &dyn Embedder,
    rebuild: Rebuild,
) -> MemoryResult<DrainOutcome> {
    if !config.enabled() {
        return Ok(DrainOutcome {
            pending: notes.len(),
            ..DrainOutcome::default()
        });
    }
    let expected = engine_meta(config)?;
    let mut index = if rebuild.wipes() {
        EmbeddingIndex::new(expected.clone())
    } else {
        load(fs, memory_dir, &expected)?.unwrap_or_else(|| EmbeddingIndex::new(expected.clone()))
    };
    let batch = config.batch();
    let mut outcome = DrainOutcome::default();
    loop {
        let (queue, stale) = pending(notes, &index, expected.prefix);
        outcome.stale = outcome.stale.max(stale);
        outcome.pending = queue.len();
        if queue.is_empty() {
            break;
        }
        let texts: Vec<String> = queue
            .iter()
            .take(batch)
            .map(|item| item.text.clone())
            .collect();
        let vectors = match embedder.embed(&texts) {
            Ok(vectors) => vectors,
            Err(error) => {
                outcome.failure = Some(error.to_string());
                break;
            }
        };
        if let Some(failure) = shape_failure(&vectors, texts.len(), expected.dimensions) {
            outcome.failure = Some(failure);
            break;
        }
        for (item, vector) in queue.iter().zip(vectors) {
            index.insert(&item.id, &item.body_hash, vector);
        }
        save(fs, memory_dir, &index)?;
        outcome.indexed = outcome.indexed.saturating_add(texts.len());
        outcome.batches = outcome.batches.saturating_add(1);
        outcome.pending = queue.len().saturating_sub(texts.len());
    }
    Ok(outcome)
}

/// Valida a resposta do provedor (um vetor por texto, dimensão esperada).
fn shape_failure(vectors: &[Vec<f32>], expected_count: usize, dimensions: usize) -> Option<String> {
    if vectors.len() != expected_count {
        return Some(format!(
            "provedor devolveu {} vetor(es) para {expected_count} texto(s)",
            vectors.len()
        ));
    }
    if vectors.iter().any(|vector| vector.len() != dimensions) {
        return Some(format!(
            "provedor devolveu vetor(es) fora da dimensao {dimensions}"
        ));
    }
    None
}

/// Fila derivada: notas sem vetor (`pending`) ou com o corpo mudado (`stale`).
fn pending(notes: &[Note], index: &EmbeddingIndex, prefix: Prefix) -> (Vec<PendingNote>, usize) {
    let mut queue = Vec::new();
    let mut stale = 0_usize;
    for note in notes {
        let Ok(id) = note.id() else {
            continue;
        };
        let Some(body_hash) = note.frontmatter.get("body_hash").and_then(Value::as_str) else {
            continue;
        };
        match index.state_of(id, body_hash) {
            EmbeddingState::Indexed => continue,
            EmbeddingState::Stale => stale = stale.saturating_add(1),
            EmbeddingState::Pending => {}
        }
        let statement = note.frontmatter.statement().unwrap_or("");
        queue.push(PendingNote {
            id: id.to_owned(),
            body_hash: body_hash.to_owned(),
            text: prefix.document(statement, &note.body),
        });
    }
    (queue, stale)
}

#[cfg(test)]
mod tests;
