//! Porta de métricas de performance.
//!
//! Métricas por função são **amostras** nomeadas (duração em microssegundos) e
//! contadores; o *export* JSON deriva de um [`MetricsSnapshot`]. Com a feature
//! `instrument` desligada (por omissão) a macro [`crate::instrument!`] expande
//! apenas para o corpo — **custo zero**.

use std::collections::BTreeMap;

use serde::Serialize;

/// Amostra de métrica, com nome estável.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// Duração de uma operação nomeada, em microssegundos.
    DurationMicros {
        /// Nome estável (ex.: `"derive_messages"`).
        name: &'static str,
        /// Duração em microssegundos.
        micros: u64,
    },
    /// Incremento de um contador nomeado.
    Counter {
        /// Nome estável.
        name: &'static str,
        /// Incremento.
        delta: u64,
    },
}

/// Destino de métricas.
pub trait Metrics: Send + Sync {
    /// Regista uma amostra.
    fn record(&self, metric: Metric);
}

/// Vista serializável e determinística das métricas acumuladas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetricsSnapshot {
    /// Durações observadas, por nome (ordem determinística).
    pub durations_micros: BTreeMap<String, Vec<u64>>,
    /// Contadores acumulados, por nome.
    pub counters: BTreeMap<String, u64>,
}
