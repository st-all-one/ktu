//! `ktu plugins` — inspecciona e recarrega plugins externos (§2.9).
//!
//! `stdout` = dados (nomes, geração); `stderr` = logs/erros.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ktu_core::Origin;
use ktu_core::plugin::Hooks;
use ktu_plugin::PluginHost;
use ktu_plugin::host::PLUGINS_DIR;

use crate::cli::PluginsAction;
use crate::config_cmd::global_dir_path;
use crate::output::{self, Format};
use crate::root_or_cwd;

/// Executa a acção pedida.
pub(crate) fn run(action: PluginsAction, format: Format) -> ExitCode {
    match action {
        PluginsAction::List { root } => list(&root_or_cwd(root), format),
        PluginsAction::Reload { root } => reload(&root_or_cwd(root), format),
        PluginsAction::Doctor { root } => doctor(&root_or_cwd(root), format),
    }
}

/// Carrega o *host* de plugins dos **dois** níveis (global + projecto).
fn host(root: &Path) -> PluginHost {
    PluginHost::layered(levels(root))
}

/// Níveis de plugins, por ordem de execução: global do utilizador, depois o projecto.
pub(crate) fn levels(root: &Path) -> Vec<(Origin, PathBuf)> {
    let mut levels: Vec<(Origin, PathBuf)> = Vec::new();
    if let Some(dir) = global_dir_path() {
        levels.push((Origin::Global, dir.join(PLUGINS_DIR)));
    }
    levels.push((Origin::Local, root.join(".ktu").join("plugins")));
    levels
}

/// Estado do host como dado (envelope JSON, com a origem de cada plugin).
fn state_data(host: &PluginHost) -> serde_json::Value {
    let plugins: Vec<serde_json::Value> = host
        .entries()
        .into_iter()
        .map(|(name, origin)| serde_json::json!({ "name": name, "origin": origin.as_str() }))
        .collect();
    serde_json::json!({
        "generation": host.generation(),
        "names": host.names(),
        "plugins": plugins,
        "errors": host.errors(),
    })
}

/// Texto humano do estado (geração + `nome\torigem`).
fn state_text(host: &PluginHost) -> String {
    let mut lines = vec![format!("geracao {}", host.generation())];
    lines.extend(
        host.entries()
            .into_iter()
            .map(|(name, origin)| format!("{name}\t{}", origin.as_str())),
    );
    lines.join("\n")
}

/// Escreve em `stderr` os erros de manifesto (logs).
fn report_errors(errors: &[String]) {
    let mut err = io::stderr().lock();
    for error in errors {
        let _ignored = writeln!(err, "ktu: plugin invalido: {error}");
    }
}

fn list(root: &Path, format: Format) -> ExitCode {
    let host = host(root);
    output::emit(
        format,
        "plugins list",
        &state_text(&host),
        &state_data(&host),
    );
    report_errors(&host.errors());
    ExitCode::SUCCESS
}

fn reload(root: &Path, format: Format) -> ExitCode {
    let host = host(root);
    let errors = host.reload();
    output::emit(
        format,
        "plugins reload",
        &state_text(&host),
        &state_data(&host),
    );
    report_errors(&errors);
    ExitCode::SUCCESS
}

fn doctor(root: &Path, format: Format) -> ExitCode {
    let host = host(root);
    let errors = host.errors();
    if errors.is_empty() {
        output::emit(
            format,
            "plugins doctor",
            &format!("plugins: ok (geracao {})", host.generation()),
            &state_data(&host),
        );
        ExitCode::SUCCESS
    } else {
        output::emit(
            format,
            "plugins doctor",
            &state_text(&host),
            &state_data(&host),
        );
        report_errors(&errors);
        ExitCode::FAILURE
    }
}
