//! `ktu commands` — inspecciona os comandos do projecto (`.agents/commands/*.md`).

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::command;
use ktu_core::layout::COMMANDS_DIR;
use ktu_tools::StdFs;

use crate::output::{self, Format};
use crate::root_or_cwd;
use crate::run::report_error;

/// Lista os comandos do projecto (determinístico; problemas vão para `stderr`).
pub(crate) fn list(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let (commands, problems) = command::load(&fs, &root.join(COMMANDS_DIR));
    for problem in &problems {
        report_error(&format!("comando ignorado: {problem}"));
    }
    let text = if commands.is_empty() {
        "sem comandos".to_owned()
    } else {
        commands
            .iter()
            .map(|item| format!("{}\t{}", item.name(), item.description()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data: Vec<serde_json::Value> = commands
        .iter()
        .map(|item| {
            serde_json::json!({
                "name": item.name(),
                "description": item.description(),
                "path": item.path().display().to_string(),
            })
        })
        .collect();
    output::emit(
        format,
        "commands list",
        &text,
        &serde_json::json!({ "commands": data, "problems": problems }),
    );
    ExitCode::SUCCESS
}
