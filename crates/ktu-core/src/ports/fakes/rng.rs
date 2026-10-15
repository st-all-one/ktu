//! *Fakes* da porta [`Rng`](super::super::Rng).

use std::sync::atomic::{AtomicU64, Ordering};

use super::super::Rng;

/// Gerador determinístico: cicla uma sequência de valores.
#[derive(Debug)]
pub struct SeqRng {
    next: AtomicU64,
    values: Vec<u64>,
}

impl SeqRng {
    /// Cria um gerador que cicla `values` pela ordem dada.
    #[must_use]
    pub fn new(values: Vec<u64>) -> Self {
        Self {
            next: AtomicU64::new(0),
            values,
        }
    }
}

impl Rng for SeqRng {
    fn next_u64(&self) -> u64 {
        let count = u64::try_from(self.values.len()).unwrap_or(0);
        if count == 0 {
            return 0;
        }
        let sequence = self.next.fetch_add(1, Ordering::Relaxed);
        let position = sequence.rem_euclid(count);
        let index = usize::try_from(position).unwrap_or(0);
        self.values.get(index).copied().unwrap_or(0)
    }
}
