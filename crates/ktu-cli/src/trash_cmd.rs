//! `ktu trash` — listar e restaurar ficheiros movidos para o *trash*.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_tools::StdFs;
use ktu_tools::trash;

/// Lista as entradas do *trash* em `stdout`.
pub(crate) fn list(root: &Path) -> ExitCode {
    let fs = StdFs;
    match trash::list(&fs, root) {
        Ok(entries) => {
            let mut stdout = io::stdout().lock();
            if entries.is_empty() {
                let _ignored = writeln!(stdout, "trash vazio");
            }
            for entry in entries {
                let _ignored = writeln!(stdout, "{}\t{}", entry.id, entry.original);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Restaura a entrada `id` para o caminho original.
pub(crate) fn restore(root: &Path, id: &str) -> ExitCode {
    let fs = StdFs;
    match trash::restore(&fs, root, id) {
        Ok(original) => {
            let mut stdout = io::stdout().lock();
            let _ignored = writeln!(stdout, "restaurado: {original}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}

fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
