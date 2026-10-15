//! Integração de `ktu-memory`: criação, recall (ponteiros), dedup idempotente,
//! persistência entre instâncias e projecções de leitura.

use ktu_core::memory::{Memory, MemoryDraft, MemoryKind, MemoryWrite};
use ktu_core::ports::{FixedClock, MemFs, Timestamp};
use ktu_memory::{JsonlMemory, export_markdown, health};

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
