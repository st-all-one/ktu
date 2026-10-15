//! Provider de demonstração do F1 (sem modelo real).
//!
//! Pede `read_file` do path mencionado no prompt e, depois de ver o resultado,
//! devolve-o como resposta final. É substituído pelo provider real (llama.cpp)
//! em F2 — existe para que o slice vertical **funcione** antes de haver modelo.

use async_trait::async_trait;
use ktu_core::domain::{Completion, Message, ToolCall, ToolCallId};
use ktu_core::provider::{Provider, ProviderError, ProviderRequest};

/// Provider de demonstração: pede um `read_file` e ecoa o resultado.
#[derive(Debug, Default)]
pub(crate) struct EchoProvider;

#[async_trait]
impl Provider for EchoProvider {
    async fn complete(&self, request: &ProviderRequest) -> Result<Completion, ProviderError> {
        if let Some(Message::Tool { content, .. }) = request.messages.last() {
            return Ok(Completion {
                text: format!("Resultado:\n{content}"),
                reasoning: String::new(),
                tool_calls: Vec::new(),
            });
        }
        let prompt = request
            .messages
            .iter()
            .rev()
            .find_map(|message| match message {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
            .unwrap_or("AGENTS.md");
        let path = extract_path(prompt);
        let arguments = serde_json::json!({ "path": path }).to_string();
        Ok(Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![ToolCall {
                id: ToolCallId::new("f1-0"),
                name: "read_file".to_owned(),
                arguments,
            }],
        })
    }
}

/// Extrai o último token que “parece” um caminho (tem `.` ou `/`).
fn extract_path(prompt: &str) -> String {
    prompt
        .split_whitespace()
        .rev()
        .find(|token| token.contains('.') || token.contains('/'))
        .unwrap_or("AGENTS.md")
        .to_owned()
}
