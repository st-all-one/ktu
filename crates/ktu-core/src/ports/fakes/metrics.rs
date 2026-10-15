//! *Fakes* da porta [`Metrics`](super::super::Metrics).

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::super::{Metric, Metrics, MetricsSnapshot};

/// Métricas descartadas (custo zero por omissão).
#[derive(Debug, Default)]
pub struct NoopMetrics;

impl Metrics for NoopMetrics {
    fn record(&self, _metric: Metric) {}
}

/// Métricas em memória, para testes e *export*.
#[derive(Debug, Default)]
pub struct RecordingMetrics {
    durations: Mutex<BTreeMap<&'static str, Vec<u64>>>,
    counters: Mutex<BTreeMap<&'static str, u64>>,
}

impl RecordingMetrics {
    /// Cria um coletor vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn durations(&self) -> MutexGuard<'_, BTreeMap<&'static str, Vec<u64>>> {
        self.durations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn counters(&self) -> MutexGuard<'_, BTreeMap<&'static str, u64>> {
        self.counters.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Vista determinística do que foi observado.
    #[must_use]
    pub fn snapshot(&self) -> MetricsSnapshot {
        let durations_micros = self
            .durations()
            .iter()
            .map(|(name, values)| ((*name).to_owned(), values.clone()))
            .collect();
        let counters = self
            .counters()
            .iter()
            .map(|(name, value)| ((*name).to_owned(), *value))
            .collect();
        MetricsSnapshot {
            durations_micros,
            counters,
        }
    }
}

impl Metrics for RecordingMetrics {
    fn record(&self, metric: Metric) {
        match metric {
            Metric::DurationMicros { name, micros } => {
                self.durations().entry(name).or_default().push(micros);
            }
            Metric::Counter { name, delta } => {
                let mut counters = self.counters();
                let current = counters.entry(name).or_default();
                *current = current.saturating_add(delta);
            }
        }
    }
}
