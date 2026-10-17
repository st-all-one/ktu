//! `ktu init` — cria o layout `.agents/` (conhecimento) e `.ktu/` (runtime).
//!
//! O **plano** do layout é puro ([`ktu_core::layout`]); aqui apenas se executa o
//! efeito no sistema de ficheiros, na borda. É idempotente: só cria o que falta.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_core::guardrails;
use ktu_core::layout;

/// Cria o layout sob `root`. Devolve os caminhos criados (relativos), por ordem.
///
/// Uma segunda chamada sobre um layout já existente devolve um vector vazio.
pub(crate) fn run(root: &Path) -> io::Result<Vec<&'static str>> {
    let mut created = Vec::new();
    for dir in layout::INIT_DIRS {
        let path = root.join(dir);
        if !path.is_dir() {
            fs::create_dir_all(&path)?;
            created.push(*dir);
        }
    }
    let notes = root.join(layout::NOTES_FILE);
    if !notes.is_file() {
        fs::write(&notes, [])?;
        created.push(layout::NOTES_FILE);
    }
    // *Seed* dos guardrails (§2.8): só o que falta, nunca sobrescreve.
    for (name, body) in guardrails::SEED {
        let path = root.join(".agents").join("guardrails").join(name);
        if !path.is_file() {
            fs::write(&path, body)?;
            created.push(name);
        }
    }
    Ok(created)
}

/// Executa `ktu init` no directório actual e reporta o que foi criado em `stdout`.
pub(crate) fn run_command() -> ExitCode {
    let Ok(root) = env::current_dir() else {
        report_error("nao foi possivel determinar o directório actual");
        return ExitCode::FAILURE;
    };
    match run(&root) {
        Ok(created) => {
            report_created(&created, &root);
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Escreve em `stdout` (dado) os caminhos criados.
fn report_created(created: &[&str], root: &Path) {
    let mut stdout = io::stdout().lock();
    if created.is_empty() {
        let _ignored = writeln!(stdout, "layout ja existe em {}", root.display());
    } else {
        for path in created {
            let _ignored = writeln!(stdout, "criado {path}");
        }
    }
}

/// Escreve em `stderr` (log) uma mensagem de erro da borda.
fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io;
    use std::process;

    use ktu_core::guardrails;
    use ktu_core::layout;

    use super::run;

    #[test]
    fn init_creates_layout_idempotently() -> io::Result<()> {
        let suffix = process::id();
        let root = env::temp_dir().join(format!("ktu-init-test-{suffix}"));
        let _ignored = fs::remove_dir_all(&root);

        let created = run(&root)?;
        assert_eq!(
            created.len(),
            layout::INIT_DIRS
                .len()
                .saturating_add(1)
                .saturating_add(guardrails::SEED.len())
        );
        assert!(root.join(".agents/memory/notes.jsonl").is_file());
        assert!(root.join(".ktu/sessions").is_dir());
        assert!(root.join(".agents/guardrails/deny-dotenv.md").is_file());

        let second = run(&root)?;
        assert_eq!(second, Vec::<&'static str>::new());

        let _ignored = fs::remove_dir_all(&root);
        Ok(())
    }
}
