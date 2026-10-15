//! Tool `bash`: executa um comando com **timeout**, **cap de saída** e
//! **análise de argv** (fail-closed).
//!
//! A análise de `argv` é uma primeira barreira determinística: comandos
//! destrutivos conhecidos são recusados antes de executar. O `cwd` é o *root*
//! da sessão e o processo é morto ao expirar (`kill_on_drop`).

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::taint::wrap_untrusted;
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

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
        let stdout = cap(&output.stdout, self.max_output);
        let stderr = cap(&output.stderr, self.max_output);
        let body = format!(
            "exit: {status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            stdout = String::from_utf8_lossy(&stdout),
            stderr = String::from_utf8_lossy(&stderr),
        );
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

fn cap(bytes: &[u8], limit: usize) -> Vec<u8> {
    let length = limit.min(bytes.len());
    bytes.get(..length).unwrap_or(&[]).to_vec()
}

#[cfg(test)]
mod tests {
    use super::analyze;

    #[test]
    fn denies_destructive_commands() {
        assert!(analyze("rm -rf /").is_some());
        assert!(analyze("sudo apt install x").is_some());
        assert!(analyze(":(){ :|:& };:").is_some());
        assert!(analyze("ls -la").is_none());
        assert!(analyze("cargo test").is_none());
    }
}
