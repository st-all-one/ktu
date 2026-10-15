//! *Fakes* da porta [`Logger`](super::super::Logger).

use std::sync::{Mutex, MutexGuard, PoisonError};

use super::super::{Level, LogRecord, Logger};

/// Log descartado (custo zero por omissão).
#[derive(Debug, Default)]
pub struct NoopLogger;

impl Logger for NoopLogger {
    fn log(&self, _record: &LogRecord<'_>) {}
}

/// Registro de log possuído (para inspecção em testes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedRecord {
    /// Nível.
    pub level: Level,
    /// Id do evento.
    pub event: String,
    /// Mensagem.
    pub message: String,
    /// Campos estruturados.
    pub fields: Vec<(String, String)>,
}

/// Logger em memória, para asserções determinísticas.
#[derive(Debug, Default)]
pub struct RecordingLogger {
    records: Mutex<Vec<OwnedRecord>>,
}

impl RecordingLogger {
    /// Cria um logger vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn records(&self) -> MutexGuard<'_, Vec<OwnedRecord>> {
        self.records.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Devolve uma cópia dos registos observados, pela ordem de emissão.
    #[must_use]
    pub fn snapshot(&self) -> Vec<OwnedRecord> {
        self.records().clone()
    }
}

impl Logger for RecordingLogger {
    fn log(&self, record: &LogRecord<'_>) {
        let owned = OwnedRecord {
            level: record.level,
            event: record.event.to_owned(),
            message: record.message.to_owned(),
            fields: record
                .fields
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        };
        self.records().push(owned);
    }
}
