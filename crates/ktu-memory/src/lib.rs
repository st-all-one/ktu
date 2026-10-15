//! `ktu-memory` — memória local e persistente entre sessões (§2.7).
//!
//! A **fonte da verdade** é `.agents/memory/notes.jsonl` (uma linha JSON por
//! nota); o índice é **derivado** e reconstruível. O motor de recuperação é o
//! `knudge-core` (BM25 + âncoras + RRF, dedup por conteúdo), consumido **em
//! memória** através da porta [`ktu_core::memory::Memory`] — nunca acoplado ao
//! kernel. Os `hints` de recall são **ponteiros** (`id` + `statement` + `score`),
//! nunca o corpo.
#![forbid(unsafe_code)]

mod convert;
mod record;
mod report;
mod store;

pub use report::{MemoryHealth, export_markdown, health};
pub use store::JsonlMemory;

/// Nome canónico do crate (usado em logs estruturados).
pub const CRATE: &str = "ktu-memory";

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_is_reported() {
        assert_eq!(CRATE, "ktu-memory");
    }
}
