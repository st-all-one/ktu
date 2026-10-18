//! `ktu agents` — inspecciona os agentes declarativos (`.agents/agents/*.md`).

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::agent_file;
use ktu_core::layout::AGENTS_FILES_DIR;
use ktu_tools::{StdFs, default_registry};

use crate::output::{self, Format};
use crate::root_or_cwd;
use crate::run::report_error;

/// Linha do `ktu agents ls` (um agente declarativo).
struct Row {
    name: String,
    mode: &'static str,
    tools: Vec<String>,
}

/// Lista os agentes declarativos com as tools **efectivas** (fail-closed).
pub(crate) fn list(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let catalog = default_registry().primary().tool_names();
    match agent_file::load(&fs, &root.join(AGENTS_FILES_DIR)) {
        Ok(agents) => {
            let rows: Vec<Row> = agents
                .iter()
                .map(|agent| Row {
                    name: agent.name().to_owned(),
                    mode: agent.mode().as_str(),
                    tools: agent.tools(&catalog),
                })
                .collect();
            let text = if rows.is_empty() {
                "sem agentes".to_owned()
            } else {
                rows.iter()
                    .map(|row| format!("{}\t{}\t{}", row.name, row.mode, row.tools.join(",")))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let data: Vec<serde_json::Value> = rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "name": row.name,
                        "mode": row.mode,
                        "tools": row.tools,
                    })
                })
                .collect();
            output::emit(
                format,
                "agents list",
                &text,
                &serde_json::json!({ "agents": data }),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&format!("agentes invalidos: {error}"));
            output::emit_error(format, "agents list", "invalid", &error);
            ExitCode::FAILURE
        }
    }
}
