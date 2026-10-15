//! Adaptador real da porta `Clock` (borda).
//!
//! Aqui — e só aqui — se lê o relógio real do SO; o domínio recebe apenas
//! [`Timestamp`](ktu_core::ports::Timestamp) e microssegundos monótonos.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ktu_core::ports::{Clock, Timestamp};

/// Relógio de parede + monótono do SO.
#[derive(Debug)]
pub(crate) struct SystemClock {
    start: Instant,
}

impl SystemClock {
    /// Cria um relógio, fixando a base monótona.
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de borda: base monótona real"
    )]
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de borda: relógio de parede real"
    )]
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        Timestamp::from_millis(i64::try_from(millis).unwrap_or(i64::MAX))
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de borda: relógio monótono real"
    )]
    fn monotonic_micros(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
}
