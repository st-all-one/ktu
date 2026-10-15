//! `ktu providers doctor` — diagnóstico do provider llama.cpp.
//!
//! Verifica a ligação ao `llama-server`, o modelo carregado, o ficheiro GGUF
//! (magic + metadados) e, opcionalmente, uma *sonda* funcional com tool-calling.

use std::path::Path;
use std::time::Duration;

use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Message, ToolSpec};
use ktu_core::provider::{Provider, ProviderRequest};
use serde_json::Value;

use crate::gguf::{self, GgufMeta};
use crate::llama::{LlamaConfig, LlamaProvider};

/// Resultado do diagnóstico.
#[allow(
    clippy::struct_excessive_bools,
    reason = "relatorio de diagnostico: flags independentes (reachable/ternary/probe)"
)]
#[derive(Clone, Debug)]
pub struct DoctorReport {
    /// *Base URL* verificada.
    pub base_url: String,
    /// Se o servidor respondeu.
    pub reachable: bool,
    /// Modelos anunciados em `/v1/models`.
    pub models: Vec<String>,
    /// Caminho do modelo carregado (`/props`).
    pub model_path: Option<String>,
    /// Contexto configurado (`/props`).
    pub n_ctx: Option<u64>,
    /// Metadados do GGUF, se inspeccionado.
    pub gguf: Option<GgufMeta>,
    /// Se o pack é ternary (exige o fork `PrismML`).
    pub ternary_pack: bool,
    /// Resultado da sonda funcional (`Some(true)` = OK).
    pub probe_ok: Option<bool>,
    /// Problemas detectados (vazio = saudável).
    pub problems: Vec<String>,
}

impl DoctorReport {
    /// Indica se não há problemas.
    #[must_use]
    pub fn healthy(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Se a sonda funcional deve correr.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    /// Não correr a sonda.
    Skip,
    /// Correr a sonda de tool-calling.
    Run,
}

/// Corre o diagnóstico.
pub async fn diagnose(
    base_url: &str,
    gguf_path: Option<&Path>,
    probe: Probe,
) -> Result<DoctorReport, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|error| error.to_string())?;
    let base = base_url.trim_end_matches('/');
    let mut problems = Vec::new();

    let models = match get_json(&client, &format!("{base}/v1/models")).await {
        Ok(value) => extract_models(&value),
        Err(error) => {
            problems.push(format!("servidor inacessivel: {error}"));
            Vec::new()
        }
    };
    if models.is_empty() && problems.is_empty() {
        problems.push("o servidor nao anunciou modelos".to_owned());
    }

    let props = get_json(&client, &format!("{base}/props")).await.ok();
    let model_path = props
        .as_ref()
        .and_then(|value| value.get("model_path"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let n_ctx = props.as_ref().and_then(extract_n_ctx);

    if let Some(path) = &model_path
        && !Path::new(path).exists()
    {
        problems.push(format!("ficheiro do modelo inexistente: {path}"));
    }

    let gguf = gguf_path.and_then(|path| match gguf::read_metadata(path) {
        Ok(meta) => Some(meta),
        Err(error) => {
            problems.push(error.to_string());
            None
        }
    });

    let ternary_pack = detect_ternary(model_path.as_deref(), gguf.as_ref());

    let probe_ok = if matches!(probe, Probe::Run) {
        Some(run_probe(base_url).await)
    } else {
        None
    };
    if probe_ok == Some(false) {
        problems.push(
            "sonda de tool-calling falhou (verifique o fork PrismML e o template)".to_owned(),
        );
    }

    Ok(DoctorReport {
        base_url: base_url.to_owned(),
        reachable: !models.is_empty(),
        models,
        model_path,
        n_ctx,
        gguf,
        ternary_pack,
        probe_ok,
        problems,
    })
}

async fn get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response.json().await.map_err(|error| error.to_string())
}

fn extract_models(value: &Value) -> Vec<String> {
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry.get("id").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn extract_n_ctx(value: &Value) -> Option<u64> {
    value.get("n_ctx").and_then(Value::as_u64).or_else(|| {
        value
            .pointer("/default_generation_settings/n_ctx")
            .and_then(Value::as_u64)
    })
}

fn detect_ternary(model_path: Option<&str>, gguf: Option<&GgufMeta>) -> bool {
    let from_path = model_path.is_some_and(|path| {
        let lowered = path.to_lowercase();
        ["ternary", "bonsai", "ptq", "pq2", "tq1"]
            .iter()
            .any(|marker| lowered.contains(marker))
    });
    let from_gguf = gguf
        .and_then(|meta| meta.architecture.as_deref())
        .is_some_and(|arch| {
            let lowered = arch.to_lowercase();
            lowered.contains("ternary") || lowered.contains("bonsai")
        });
    from_path || from_gguf
}

async fn run_probe(base_url: &str) -> bool {
    let config = LlamaConfig::new(base_url.to_owned(), "probe");
    let Ok(provider) = LlamaProvider::new(config) else {
        return false;
    };
    let request = ProviderRequest {
        system: "Responde a ferramentas.".to_owned(),
        messages: vec![Message::User {
            content: "Chama a tool ping com valor 1.".to_owned(),
        }],
        tools: vec![ToolSpec {
            name: "ping".to_owned(),
            description: "Devolve o valor.".to_owned(),
            parameters:
                r#"{"type":"object","properties":{"value":{"type":"integer"}},"required":["value"]}"#
                    .to_owned(),
        }],
        cancel: CancelToken::new(),
    };
    match provider.complete(&request).await {
        Ok(completion) => !completion.tool_calls.is_empty() || !completion.text.is_empty(),
        Err(_error) => false,
    }
}
