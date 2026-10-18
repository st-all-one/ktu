//! `ktu-memory` — memória local e persistente entre sessões (§2.7).
//!
//! A **fonte da verdade** é `.agents/memory/notes.jsonl` (uma linha JSON por
//! nota); o índice é **derivado** e reconstruível. O motor de recuperação é o
//! `knudge-core` (BM25 + âncoras + RRF, dedup por conteúdo), consumido **em
//! memória** através da porta [`ktu_core::memory::Memory`] — nunca acoplado ao
//! kernel. Os `hints` de recall são **ponteiros** (`id` + `statement` + `score`),
//! nunca o corpo.
//!
//! O canal **semântico** (H8) é opcional: quando há provedor ([`ktu_core::ports::Embedder`])
//! e índice derivado, os ids do canal vetorial entram na **fusão RRF** do recall; sem
//! provedor, o recall é lexical e o canal desliga **sem aviso**.
#![forbid(unsafe_code)]

mod convert;
pub mod drain;
pub mod embed;
pub mod project;
mod report;
mod store;

pub use drain::{DrainOutcome, DrainStatus, Rebuild};
pub use knudge_core::store::Note;
pub use report::{MemoryHealth, export_markdown, health};
pub use store::JsonlMemory;

/// Agente que produz as notas deste store (proveniência PROV-lite, D207).
pub const AGENT: &str = "ktu";
/// Actividade de uma nota escrita directamente pelo agente.
pub const ACTIVITY_WRITE: &str = "write";
/// Actividade de uma nota **derivada** (projecção do log de sessão).
pub const ACTIVITY_DERIVE: &str = "derive";

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
