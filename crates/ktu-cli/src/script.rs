//! Execução do worker embutido (`scripts/ktu-idle.sh`) — o `ktu memory drain service`.
//!
//! O corpo do worker vive **dentro do binário** (sem *supply-chain* de rede). O
//! `install` materializa-o num caminho estável (`<config>/ktu/ktu-idle.sh`), porque
//! o agendador precisa de um caminho: o worker copia-se para `~/.local/bin/ktu-idle`.
//! As restantes acções correm por `stdin` (`bash -s`), sem *staging*.
//!
//! O `stderr` do worker é **herdado** (log em directo, como manda o contrato
//! `stdout` = dados · `stderr` = logs); só o `stdout` é capturado.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Corpo do worker, embutido no binário.
pub(crate) const WORKER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/ktu-idle.sh"
));

/// Nome do ficheiro do worker materializado.
pub(crate) const WORKER_FILE: &str = "ktu-idle.sh";

/// Resultado de uma execução do worker.
pub(crate) struct WorkerRun {
    /// `stdout` do worker (dados).
    pub(crate) stdout: String,
    /// `true` se o worker terminou com sucesso.
    pub(crate) success: bool,
}

/// Materializa o worker em `dir`, devolvendo o caminho.
///
/// # Errors
/// Devolve mensagem legível se o directório ou o ficheiro não puderem ser escritos.
pub(crate) fn materialize(dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir)
        .map_err(|error| format!("nao foi possivel criar {}: {error}", dir.display()))?;
    let path = dir.join(WORKER_FILE);
    std::fs::write(&path, WORKER)
        .map_err(|error| format!("nao foi possivel escrever {}: {error}", path.display()))?;
    Ok(path)
}

/// Corre o worker **materializado** (o `install` precisa de `$SELF`).
///
/// # Errors
/// Devolve mensagem legível se o `bash` não arrancar ou o worker não terminar.
pub(crate) fn run_path(path: &Path, args: &[String]) -> Result<WorkerRun, String> {
    let mut command = Command::new("bash");
    command.arg(path);
    collect(&mut command, args, None)
}

/// Corre o worker embutido por `stdin` (`bash -s -- <args>`), sem *staging*.
///
/// # Errors
/// Devolve mensagem legível se o `bash` não arrancar ou o worker não terminar.
pub(crate) fn run_embedded(args: &[String]) -> Result<WorkerRun, String> {
    let mut command = Command::new("bash");
    command.arg("-s").arg("--");
    collect(&mut command, args, Some(WORKER))
}

/// Corre o comando, herdando `stderr` e capturando `stdout` (opcionalmente alimenta o `stdin`).
fn collect(
    command: &mut Command,
    args: &[String],
    input: Option<&str>,
) -> Result<WorkerRun, String> {
    command
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = command
        .spawn()
        .map_err(|error| format!("nao foi possivel executar o worker: {error}"))?;
    if let Some(body) = input
        && let Some(mut stdin) = child.stdin.take()
    {
        stdin
            .write_all(body.as_bytes())
            .map_err(|error| format!("nao foi possivel alimentar o worker: {error}"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("o worker nao terminou: {error}"))?;
    Ok(WorkerRun {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        success: output.status.success(),
    })
}
