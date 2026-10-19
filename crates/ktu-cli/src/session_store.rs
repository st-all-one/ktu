//! Persistência de sessão do `ktu run` (carregar, gravar, checkpoint).
//!
//! Separado de `run.rs` para manter ficheiros pequenos e o `drive` legível. Uma
//! sessão **efémera** ([`SessionTarget::Ephemeral`](crate::session_mode::SessionTarget::Ephemeral))
//! não carrega nem grava nada.

use std::path::Path;

use ktu_core::checkpoint::{self, Checkpoint};
use ktu_core::ports::{Clock, Fs};
use ktu_core::session::{Event, SessionLog, title_of};
use ktu_core::store;

use crate::run::{RunConfig, report_error};
use crate::session_mode::force_compaction;

/// Aplica o arranque da sessão: checkpoint (se retoma) e compactação forçada.
pub(crate) fn apply_session_start(
    fs: &dyn Fs,
    dir: &Path,
    config: &RunConfig,
    log: &mut SessionLog,
) {
    apply_checkpoint(fs, dir, config, log);
    if config.mode.compacts() {
        force_compaction(log);
    }
}

/// Aplica ao log o corte de compactação registado no checkpoint (retoma idêntica).
fn apply_checkpoint(fs: &dyn Fs, dir: &Path, config: &RunConfig, log: &mut SessionLog) {
    if !config.mode.resumes() {
        return;
    }
    let Some(id) = config.session.id() else {
        return;
    };
    let Ok(Some(checkpoint)) = checkpoint::load(fs, dir, id) else {
        return;
    };
    if let Some(keep_from) = checkpoint.keep_from {
        log.set_compaction(usize::try_from(keep_from).unwrap_or(0));
    }
}

/// Grava o checkpoint tipado e durável (inclui o corte de compactação).
pub(crate) fn save_checkpoint(
    fs: &dyn Fs,
    dir: &Path,
    config: &RunConfig,
    log: &SessionLog,
    clock: &dyn Clock,
) {
    let Some(id) = config.session.id() else {
        return;
    };
    save_checkpoint_for(fs, dir, id, log, clock);
}

/// Grava o checkpoint de **uma** sessão pelo id (o actor do REPL não tem `RunConfig` da sessão).
pub(crate) fn save_checkpoint_for(
    fs: &dyn Fs,
    dir: &Path,
    id: &str,
    log: &SessionLog,
    clock: &dyn Clock,
) {
    let checkpoint = Checkpoint {
        session: id.to_owned(),
        turns: count_turns(log),
        compacted: log.is_compacted(),
        keep_from: log
            .compaction_from()
            .and_then(|value| u32::try_from(value).ok()),
        // Derivado do log (determinístico): o mesmo pedido dá o mesmo título.
        title: title_of(log),
        updated_ms: clock.now().as_millis(),
    };
    if let Err(error) = checkpoint::save(fs, dir, &checkpoint) {
        report_error(&format!("nao foi possivel gravar o checkpoint: {error}"));
    }
}

/// Conta os turnos concluídos no log (`TurnEnd`).
fn count_turns(log: &SessionLog) -> u32 {
    let count = log
        .events()
        .iter()
        .filter(|event| matches!(event, Event::TurnEnd { .. }))
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Carrega o log da sessão (efémera ou sem `--resume` ⇒ log novo).
pub(crate) fn load_session(
    fs: &dyn Fs,
    dir: &Path,
    config: &RunConfig,
) -> Result<SessionLog, String> {
    if !config.mode.resumes() {
        return Ok(SessionLog::new());
    }
    let Some(id) = config.session.id() else {
        return Ok(SessionLog::new());
    };
    match store::load_recovering(fs, dir, id) {
        Ok(Some(log)) => Ok(log),
        Ok(None) => Ok(SessionLog::new()),
        Err(error) => Err(format!("nao foi possivel retomar {id}: {error}")),
    }
}

/// Grava o log da sessão de forma durável (no-op se efémera).
pub(crate) fn save_session(
    fs: &dyn Fs,
    dir: &Path,
    config: &RunConfig,
    log: &SessionLog,
) -> ktu_core::Result<()> {
    let Some(id) = config.session.id() else {
        return Ok(());
    };
    store::save(fs, dir, id, log)
}
