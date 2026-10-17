//! `ktu guardrails` — inspecciona os guardrails declarativos (§2.8).

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::guardrails;
use ktu_tools::StdFs;

use crate::root_or_cwd;

/// Lista as regras declarativas carregadas (fail-closed).
pub(crate) fn list(root: Option<PathBuf>) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let dir = root.join(".agents").join("guardrails");
    match guardrails::load(&fs, &dir) {
        Ok(rules) => {
            let mut out = io::stdout().lock();
            for rule in rules {
                let _ignored = writeln!(out, "{}", rule.id);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            let mut err = io::stderr().lock();
            let _ignored = writeln!(err, "ktu: guardrails invalidos: {error}");
            ExitCode::FAILURE
        }
    }
}
