//! Adaptador de log da borda: implementa a porta `Logger` do core com `tracing`.
//!
//! **stdout = dados, stderr = logs** (§1.3 do `epic.md`). O core não conhece
//! `tracing`; é aqui que os eventos estruturados ganham destino. O `event` é um
//! id estável ([`ktu_core::events`]) e viaja como campo, não como texto livre.

use std::io;

use ktu_core::ports::{Level, LogRecord, Logger};
use tracing::subscriber::set_global_default;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

/// Logger que emite registos estruturados (JSON) para `stderr`.
#[derive(Debug, Default)]
pub(crate) struct TracingLogger;

impl Logger for TracingLogger {
    fn log(&self, record: &LogRecord<'_>) {
        match record.level {
            Level::Error => {
                tracing::error!(event = record.event, message = record.message, fields = ?record.fields);
            }
            Level::Warn => {
                tracing::warn!(event = record.event, message = record.message, fields = ?record.fields);
            }
            Level::Info => {
                tracing::info!(event = record.event, message = record.message, fields = ?record.fields);
            }
            Level::Debug => {
                tracing::debug!(event = record.event, message = record.message, fields = ?record.fields);
            }
            Level::Trace => {
                tracing::trace!(event = record.event, message = record.message, fields = ?record.fields);
            }
        }
    }
}

/// Configura o subscritor global (JSON em `stderr`, nível por `RUST_LOG`,
/// por omissão `info`) e devolve o logger da borda.
#[must_use]
pub(crate) fn init() -> TracingLogger {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_ignored| EnvFilter::new("info"));
    let subscriber = fmt()
        .json()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .finish();
    let _ignored = set_global_default(subscriber);
    TracingLogger
}
