//! `ktu self` — utilitários do binário (versão e *upgrade*).
//!
//! `version` imprime a versão; `upgrade` corre o instalador oficial (release +
//! checksum) — o `install.sh` do repositório quando presente, ou imprime o
//! comando canónico se ausente (o humano decide correr).

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::cli::Cli;
use crate::output::{self, Format};

/// Comando canónico de instalação/actualização (release + checksum).
const INSTALL_CMD: &str =
    "curl -fsSL https://raw.githubusercontent.com/st-all-one/ktu/main/install.sh | bash";

/// Imprime a versão do binário em `stdout` (texto ou envelope JSON).
pub(crate) fn version(format: Format) -> ExitCode {
    let text = format!("ktu {}", ktu_core::VERSION);
    output::emit(
        format,
        "self version",
        &text,
        &serde_json::json!({ "version": ktu_core::VERSION }),
    );
    ExitCode::SUCCESS
}

/// Actualiza o binário: corre o `install.sh` local ou imprime o comando canónico.
pub(crate) fn upgrade() -> ExitCode {
    if let Some(script) = locate_installer() {
        run_installer(&script)
    } else {
        let mut stdout = io::stdout().lock();
        let _ignored = writeln!(stdout, "{INSTALL_CMD}");
        ExitCode::SUCCESS
    }
}

/// Gera um script de *completions* para `shell` em `stdout`.
pub(crate) fn completions(shell: &str) -> ExitCode {
    let Some(shell) = parse_shell(shell) else {
        report("shell desconhecido: use bash|zsh|fish|powershell|elvish");
        return ExitCode::FAILURE;
    };
    let mut command = Cli::command();
    let mut stdout = io::stdout().lock();
    generate(shell, &mut command, "ktu", &mut stdout);
    ExitCode::SUCCESS
}

/// Traduz o nome da shell para o tipo do `clap_complete`.
fn parse_shell(name: &str) -> Option<Shell> {
    match name {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "powershell" | "pwsh" => Some(Shell::PowerShell),
        "elvish" => Some(Shell::Elvish),
        _ => None,
    }
}

/// Procura o `install.sh` em `KTU_SOURCE_DIR` e no directório actual.
fn locate_installer() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = env::var_os("KTU_SOURCE_DIR") {
        candidates.push(PathBuf::from(dir).join("install.sh"));
    }
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd.join("install.sh"));
    }
    candidates.into_iter().find(|path| path.is_file())
}

/// Corre o instalador local.
fn run_installer(script: &Path) -> ExitCode {
    match std::process::Command::new("bash").arg(script).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => {
            report("o instalador terminou com erro");
            ExitCode::FAILURE
        }
        Err(error) => {
            report(&format!("nao foi possivel correr o instalador: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// Escreve uma mensagem de erro da borda em `stderr`.
fn report(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}

#[cfg(test)]
mod tests {
    use super::{INSTALL_CMD, parse_shell};

    #[test]
    fn install_command_is_the_canonical_one() {
        assert!(INSTALL_CMD.contains("install.sh"));
        assert!(INSTALL_CMD.starts_with("curl"));
    }

    #[test]
    fn known_shells_are_accepted() {
        assert!(parse_shell("bash").is_some());
        assert!(parse_shell("zsh").is_some());
        assert!(parse_shell("pwsh").is_some());
        assert!(parse_shell("nope").is_none());
    }
}
