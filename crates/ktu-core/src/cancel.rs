//! Sinal de cancelamento partilhável (I4).
//!
//! O mesmo sinal é consultado em todos os pontos de espera do loop; cancelar
//! nunca fecha um turno em silêncio — injecta resultados sintéticos para as
//! chamadas pendentes (I2) e termina com `StopReason::Cancelled`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Sinal de cancelamento clonável.
#[derive(Debug, Clone, Default)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    /// Cria um sinal não cancelado.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Marca como cancelado.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    /// Indica se foi cancelado.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}
