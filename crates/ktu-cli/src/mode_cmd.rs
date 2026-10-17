//! `ktu mode <plan|build>` — comando `SetMode` (§2.13).
//!
//! A transição é aplicada com a regra de aprovação (I15), **logada** como evento
//! `ModeSet` na sessão e por isso **sobrevivente a `resume`**.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_core::plan::{Approval, Mode};
use ktu_core::session::SessionLog;
use ktu_core::store;
use ktu_tools::StdFs;

/// Aplica e persiste a mudança de modo de uma sessão.
pub(crate) fn set(root: &Path, session: &str, target: &str, approval: Approval) -> ExitCode {
    let Some(target) = Mode::parse(target) else {
        report_error(&format!("modo desconhecido: {target} (use plan|build)"));
        return ExitCode::FAILURE;
    };
    let dir = root.join(".ktu").join("sessions");
    if let Err(error) = std::fs::create_dir_all(&dir) {
        report_error(&format!(
            "nao foi possivel criar {}: {error}",
            dir.display()
        ));
        return ExitCode::FAILURE;
    }
    let fs = StdFs;
    let mut log = match store::load_recovering(&fs, &dir, session) {
        Ok(Some(log)) => log,
        Ok(None) => SessionLog::new(),
        Err(error) => {
            report_error(&format!("nao foi possivel retomar {session}: {error}"));
            return ExitCode::FAILURE;
        }
    };
    let current = log.mode();
    match Mode::transition(current, target, approval) {
        Ok(mode) => {
            log.set_mode(mode);
            if let Err(error) = store::save(&fs, &dir, session, &log) {
                report_error(&format!("nao foi possivel gravar a sessao: {error}"));
                return ExitCode::FAILURE;
            }
            let mut stdout = io::stdout().lock();
            let _ignored = writeln!(stdout, "modo: {}", mode.as_str());
            ExitCode::SUCCESS
        }
        Err(message) => {
            report_error(&message);
            ExitCode::FAILURE
        }
    }
}

/// Escreve em `stderr` uma mensagem de erro da borda.
fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
