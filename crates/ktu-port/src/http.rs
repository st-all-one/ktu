//! Transporte `Http` — cliente da porta sobre HTTP/1.1 + HTTP/2 (SSE).
//!
//! Fala os mesmos endpoints que o servidor (`ktu serve`); nenhum front-end tem
//! lógica própria. A leitura de SSE é um parser de linhas puro e testável.

use futures::StreamExt;
use ktu_core::plan::{Approval, Mode};
use serde::de::DeserializeOwned;
use serde_json::json;
use tokio::sync::mpsc;

use crate::port::{EventStream, Port};
use crate::protocol::{Health, PortError, ServerMsg, SessionInfo, SessionState};

/// Resposta de `POST .../turn`.
#[derive(serde::Deserialize)]
struct TurnAck {
    turn_id: u64,
}

/// Cliente HTTP da porta única.
pub struct HttpPort {
    base: String,
    client: reqwest::Client,
}

impl HttpPort {
    /// Cria o cliente apontado à base (ex.: `http://127.0.0.1:7717`).
    #[must_use]
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    async fn decode<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, PortError> {
        let response = request
            .send()
            .await
            .map_err(|error| PortError::Transport(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PortError::Protocol(format!("http {status}: {body}")));
        }
        response
            .json::<T>()
            .await
            .map_err(|error| PortError::Protocol(error.to_string()))
    }

    async fn discard(&self, request: reqwest::RequestBuilder) -> Result<(), PortError> {
        let response = request
            .send()
            .await
            .map_err(|error| PortError::Transport(error.to_string()))?;
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(PortError::Protocol(format!("http {status}: {body}")))
        }
    }
}

#[async_trait::async_trait]
impl Port for HttpPort {
    async fn health(&self) -> Result<Health, PortError> {
        self.decode(self.client.get(self.url("/v1/health"))).await
    }

    async fn create_session(&self, session_id: &str) -> Result<SessionInfo, PortError> {
        self.decode(
            self.client
                .post(self.url("/v1/session"))
                .json(&json!({ "session_id": session_id })),
        )
        .await
    }

    async fn turn(&self, session_id: &str, prompt: &str) -> Result<u64, PortError> {
        let ack: TurnAck = self
            .decode(
                self.client
                    .post(self.url(&format!("/v1/session/{session_id}/turn")))
                    .json(&json!({ "prompt": prompt })),
            )
            .await?;
        Ok(ack.turn_id)
    }

    async fn cancel(&self, session_id: &str) -> Result<(), PortError> {
        self.discard(
            self.client
                .post(self.url(&format!("/v1/session/{session_id}/cancel"))),
        )
        .await
    }

    async fn set_mode(
        &self,
        session_id: &str,
        mode: Mode,
        approval: Approval,
    ) -> Result<(), PortError> {
        self.discard(
            self.client
                .post(self.url(&format!("/v1/session/{session_id}/mode")))
                .json(&json!({ "mode": mode, "approval": approval })),
        )
        .await
    }

    async fn state(&self, session_id: &str) -> Result<SessionState, PortError> {
        self.decode(
            self.client
                .get(self.url(&format!("/v1/session/{session_id}"))),
        )
        .await
    }

    fn subscribe(&self, session_id: &str) -> EventStream {
        let (tx, rx) = mpsc::unbounded_channel();
        let client = self.client.clone();
        let url = self.url(&format!("/v1/session/{session_id}/events"));
        tokio::spawn(async move {
            let Ok(response) = client.get(url).send().await else {
                let _ignored = tx.send(ServerMsg::Error {
                    message: "stream SSE indisponivel".to_owned(),
                });
                return;
            };
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            while let Some(chunk) = stream.next().await {
                let Ok(bytes) = chunk else { break };
                buffer.push_str(&String::from_utf8_lossy(&bytes));
                while let Some(newline) = buffer.find('\n') {
                    let line: String = buffer.drain(..=newline).collect();
                    if let Some(message) = parse_sse_line(&line)
                        && tx.send(message).is_err()
                    {
                        return;
                    }
                }
            }
        });
        rx
    }
}

/// Converte uma linha SSE (`data: <json>`) numa mensagem do servidor.
///
/// Ignora comentários (`:`) e linhas que não são eventos válidos.
#[must_use]
pub fn parse_sse_line(line: &str) -> Option<ServerMsg> {
    let data = line.trim_end().strip_prefix("data:")?;
    serde_json::from_str::<ServerMsg>(data.trim()).ok()
}

#[cfg(test)]
mod tests {
    use ktu_core::session::Event;

    use super::parse_sse_line;
    use crate::protocol::ServerMsg;

    #[test]
    fn parses_a_data_line() {
        let message = ServerMsg::Log {
            event: Event::TurnStart {
                prompt: "ola".to_owned(),
            },
        };
        let line = format!(
            "data: {}\n",
            serde_json::to_string(&message).unwrap_or_default()
        );
        assert_eq!(parse_sse_line(&line), Some(message));
    }

    #[test]
    fn ignores_comments_and_garbage() {
        assert_eq!(parse_sse_line(": keep-alive\n"), None);
        assert_eq!(parse_sse_line("event: log\n"), None);
        assert_eq!(parse_sse_line("data: nao-e-json\n"), None);
    }
}
