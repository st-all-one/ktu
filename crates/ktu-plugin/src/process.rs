//! Plugin externo: JSON-lines sobre stdio, com *timeout* (fail-closed em `gate`).

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use ktu_core::plugin::{HookEvent, HookPayload, Mode, Plugin, PluginVerdict};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use crate::manifest::Manifest;

/// Plugin implementado por um processo externo (qualquer linguagem).
pub struct ProcessPlugin {
    name: String,
    program: String,
    args: Vec<String>,
    dir: PathBuf,
    events: BTreeSet<HookEvent>,
    mode: Mode,
    timeout: Duration,
}

impl ProcessPlugin {
    /// Constrói o plugin a partir do manifesto e do directório.
    ///
    /// # Errors
    /// Devolve mensagem se o *entrypoint*, os eventos ou o modo forem inválidos.
    pub fn new(manifest: &Manifest, dir: PathBuf) -> Result<Self, String> {
        let mut parts = manifest.entrypoint.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| "entrypoint vazio".to_owned())?
            .to_owned();
        let args: Vec<String> = parts.map(str::to_owned).collect();
        Ok(Self {
            name: manifest.name.clone(),
            program,
            args,
            dir,
            events: manifest.events()?,
            mode: manifest.mode()?,
            timeout: Duration::from_millis(manifest.timeout_ms),
        })
    }

    /// Corre uma chamada ao processo: escreve o pedido, lê o veredicto.
    async fn call(&self, payload: &HookPayload) -> Result<PluginVerdict, String> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .current_dir(&self.dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|error| format!("spawn {}: {error}", self.name))?;
        let request = serde_json::json!({
            "id": 1,
            "event": payload.event().as_str(),
            "payload": payload,
        });
        let mut line = serde_json::to_string(&request).map_err(|error| error.to_string())?;
        line.push('\n');
        let mut stdin = child.stdin.take().ok_or_else(|| "sem stdin".to_owned())?;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|error| error.to_string())?;
        stdin.shutdown().await.map_err(|error| error.to_string())?;
        drop(stdin);
        let stdout = child.stdout.take().ok_or_else(|| "sem stdout".to_owned())?;
        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        let read = timeout(self.timeout, reader.read_line(&mut response))
            .await
            .map_err(|_| "timeout".to_owned())?;
        read.map_err(|error| error.to_string())?;
        let _killed = child.kill().await;
        serde_json::from_str::<PluginVerdict>(response.trim())
            .map_err(|error| format!("resposta invalida: {error}"))
    }
}

#[async_trait]
impl Plugin for ProcessPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn subscribes(&self, event: HookEvent) -> bool {
        self.events.contains(&event)
    }

    async fn handle(&self, payload: &HookPayload) -> PluginVerdict {
        match self.call(payload).await {
            Ok(verdict) => verdict,
            // Gate em erro/timeout é fail-closed (I11); observe/transform passam.
            Err(_) => match self.mode {
                Mode::Gate => PluginVerdict::Deny {
                    reason: "plugin_timeout".to_owned(),
                },
                Mode::Observe | Mode::Transform => PluginVerdict::Pass,
            },
        }
    }
}
