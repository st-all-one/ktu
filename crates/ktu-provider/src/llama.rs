//! Provider built-in: llama.cpp via `llama-server` (OpenAI-compatible).
//!
//! `POST /v1/chat/completions` com `stream: true`; o corpo SSE é framado por
//! [`crate::sse`] e acumulado por [`crate::stream`]. O cancelamento é observado
//! **a meio da stream** (I4). Erros transitórios são repetidos com *backoff*.

use std::time::Duration;

use async_trait::async_trait;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::Completion;
use ktu_core::provider::{Provider, ProviderError, ProviderRequest};
use reqwest::Response;
use serde_json::Value;
use tokio::time::sleep;

use crate::fallback;
use crate::sampling::{ReasoningEffort, Sampling};
use crate::sse::SseFramer;
use crate::stream::CompletionAccumulator;
use crate::wire::{self, Flavor};

/// URL por omissão do `llama-server` local.
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";

/// Configuração do provider.
#[derive(Clone, Debug)]
pub struct LlamaConfig {
    /// *Base URL* (ex.: `http://127.0.0.1:8080`).
    pub base_url: String,
    /// Nome/alias do modelo (informativo; o servidor usa o modelo carregado).
    pub model: String,
    /// Amostragem (thinking mode por omissão).
    pub sampling: Sampling,
    /// Esforço de raciocínio.
    pub effort: ReasoningEffort,
    /// Tentativas por passo (com *backoff*) para erros transitórios.
    pub max_attempts: u32,
    /// Espera entre tentativas.
    pub retry_backoff: Duration,
    /// Tempo limite do pedido.
    pub timeout: Duration,
    /// Chave do gateway remoto (`Authorization: Bearer`), quando exigida.
    pub api_key: Option<String>,
    /// Id de sessão/afinidade enviado nos cabeçalhos (`opencode-go`).
    pub session: Option<String>,
    /// Cabeçalho que leva o id de sessão (ex.: `x-opencode-session`).
    pub session_header: Option<String>,
    /// Cabeçalhos extra que também levam o id de sessão (afinidade/cache).
    pub affinity_headers: Vec<String>,
    /// Dialeto do corpo: extensões do `llama.cpp` ou só campos `OpenAI`.
    pub flavor: Flavor,
}

impl LlamaConfig {
    /// Configuração por omissão para um dado `base_url`/`model`.
    #[must_use]
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            sampling: Sampling::ornith(),
            effort: ReasoningEffort::default(),
            max_attempts: 4,
            retry_backoff: Duration::from_secs(1),
            timeout: Duration::from_secs(600),
            api_key: None,
            session: None,
            session_header: None,
            affinity_headers: Vec::new(),
            flavor: Flavor::Llama,
        }
    }
}

/// Provider llama.cpp.
pub struct LlamaProvider {
    client: reqwest::Client,
    config: LlamaConfig,
}

impl LlamaProvider {
    /// Cria o provider (cliente HTTP sem TLS, para `llama-server` local).
    pub fn new(config: LlamaConfig) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|error| ProviderError::Unavailable(error.to_string()))?;
        Ok(Self { client, config })
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/v1/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }

    async fn attempt(
        &self,
        endpoint: &str,
        body: &Value,
        cancel: &CancelToken,
    ) -> Result<Completion, ProviderError> {
        let mut builder = self.client.post(endpoint).json(body);
        if let Some(key) = self.config.api_key.as_deref() {
            builder = builder.bearer_auth(key);
        }
        if let Some(session) = self.config.session.as_deref() {
            if let Some(header) = self.config.session_header.as_deref() {
                builder = builder.header(header, session);
            }
            for header in &self.config.affinity_headers {
                builder = builder.header(header, session);
            }
        }
        let response = builder
            .send()
            .await
            .map_err(|error| ProviderError::Unavailable(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            let message = format!("HTTP {status}: {detail}");
            let retryable =
                status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS;
            return Err(if retryable {
                ProviderError::Unavailable(message)
            } else {
                ProviderError::Invalid(message)
            });
        }
        let completion = read_stream(response, cancel).await?;
        Ok(apply_fallbacks(completion))
    }
}

#[async_trait]
impl Provider for LlamaProvider {
    async fn complete(&self, request: &ProviderRequest) -> Result<Completion, ProviderError> {
        let body = wire::request_body(
            &self.config.model,
            request,
            self.config.sampling,
            self.config.effort,
            self.config.flavor,
        );
        let endpoint = self.endpoint();
        let attempts = self.config.max_attempts.max(1);
        let mut last_error = None;
        for attempt in 0..attempts {
            if attempt > 0 {
                sleep(self.config.retry_backoff).await;
            }
            match self.attempt(&endpoint, &body, &request.cancel).await {
                Ok(completion) => return Ok(completion),
                Err(ProviderError::Cancelled) => return Err(ProviderError::Cancelled),
                Err(error) if is_retryable(&error) => last_error = Some(error),
                Err(error) => return Err(error),
            }
        }
        Err(last_error.unwrap_or_else(|| ProviderError::Unavailable("sem tentativas".to_owned())))
    }
}

/// Lê o corpo SSE e acumula uma [`Completion`] (cancelável a meio).
async fn read_stream(
    mut response: Response,
    cancel: &CancelToken,
) -> Result<Completion, ProviderError> {
    let mut framer = SseFramer::new();
    let mut accumulator = CompletionAccumulator::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| ProviderError::Unavailable(error.to_string()))?
    {
        for payload in framer.push(&chunk) {
            if cancel.is_cancelled() {
                return Err(ProviderError::Cancelled);
            }
            accumulator.apply(&payload)?;
        }
    }
    Ok(accumulator.finish())
}

/// Aplica os *fallbacks* de pensamento inline e tool-calls em texto.
fn apply_fallbacks(mut completion: Completion) -> Completion {
    if completion.reasoning.is_empty() && fallback::has_thinking(&completion.text) {
        let (reasoning, content) = fallback::split_thinking(&completion.text);
        completion.reasoning = reasoning;
        completion.text = content;
    }
    if completion.tool_calls.is_empty() {
        let calls = fallback::parse_text_tool_calls(&completion.text);
        if !calls.is_empty() {
            completion.tool_calls = calls;
            completion.text = fallback::strip_text_tool_calls(&completion.text);
        }
    }
    completion
}

fn is_retryable(error: &ProviderError) -> bool {
    matches!(error, ProviderError::Unavailable(_))
}
