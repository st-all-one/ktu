//! `ktu guardrails` — inspecciona os guardrails declarativos (§2.8).

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::guardrails;
use ktu_tools::StdFs;

use crate::config_cmd::global_dir_path;
use crate::output::{self, Format};
use crate::root_or_cwd;

/// Lista as regras declarativas **efectivas** (fail-closed) com a origem de cada uma.
///
/// O piso global aparece primeiro: é ele que o projecto não pode desligar (M5/M8).
pub(crate) fn list(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let user_dir = global_dir_path();
    match guardrails::load_layered(&fs, user_dir.as_deref(), &root) {
        Ok(rules) => {
            let text = if rules.is_empty() {
                "sem guardrails".to_owned()
            } else {
                rules
                    .iter()
                    .map(|(origin, rule)| format!("{}\t{}", rule.id, origin.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let data: Vec<serde_json::Value> = rules
                .iter()
                .map(|(origin, rule)| {
                    serde_json::json!({ "id": rule.id, "origin": origin.as_str() })
                })
                .collect();
            output::emit(
                format,
                "guardrails list",
                &text,
                &serde_json::json!({ "rules": data }),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(
                format,
                "guardrails list",
                "invalid",
                &format!("guardrails invalidos: {error}"),
            );
            ExitCode::FAILURE
        }
    }
}
