//! `ktu commands` — inspecciona os comandos do projecto (`.agents/commands/*.md`).

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::command;
use ktu_tools::StdFs;

use crate::config_cmd::global_dir_path;
use crate::output::{self, Format};
use crate::root_or_cwd;
use crate::run::report_error;

/// Lista as receitas **efectivas** (global + projecto) com a origem de cada uma.
///
/// O local ganha por nome (M8); os problemas dos dois níveis vão para `stderr`.
pub(crate) fn list(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let user_dir = global_dir_path();
    let (commands, problems) = command::load_layered(&fs, user_dir.as_deref(), &root);
    for problem in &problems {
        report_error(&format!("comando ignorado: {problem}"));
    }
    let text = if commands.is_empty() {
        "sem comandos".to_owned()
    } else {
        commands
            .iter()
            .map(|(origin, item)| {
                format!(
                    "{}\t{}\t{}",
                    item.name(),
                    item.description(),
                    origin.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data: Vec<serde_json::Value> = commands
        .iter()
        .map(|(origin, item)| {
            serde_json::json!({
                "name": item.name(),
                "description": item.description(),
                "path": item.path().display().to_string(),
                "origin": origin.as_str(),
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
