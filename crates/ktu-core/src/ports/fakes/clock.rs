//! *Fakes* da porta [`Clock`](super::super::Clock).

use std::sync::atomic::{AtomicU64, Ordering};

use super::super::{Clock, Timestamp};

/// Relógio fixo: devolve sempre o mesmo instante e zero microssegundos.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock {
    now: Timestamp,
}

impl FixedClock {
    /// Cria um relógio fixo no instante dado.
    #[must_use]
    pub const fn new(now: Timestamp) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.now
    }

    fn monotonic_micros(&self) -> u64 {
        0
    }
}

/// Relógio de sequência: parede fixa, leituras monótonas pré-definidas (ciclo).
#[derive(Debug)]
pub struct SeqClock {
    wall: Timestamp,
    micros: Vec<u64>,
    next: AtomicU64,
}

impl SeqClock {
    /// Cria um relógio com a parede `wall` e a sequência monótona `micros`.
    #[must_use]
    pub fn new(wall: Timestamp, micros: Vec<u64>) -> Self {
        Self {
            wall,
            micros,
            next: AtomicU64::new(0),
        }
    }
}

impl Clock for SeqClock {
    fn now(&self) -> Timestamp {
        self.wall
    }

    fn monotonic_micros(&self) -> u64 {
        let count = u64::try_from(self.micros.len()).unwrap_or(0);
        if count == 0 {
            return 0;
        }
        let sequence = self.next.fetch_add(1, Ordering::Relaxed);
        let position = sequence.rem_euclid(count);
        let index = usize::try_from(position).unwrap_or(0);
        self.micros.get(index).copied().unwrap_or(0)
    }
}
