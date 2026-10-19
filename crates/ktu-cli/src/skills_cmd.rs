//! `ktu skills` — inspecciona as skills declarativas (globais + do projecto).
//!
//! M8: uma skill compõe o system prompt, logo tem de **aparecer** num `ls`, com a
//! origem. O corpo não sai aqui (é longo); sai o nome, a descrição e o ficheiro.

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::skill;
use ktu_tools::StdFs;

use crate::cli::SkillsAction;
use crate::config_cmd::global_dir_path;
use crate::output::{self, Format};
use crate::root_or_cwd;
use crate::run::report_error;

/// Executa a acção pedida.
pub(crate) fn run(action: SkillsAction, format: Format) -> ExitCode {
    match action {
        SkillsAction::Ls { root } => list(root, format),
    }
}

/// Lista as skills efectivas (local ganha por nome) com a origem de cada uma.
fn list(root: Option<PathBuf>, format: Format) -> ExitCode {
    let root = root_or_cwd(root);
    let fs = StdFs;
    let user_dir = global_dir_path();
    let (skills, problems) = skill::load(&fs, user_dir.as_deref(), &root);
    for problem in &problems {
        report_error(&format!("skill ignorada: {problem}"));
    }
    let text = if skills.is_empty() {
        "sem skills".to_owned()
    } else {
        skills
            .iter()
            .map(|item| {
                format!(
                    "{}\t{}\t{}",
                    item.name(),
                    item.description(),
                    item.origin().as_str()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data: Vec<serde_json::Value> = skills
        .iter()
        .map(|item| {
            serde_json::json!({
                "name": item.name(),
                "description": item.description(),
                "path": item.path().display().to_string(),
                "origin": item.origin().as_str(),
            })
        })
        .collect();
    output::emit(
        format,
        "skills list",
        &text,
        &serde_json::json!({ "skills": data, "problems": problems }),
    );
    ExitCode::SUCCESS
}
