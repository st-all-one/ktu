//! `ktu trash` — listar e restaurar ficheiros movidos para o *trash*.

use std::path::Path;
use std::process::ExitCode;

use ktu_tools::StdFs;
use ktu_tools::trash;

use crate::output::{self, Format};

/// Lista as entradas do *trash* em `stdout`.
pub(crate) fn list(root: &Path, format: Format) -> ExitCode {
    let fs = StdFs;
    match trash::list(&fs, root) {
        Ok(entries) => {
            let text = if entries.is_empty() {
                "trash vazio".to_owned()
            } else {
                entries
                    .iter()
                    .map(|entry| format!("{}\t{}", entry.id, entry.original))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let data: Vec<serde_json::Value> = entries
                .iter()
                .map(|entry| serde_json::json!({ "id": entry.id, "original": entry.original }))
                .collect();
            output::emit(
                format,
                "trash list",
                &text,
                &serde_json::json!({ "entries": data }),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(format, "trash list", "io", &error.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Restaura a entrada `id` para o caminho original.
pub(crate) fn restore(root: &Path, id: &str, format: Format) -> ExitCode {
    let fs = StdFs;
    match trash::restore(&fs, root, id) {
        Ok(original) => {
            output::emit(
                format,
                "trash restore",
                &format!("restaurado: {original}"),
                &serde_json::json!({ "original": original }),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(format, "trash restore", "not_found", &error.to_string());
            ExitCode::FAILURE
        }
    }
}
