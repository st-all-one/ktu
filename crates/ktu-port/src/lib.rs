//! `ktu-port` — a **porta única** e agnóstica: protocolo versionado (NDJSON/SSE)
//! com duas implementações de transporte (`Channel` in-process e `Http` sobre
//! HTTP/1.1 + HTTP/2). Não contém lógica de agente (§2.3 do `plan.md`).
//!
//! Front-ends (CLI, TUI, web) falam **apenas** por [`Port`]; o kernel implementa
//! [`Host`] e nunca partilha estado com quem observa (I8).

#![forbid(unsafe_code)]

pub mod channel;
pub mod host;
pub mod http;
pub mod port;
pub mod protocol;

pub use channel::Channel;
pub use host::{Host, SharedHost};
pub use http::HttpPort;
pub use port::{EventStream, Port};
pub use protocol::{
    ClientMsg, Health, PROTOCOL_VERSION, PortError, ServerMsg, SessionInfo, SessionState,
};

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
