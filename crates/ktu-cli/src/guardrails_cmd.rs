//! `ktu guardrails` — inspecciona os guardrails declarativos (§2.8).

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::guardrails;
use ktu_tools::StdFs;

use crate::output::{self, Format};
use crate::root_or_cwd;

/// Lista as regras declarativas carregadas (fail-closed).
pub(crate) fn list(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let dir = root.join(".agents").join("guardrails");
    match guardrails::load(&fs, &dir) {
        Ok(rules) => {
            let ids: Vec<String> = rules.iter().map(|rule| rule.id.clone()).collect();
            output::emit(
                format,
                "guardrails list",
                &ids.join("\n"),
                &serde_json::json!({ "rules": ids }),
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
