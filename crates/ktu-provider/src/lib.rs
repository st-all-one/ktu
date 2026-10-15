//! `ktu-provider` — o provider **built-in**: llama.cpp via `llama-server`.
//!
//! HTTP `POST /v1/chat/completions` (`stream: true`, `tools: [...]`), SSE
//! parseado em deltas. Tool-calls nativas quando o template do modelo suporta;
//! *fallback* para parsing de tool-call em texto. Outros providers só via GDK
//! (feature `gdk`), num crate separado (§2.5 do `plan.md`).
#![forbid(unsafe_code)]

pub mod doctor;
pub mod fallback;
pub mod gguf;
pub mod llama;
pub mod sampling;
pub mod sse;
pub mod stream;
pub mod wire;

pub use doctor::{DoctorReport, Probe, diagnose};
pub use gguf::GgufMeta;
pub use llama::{DEFAULT_BASE_URL, LlamaConfig, LlamaProvider};
pub use sampling::{ReasoningEffort, Sampling};
pub use wire::Flavor;

/// Nome canónico do crate (usado em logs estruturados).
pub const CRATE: &str = "ktu-provider";

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_is_reported() {
        assert_eq!(CRATE, "ktu-provider");
    }
}
