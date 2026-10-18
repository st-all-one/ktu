//! Adaptador HTTP da porta [`Embedder`]: `llama-server` local, OpenAI-compatible.
//!
//! O `llama-server` com o GGUF de embeddings (o **mesmo modelo** do knudge,
//! `embeddinggemma-2-Q8_0.gguf`, porta 8889) expõe `POST /v1/embeddings`.
//!
//! O adaptador é **síncrono** porque a porta é síncrona (o recall do kernel é
//! síncrono): o dreno corre fora do runtime e, no loop, a chamada bloqueia o
//! *worker* durante a consulta. O servidor é **local** e o canal é **opt-in**, por
//! isso o custo é aceite; o canal desliga sem aviso quando o servidor está fora.

use std::time::Duration;

use ktu_core::memory_config::Embeddings;
use ktu_core::ports::{EmbedError, Embedder, EmbeddingMeta};
use reqwest::blocking::Client;
use serde_json::{Value, json};

/// Tempo limite por pedido, em milissegundos.
pub const TIMEOUT_MS: u64 = 30_000;
/// Máximo de escalares do detalhe de erro devolvido pelo servidor.
const MAX_DETAIL: usize = 200;

/// Embedder HTTP para um servidor local OpenAI-compatible.
pub struct HttpEmbedder {
    client: Client,
    endpoint: String,
    meta: EmbeddingMeta,
}

impl HttpEmbedder {
    /// Monta o adaptador a partir de `memory.toml`.
    ///
    /// # Errors
    /// Devolve [`EmbedError::Invalid`] se o endpoint não for `http://` e
    /// [`EmbedError::Unavailable`] se o cliente HTTP não arrancar.
    pub fn from_config(config: &Embeddings) -> Result<Self, EmbedError> {
        let endpoint = config.endpoint.trim().to_owned();
        if !endpoint.starts_with("http://") {
            return Err(EmbedError::invalid(format!(
                "embeddings.endpoint deve comecar por http:// (recebido `{endpoint}`)"
            )));
        }
        let client = Client::builder()
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .build()
            .map_err(|error| EmbedError::unavailable(error.to_string()))?;
        Ok(Self {
            client,
            endpoint,
            meta: EmbeddingMeta::new(
                config.provider.as_str(),
                config.model.clone(),
                config.revision.clone(),
                config.dimensions,
            ),
        })
    }

    /// Endpoint em uso.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl Embedder for HttpEmbedder {
    fn meta(&self) -> &EmbeddingMeta {
        &self.meta
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let body = json!({ "model": self.meta.model, "input": texts });
        let response = self
            .client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .map_err(|error| EmbedError::unavailable(format!("{}: {error}", self.endpoint)))?;
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().unwrap_or_default();
            return Err(EmbedError::unavailable(format!(
                "{}: HTTP {status}: {}",
                self.endpoint,
                truncate(&detail)
            )));
        }
        let value: Value = response
            .json()
            .map_err(|error| EmbedError::invalid(format!("resposta nao e JSON: {error}")))?;
        parse_vectors(&value, texts.len(), self.meta.dimensions)
    }
}

/// Extrai os vetores da resposta OpenAI-compatible (`data[].embedding`), com a
/// forma legada (`embeddings: [[...]]`) como alternativa.
///
/// A dimensão é **contrato**: um vetor de tamanho diferente do configurado
/// corromperia o índice em silêncio (a similaridade trunca via `zip`) — falha
/// explícita.
fn parse_vectors(
    value: &Value,
    expected: usize,
    dimensions: usize,
) -> Result<Vec<Vec<f32>>, EmbedError> {
    let mut vectors = parse_raw(value)?;
    if vectors.len() != expected {
        return Err(EmbedError::invalid(format!(
            "provedor devolveu {} vetor(es) para {expected} texto(s)",
            vectors.len()
        )));
    }
    for vector in &mut vectors {
        if vector.len() != dimensions {
            return Err(EmbedError::invalid(format!(
                "provedor devolveu vetor de {} dimensoes, mas embeddings.dimensions={dimensions}",
                vector.len()
            )));
        }
    }
    Ok(vectors)
}

/// Lê a lista de vetores, ordenando por `index` quando o servidor o indica.
fn parse_raw(value: &Value) -> Result<Vec<Vec<f32>>, EmbedError> {
    if let Some(data) = value.get("data").and_then(Value::as_array) {
        let mut items: Vec<(u64, Vec<f32>)> = Vec::with_capacity(data.len());
        for (position, item) in data.iter().enumerate() {
            let index = item
                .get("index")
                .and_then(Value::as_u64)
                .unwrap_or_else(|| u64::try_from(position).unwrap_or(u64::MAX));
            let embedding = item
                .get("embedding")
                .and_then(Value::as_array)
                .ok_or_else(|| EmbedError::invalid("item de `data` sem `embedding`"))?;
            items.push((index, numbers(embedding)));
        }
        items.sort_by_key(|(index, _)| *index);
        return Ok(items.into_iter().map(|(_, vector)| vector).collect());
    }
    if let Some(list) = value.get("embeddings").and_then(Value::as_array) {
        return list
            .iter()
            .map(|item| {
                item.as_array()
                    .map(|values| numbers(values))
                    .ok_or_else(|| EmbedError::invalid("`embeddings` deve ser uma lista de listas"))
            })
            .collect();
    }
    Err(EmbedError::invalid(
        "resposta de embeddings sem `data`/`embeddings`",
    ))
}

/// Converte uma lista JSON de números em `f32`.
fn numbers(values: &[Value]) -> Vec<f32> {
    values
        .iter()
        .filter_map(Value::as_f64)
        .map(narrow)
        .collect()
}

/// Converte `f64` (JSON) em `f32`.
#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "vetor de embedding em f32; a perda de precisao do f64 e irrelevante"
)]
fn narrow(value: f64) -> f32 {
    value as f32
}

/// Detalhe de erro curto (nunca despeja a resposta inteira no log).
fn truncate(text: &str) -> String {
    text.chars().take(MAX_DETAIL).collect()
}

#[cfg(test)]
mod tests {
    use super::{HttpEmbedder, parse_vectors};
    use ktu_core::memory_config::{EmbeddingProvider, Embeddings};
    use ktu_core::ports::Embedder;
    use serde_json::json;

    fn config(endpoint: &str) -> Embeddings {
        Embeddings {
            provider: EmbeddingProvider::Http,
            endpoint: endpoint.to_owned(),
            dimensions: 2,
            ..Embeddings::default()
        }
    }

    #[test]
    fn parses_openai_and_legacy_shapes() -> Result<(), String> {
        let openai = json!({
            "data": [
                { "index": 1, "embedding": [1.0, 0.0] },
                { "index": 0, "embedding": [0.5, 0.5] }
            ]
        });
        let vectors = parse_vectors(&openai, 2, 2).map_err(|e| e.to_string())?;
        assert_eq!(vectors.first(), Some(&vec![0.5_f32, 0.5]));
        assert_eq!(
            vectors.get(1),
            Some(&vec![1.0_f32, 0.0]),
            "ordena por `index`"
        );

        let legacy = json!({ "embeddings": [[0.5, 0.5], [1.0, 0.0]] });
        assert_eq!(
            parse_vectors(&legacy, 2, 2)
                .map_err(|e| e.to_string())?
                .len(),
            2
        );
        Ok(())
    }

    #[test]
    fn rejects_missing_data_and_wrong_dimensions() {
        assert!(parse_vectors(&json!({ "object": "list" }), 1, 2).is_err());
        assert!(parse_vectors(&json!({ "data": [{ "embedding": [0.1] }] }), 1, 2).is_err());
        assert!(
            parse_vectors(&json!({ "data": [{ "embedding": [0.1, 0.2] }] }), 2, 2).is_err(),
            "menos vetores do que textos"
        );
        assert!(parse_vectors(&json!({ "data": [{ "embedding": [0.1, 0.2] }] }), 1, 2).is_ok());
    }

    #[test]
    fn builds_from_config_and_rejects_plain_http() -> Result<(), String> {
        let embedder = HttpEmbedder::from_config(&config("http://127.0.0.1:8889/v1/embeddings"))
            .map_err(|e| e.to_string())?;
        assert_eq!(embedder.endpoint(), "http://127.0.0.1:8889/v1/embeddings");
        assert_eq!(embedder.meta().provider, "http");
        assert_eq!(embedder.meta().model, "google/embeddinggemma-2");
        assert!(HttpEmbedder::from_config(&config("https://api.example/v1")).is_err());
        Ok(())
    }

    #[test]
    fn empty_batch_needs_no_server() -> Result<(), String> {
        let embedder = HttpEmbedder::from_config(&config("http://127.0.0.1:1/v1/embeddings"))
            .map_err(|e| e.to_string())?;
        assert_eq!(embedder.embed(&[]).map_err(|e| e.to_string())?.len(), 0);
        Ok(())
    }
}
