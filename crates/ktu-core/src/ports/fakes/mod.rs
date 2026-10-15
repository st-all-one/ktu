//! *Fakes* determinísticos das portas, para testes.
//!
//! Nenhum teste do núcleo toca no relógio, no ambiente, no sistema de ficheiros
//! nem no log reais: usa estas implementações (disciplina herdada do knudge).
//! O estado partilhado usa `Mutex` (nunca `RefCell`/`Cell`, proibidos pelo
//! `clippy.toml`).

mod clock;
mod env;
mod fs;
mod logger;
mod metrics;
mod rng;

pub use clock::{FixedClock, SeqClock};
pub use env::MapEnv;
pub use fs::MemFs;
pub use logger::{NoopLogger, OwnedRecord, RecordingLogger};
pub use metrics::{NoopMetrics, RecordingMetrics};
pub use rng::SeqRng;
