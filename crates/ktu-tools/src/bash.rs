//! Tool `bash`: executa um comando com **timeout**, **cap de saída** e
//! **análise de argv** (fail-closed).
//!
//! A análise de `argv` é uma primeira barreira determinística: comandos
//! destrutivos conhecidos são recusados antes de executar. O `cwd` é o *root*
//! da sessão e o processo é morto ao expirar (`kill_on_drop`).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::taint::wrap_untrusted;
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use crate::truncate;

/// Limite por omissão da saída (bytes).
const DEFAULT_MAX_OUTPUT: usize = 32_768;

/// Tempo limite por omissão (segundos).
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Padrões destrutivos recusados pela análise de `argv` (substring, *fail-closed*).
const DESTRUCTIVE: &[(&str, &str)] = &[
    ("rm -rf", "remocao recursiva"),
    ("rm -fr", "remocao recursiva"),
    ("mkfs", "formatacao de dispositivo"),
    ("shutdown", "desligar a maquina"),
    ("reboot", "reiniciar a maquina"),
    ("sudo ", "elevacao de privilegios"),
    (":(){", "fork bomb"),
    ("> /dev/sd", "escrita directa em dispositivo"),
    ("dd if=/dev/", "escrita directa em dispositivo"),
];

/// Prefixos de sistema permitidos (leitura/execução de binários e dispositivos).
///
/// Tudo o resto que seja absoluto e fique **fora** do *root* é recusado: o
/// sandbox não deve escrever (nem ler) fora da sessão (ver `risk_01ua5ei2`).
const SYSTEM_PREFIXES: &[&str] = &[
    "/bin/",
    "/sbin/",
    "/usr/",
    "/lib/",
    "/lib64/",
    "/dev/null",
    "/dev/stdout",
    "/dev/stderr",
    "/dev/fd/",
    "/proc/",
    "/sys/",
];

/// Tool de execução de comandos.
pub struct BashTool {
    root: PathBuf,
    timeout: Duration,
    max_output: usize,
}

impl BashTool {
    /// Cria a tool com o *root* da sessão.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_output: DEFAULT_MAX_OUTPUT,
        }
    }

    /// Define o tempo limite (construtor).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Define o cap de saída em bytes (construtor).
    #[must_use]
    pub fn with_max_output(mut self, max_output: usize) -> Self {
        self.max_output = max_output;
        self
    }

    async fn run(&self, args: &Value) -> ToolOutcome {
        let Some(script) = args.get("command").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'command' em falta");
        };
        if let Some(reason) = analyze(script) {
            return ToolOutcome::error(format!("comando recusado: {reason}"));
        }
        if let Some(reason) = analyze_paths(script, &self.root) {
            return ToolOutcome::error(format!("comando recusado: {reason}"));
        }
        let mut command = Command::new("bash");
        command
            .arg("-c")
            .arg(script)
            .current_dir(&self.root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => return ToolOutcome::error(format!("nao foi possivel executar: {error}")),
        };
        let output = match timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(error)) => {
                return ToolOutcome::error(format!("falha ao executar: {error}"));
            }
            Err(_elapsed) => {
                return ToolOutcome::error(format!(
                    "timeout: comando excedeu {}s",
                    self.timeout.as_secs()
                ));
            }
        };
        let status = output.status.code().unwrap_or(-1);
        let stdout = spill_stream(&self.root, "stdout", &output.stdout, self.max_output);
        let stderr = spill_stream(&self.root, "stderr", &output.stderr, self.max_output);
        let body = format!("exit: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}");
        let result = ToolOutcome::ok(wrap_untrusted("bash", &body));
        if status == 0 {
            result
        } else {
            ToolOutcome::error(body)
        }
    }
}

#[async_trait]
impl ToolExecutor for BashTool {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "bash".to_owned(),
            description: "Executa um comando de shell no root da sessao (com timeout).".to_owned(),
            parameters:
                r#"{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}"#
                    .to_owned(),
        }]
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        let args: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
        self.run(&args).await
    }
}

/// Analisa o comando e devolve o motivo de recusa, se destrutivo.
#[must_use]
pub fn analyze(script: &str) -> Option<&'static str> {
    let lowered = script.to_lowercase();
    DESTRUCTIVE
        .iter()
        .find(|(pattern, _reason)| lowered.contains(*pattern))
        .map(|(_pattern, reason)| *reason)
}

/// Analisa os **caminhos** do comando: recusa `~`/`$HOME`, `..` e absolutos
/// fora do *root* (exceto localizações de sistema read-only). Fail-closed.
#[must_use]
pub fn analyze_paths(script: &str, root: &Path) -> Option<&'static str> {
    if script.contains('~') || script.contains("$HOME") {
        return Some("expansao de home fora do root");
    }
    let roots = root_prefixes(root);
    for token in tokens(script) {
        if token == ".." || token.starts_with("../") {
            return Some("path relativo escapa do root");
        }
        if !token.starts_with('/') {
            continue;
        }
        if roots
            .iter()
            .any(|prefix| token.starts_with(prefix.as_str()))
        {
            continue;
        }
        if SYSTEM_PREFIXES
            .iter()
            .any(|prefix| token.starts_with(prefix))
        {
            continue;
        }
        return Some("caminho absoluto fora do root");
    }
    None
}

/// Prefixos do *root* (canónico e original), com separador final.
fn root_prefixes(root: &Path) -> Vec<String> {
    let mut prefixes = Vec::new();
    for candidate in [root.canonicalize().ok(), Some(root.to_path_buf())] {
        let Some(path) = candidate else {
            continue;
        };
        let mut text = path.to_string_lossy().into_owned();
        if !text.ends_with('/') {
            text.push('/');
        }
        if !prefixes.contains(&text) {
            prefixes.push(text);
        }
    }
    prefixes
}

/// Divide o comando em tokens (espaços e meta-caracteres de shell).
fn tokens(script: &str) -> Vec<&str> {
    script
        .split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    ';' | '|'
                        | '&'
                        | '('
                        | ')'
                        | '<'
                        | '>'
                        | '"'
                        | '\''
                        | '='
                        | '`'
                        | ','
                        | ':'
                        | '{'
                        | '}'
                )
        })
        .filter(|token| !token.is_empty())
        .collect()
}

/// Trunca uma *stream* (linhas + bytes) e, se cortada, guarda o completo em
/// `.ktu/out/` com um ponteiro legível para o modelo.
fn spill_stream(root: &Path, label: &str, bytes: &[u8], max_bytes: usize) -> String {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let capped = truncate::truncate(&text, truncate::MAX_LINES, max_bytes);
    if !capped.is_cut {
        return capped.text;
    }
    let shown = capped.text.lines().count();
    let total = capped.total_lines;
    match truncate::spill(root, label, &text) {
        Some(path) => format!(
            "{}\n[truncado: {shown} de {total} linhas; completo em {path}]",
            capped.text
        ),
        None => format!("{}\n[truncado: {shown} de {total} linhas]", capped.text),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{analyze, analyze_paths};

    #[test]
    fn denies_destructive_commands() {
        assert!(analyze("rm -rf /").is_some());
        assert!(analyze("sudo apt install x").is_some());
        assert!(analyze(":(){ :|:& };:").is_some());
        assert!(analyze("ls -la").is_none());
        assert!(analyze("cargo test").is_none());
    }

    #[test]
    fn denies_paths_outside_the_root() {
        let root = Path::new("/tmp/ktu-root");
        assert!(analyze_paths("cat ~/x", root).is_some());
        assert!(analyze_paths("echo $HOME/y", root).is_some());
        assert!(analyze_paths("cd ..", root).is_some());
        assert!(analyze_paths("cat ../segredo", root).is_some());
        assert!(analyze_paths("cat /home/one/x", root).is_some());
        assert!(analyze_paths("cp x /tmp/outro/y", root).is_some());
        assert!(analyze_paths("cat /etc/shadow", root).is_some());
    }

    #[test]
    fn allows_commands_inside_the_root_and_system_tools() {
        let root = Path::new("/tmp/ktu-root");
        assert!(analyze_paths("ls -la", root).is_none());
        assert!(analyze_paths("cat exploration/x.md", root).is_none());
        assert!(analyze_paths("echo hi > out.txt", root).is_none());
        assert!(analyze_paths("ls 2>/dev/null", root).is_none());
        assert!(analyze_paths("cat /tmp/ktu-root/exploration/x.md", root).is_none());
        assert!(analyze_paths("/usr/bin/env python3 run.py", root).is_none());
        assert!(analyze_paths("python3 -c 'print(1)'", root).is_none());
    }
}
