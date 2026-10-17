//! `ktu plugins` — inspecciona e recarrega plugins externos (§2.9).
//!
//! `stdout` = dados (nomes, geração); `stderr` = logs/erros.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_core::plugin::Hooks;
use ktu_plugin::PluginHost;

use crate::cli::PluginsAction;
use crate::root_or_cwd;

/// Executa a acção pedida.
pub(crate) fn run(action: PluginsAction) -> ExitCode {
    match action {
        PluginsAction::List { root } => list(&root_or_cwd(root)),
        PluginsAction::Reload { root } => reload(&root_or_cwd(root)),
        PluginsAction::Doctor { root } => doctor(&root_or_cwd(root)),
    }
}

/// Carrega o *host* de plugins do projecto.
fn host(root: &Path) -> PluginHost {
    PluginHost::load(root.join(".ktu").join("plugins"))
}

/// Imprime a geração e os plugins activos.
fn print_state(host: &PluginHost) {
    let mut out = io::stdout().lock();
    let _ignored = writeln!(out, "geracao {}", host.generation());
    for name in host.names() {
        let _ignored = writeln!(out, "{name}");
    }
}

/// Escreve em `stderr` os erros de manifesto.
fn report_errors(errors: &[String]) {
    let mut err = io::stderr().lock();
    for error in errors {
        let _ignored = writeln!(err, "ktu: plugin invalido: {error}");
    }
}

fn list(root: &Path) -> ExitCode {
    let host = host(root);
    print_state(&host);
    report_errors(&host.errors());
    ExitCode::SUCCESS
}

fn reload(root: &Path) -> ExitCode {
    let host = host(root);
    let errors = host.reload();
    print_state(&host);
    report_errors(&errors);
    ExitCode::SUCCESS
}

fn doctor(root: &Path) -> ExitCode {
    let host = host(root);
    let errors = host.errors();
    if errors.is_empty() {
        let mut out = io::stdout().lock();
        let _ignored = writeln!(out, "plugins: ok (geracao {})", host.generation());
        ExitCode::SUCCESS
    } else {
        report_errors(&errors);
        ExitCode::FAILURE
    }
}
