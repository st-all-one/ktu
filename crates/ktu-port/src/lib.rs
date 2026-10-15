//! `ktu-port` — a **porta única** e agnóstica: protocolo versionado (NDJSON/SSE)
//! com duas implementações de transporte (`Channel` in-process e `Http` sobre
//! HTTP/1.1 + HTTP/2). Não contém lógica de agente (§2.3 do `plan.md`).
#![forbid(unsafe_code)]

/// Nome canónico do crate (usado em logs estruturados).
pub const CRATE: &str = "ktu-port";

#[cfg(test)]
mod tests {
    use super::CRATE;

    #[test]
    fn crate_name_is_reported() {
        assert_eq!(CRATE, "ktu-port");
    }
}
