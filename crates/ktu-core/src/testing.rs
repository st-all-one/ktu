//! Dublês determinísticos para testes e **gates**: providers falsos e tools falsas.
//!
//! - [`FakeProvider`] faz *replay* determinístico de um script de completions (I6).
//! - [`EchoProvider`] é o provider de demonstração (sem modelo): pede `read_file` do
//!   caminho mencionado no prompt e ecoa o resultado. É o mesmo em todos os caminhos
//!   (`ktu run --provider mock`, REPL e crate) — é o que torna a prova de M4 byte-a-byte.
//! - [`parse_script`] lê um guião JSONL (uma [`Completion`] por linha) para o
//!   `--provider scripted` do CLI: uma cadeia longa **sem** modelo, determinística.
//! - [`FakeTools`] devolve respostas pré-definidas e regista as chamadas.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;

use crate::domain::{Completion, Message, ToolCall, ToolCallId, ToolSpec};
use crate::provider::{Provider, ProviderError, ProviderRequest};
use crate::tools::{ToolExecutor, ToolOutcome};

/// Provider falso: replay determinístico de um script de completions.
#[derive(Debug)]
pub struct FakeProvider {
    script: Vec<Completion>,
    cursor: Mutex<usize>,
    tool_counts: Mutex<Vec<usize>>,
}

impl FakeProvider {
    /// Cria um provider que devolve o script pela ordem dada.
    #[must_use]
    pub fn new(script: Vec<Completion>) -> Self {
        Self {
            script,
            cursor: Mutex::new(0),
            tool_counts: Mutex::new(Vec::new()),
        }
    }

    /// Número de tools oferecidas em cada pedido (por ordem).
    #[must_use]
    pub fn tool_counts(&self) -> Vec<usize> {
        self.tool_counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn cursor(&self) -> MutexGuard<'_, usize> {
        self.cursor.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl Provider for FakeProvider {
    async fn complete(&self, request: &ProviderRequest) -> Result<Completion, ProviderError> {
        self.tool_counts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(request.tools.len());
        let mut cursor = self.cursor();
        let index = *cursor;
        *cursor = index.saturating_add(1);
        self.script
            .get(index)
            .cloned()
            .ok_or_else(|| ProviderError::Unavailable(format!("script esgotado no passo {index}")))
    }
}

/// Provider de demonstração (sem modelo): pede `read_file` e ecoa o resultado.
///
/// Determinístico e sem rede: o caminho mencionado no prompt é lido e o conteúdo
/// é devolvido como resposta final. Existe para que o loop funcione **antes** de
/// haver modelo (e para que os gates provem a mecânica sem depender de inferência).
#[derive(Debug, Default)]
pub struct EchoProvider;

impl EchoProvider {
    /// Cria o provider (sem estado).
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

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

/// Extrai o último token que "parece" um caminho (tem `.` ou `/`).
fn extract_path(prompt: &str) -> String {
    prompt
        .split_whitespace()
        .rev()
        .find(|token| token.contains('.') || token.contains('/'))
        .unwrap_or("AGENTS.md")
        .to_owned()
}

/// Um passo do guião (formato JSONL do `--provider scripted`).
///
/// Formato próprio do guião (não é o contrato do domínio): `{"text": "…",
/// "reasoning": "…", "tool_calls": [{"id": "c1", "name": "read_file",
/// "arguments": "{…}"}]}`. Campos ausentes ficam vazios.
#[derive(Debug, serde::Deserialize)]
struct ScriptStep {
    #[serde(default)]
    text: String,
    #[serde(default)]
    reasoning: String,
    #[serde(default)]
    tool_calls: Vec<ScriptCall>,
}

/// Uma chamada de tool no guião.
#[derive(Debug, serde::Deserialize)]
struct ScriptCall {
    id: String,
    name: String,
    #[serde(default)]
    arguments: String,
}

impl ScriptStep {
    /// Converte o passo do guião numa [`Completion`] do domínio.
    fn completion(self) -> Completion {
        Completion {
            text: self.text,
            reasoning: self.reasoning,
            tool_calls: self
                .tool_calls
                .into_iter()
                .map(|call| ToolCall {
                    id: ToolCallId::new(&call.id),
                    name: call.name,
                    arguments: call.arguments,
                })
                .collect(),
        }
    }
}

/// Lê um guião JSONL: uma [`Completion`] por linha (linhas vazias ignoradas).
///
/// É a forma **pura** de uma cadeia longa sem modelo: o CLI lê o ficheiro (borda)
/// e entrega as completions ao [`FakeProvider`] (replay determinístico, I6).
///
/// # Errors
/// Devolve mensagem legível com o número da linha se o JSON for inválido.
pub fn parse_script(text: &str) -> Result<Vec<Completion>, String> {
    let mut script = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let step: ScriptStep = serde_json::from_str(line).map_err(|error| {
            format!("linha {}: JSON invalido: {error}", index.saturating_add(1))
        })?;
        script.push(step.completion());
    }
    if script.is_empty() {
        return Err("guiao vazio (uma Completion JSON por linha)".to_owned());
    }
    Ok(script)
}

/// Tools falsas: respostas pré-definidas e registo das chamadas.
#[derive(Debug)]
pub struct FakeTools {
    specs: Vec<ToolSpec>,
    deferred: Vec<ToolSpec>,
    responses: Mutex<VecDeque<ToolOutcome>>,
    calls: Mutex<Vec<ToolCall>>,
}

impl FakeTools {
    /// Cria tools com especificações e respostas (consumidas por ordem).
    #[must_use]
    pub fn new(specs: Vec<ToolSpec>, responses: Vec<ToolOutcome>) -> Self {
        Self {
            specs,
            deferred: Vec::new(),
            responses: Mutex::new(responses.into()),
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Declara tools **diferidas** (fora do catálogo até `tool_search`).
    #[must_use]
    pub fn with_deferred(mut self, deferred: Vec<ToolSpec>) -> Self {
        self.deferred = deferred;
        self
    }

    /// Devolve as chamadas registadas, por ordem.
    #[must_use]
    pub fn recorded_calls(&self) -> Vec<ToolCall> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl ToolExecutor for FakeTools {
    fn specs(&self) -> Vec<ToolSpec> {
        self.specs.clone()
    }

    fn deferred(&self) -> Vec<ToolSpec> {
        self.deferred.clone()
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(call.clone());
        self.responses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .unwrap_or_else(|| ToolOutcome::error("sem resposta preparada"))
    }
}

#[cfg(test)]
mod tests {
    use tokio::runtime::Builder;

    use super::{EchoProvider, parse_script};
    use crate::cancel::CancelToken;
    use crate::domain::{Message, ToolCallId};
    use crate::provider::{Provider, ProviderRequest};

    fn request(messages: Vec<Message>) -> ProviderRequest {
        ProviderRequest {
            system: String::new(),
            messages,
            tools: Vec::new(),
            cancel: CancelToken::new(),
        }
    }

    #[test]
    fn echo_provider_reads_the_path_in_the_prompt_and_echoes_the_result()
    -> Result<(), Box<dyn std::error::Error>> {
        let runtime = Builder::new_current_thread().build()?;
        let provider = EchoProvider::new();
        let first = runtime.block_on(provider.complete(&request(vec![Message::User {
            content: "le src/main.rs".to_owned(),
        }])))?;
        assert_eq!(first.tool_calls.len(), 1);
        assert_eq!(
            first.tool_calls.first().map(|call| call.name.as_str()),
            Some("read_file")
        );
        assert!(
            first
                .tool_calls
                .first()
                .is_some_and(|call| call.arguments.contains("src/main.rs"))
        );

        let second = runtime.block_on(provider.complete(&request(vec![Message::Tool {
            tool_call_id: ToolCallId::new("f1-0"),
            content: "conteudo".to_owned(),
            is_error: false,
        }])))?;
        assert_eq!(second.text, "Resultado:\nconteudo");
        assert_eq!(second.tool_calls, Vec::new());
        Ok(())
    }

    #[test]
    fn parse_script_reads_one_completion_per_line_and_reports_errors()
    -> Result<(), Box<dyn std::error::Error>> {
        let guiao = "# comentario\n{\"text\":\"ola\"}\n\n{\"text\":\"\",\"tool_calls\":[{\"id\":\"c1\",\"name\":\"read_file\",\"arguments\":\"{}\"}]}\n";
        let script = parse_script(guiao)?;
        assert_eq!(script.len(), 2);
        assert_eq!(script.first().map(|step| step.text.as_str()), Some("ola"));
        assert_eq!(script.get(1).map(|step| step.tool_calls.len()), Some(1));

        assert!(parse_script("   \n").is_err());
        assert!(parse_script("{nao json}\n").is_err());
        Ok(())
    }
}
