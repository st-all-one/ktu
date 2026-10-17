//! `ktu-core` — núcleo **puro** do ktu: domínio, portas e o loop agêntico.
//!
//! Este crate não conhece `ktu-tools`, `ktu-provider`, `ktu-port` nem
//! `ktu-memory` (firewall de camadas, §2.1 do `plan.md`). Todos os efeitos
//! entram por **portas** (`Clock`, `Rng`, `Fs`, `Env`, `Logger`, `Metrics`) —
//! implementações reais na borda e *fakes* nos testes (determinismo, §4).
#![forbid(unsafe_code)]

pub mod agent;
pub mod budget;
pub mod cancel;
pub mod checkpoint;
pub mod compact;
pub mod domain;
pub mod error;
pub mod events;
pub mod guard;
pub mod guardrails;
pub mod layout;
pub mod memory;
pub mod plan;
pub mod plugin;
pub mod ports;
pub mod prompt;
pub mod provider;
pub mod session;
pub mod store;
pub mod subagent;
pub mod taint;
pub mod testing;
pub mod tools;

pub use error::{Error, Result};

/// Versão do crate, herdada do workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Mede a duração de um bloco/expressão e regista-a como
/// [`ports::Metric::DurationMicros`], quando a feature `instrument` está ligada.
///
/// Sem a feature (por omissão) expande apenas para o corpo: **custo zero**.
#[cfg(feature = "instrument")]
#[macro_export]
macro_rules! instrument {
    ($metrics:expr, $clock:expr, $name:literal, $body:expr) => {{
        let start = $clock.monotonic_micros();
        let output = $body;
        let end = $clock.monotonic_micros();
        $metrics.record($crate::ports::Metric::DurationMicros {
            name: $name,
            micros: end.saturating_sub(start),
        });
        output
    }};
}

/// Versão de custo zero da macro [`instrument!`](crate::instrument!) quando a
/// feature `instrument` está desligada (por omissão). Referencia os argumentos
/// para que as portas injectadas continuem “usadas”.
#[cfg(not(feature = "instrument"))]
#[macro_export]
macro_rules! instrument {
    ($metrics:expr, $clock:expr, $name:literal, $body:expr) => {{
        let _instrument_ignored = ($metrics, $clock, $name);
        $body
    }};
}

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_reported() {
        assert_ne!(VERSION, "");
    }
}
