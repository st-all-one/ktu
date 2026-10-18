//! Delegação para subagentes read-only (`explorer`) — §2.6.
//!
//! O núcleo define a **porta** [`SubagentHost`]; a borda (CLI/serve) implementa a
//! corrida de uma sessão-filha de primeira classe. O loop só conhece o pedido e o
//! relatório final capado — o corpo da filha fica no log em disco.

use async_trait::async_trait;

use crate::domain::ToolSpec;

/// Nome canónico da tool de delegação.
pub const TOOL: &str = "subagent";

/// Pedido de delegação (o que o modelo emitiu, já parseado).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentRequest {
    /// Agente alvo (ex.: `explorer`).
    pub agent: String,
    /// Descrição curta da tarefa (para o log).
    pub description: String,
    /// Instrução concreta dada à sessão-filha.
    pub prompt: String,
    /// Sessão-filha a retomar; `None` cria uma nova com contexto fresco.
    pub session_id: Option<String>,
}

/// Relatório final de um subagente (texto capado; a sessão fica em disco).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubagentReport {
    /// Identificador da sessão-filha (retomável).
    pub session_id: String,
    /// Texto final produzido pela filha.
    pub text: String,
}

/// Porta de execução de um subagente. Implementada na borda.
#[async_trait]
pub trait SubagentHost: Send + Sync {
    /// Corre (ou retoma) a sessão-filha e devolve o relatório final.
    async fn run(&self, request: SubagentRequest) -> Result<SubagentReport, String>;

    /// Nomes dos subagentes disponíveis (para o catálogo do modelo).
    ///
    /// Por omissão, vazio: a descrição da tool não enumera nomes.
    fn agents(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Especificação da tool `subagent` oferecida ao modelo.
#[must_use]
pub fn spec() -> ToolSpec {
    spec_with(&[])
}

/// Especificação da tool `subagent` com os nomes de agentes disponíveis.
///
/// A lista de nomes entra na **descrição** (o modelo não adivinha): é derivada
/// dos agentes registados, em ordem determinística.
#[must_use]
pub fn spec_with(agents: &[String]) -> ToolSpec {
    let description = if agents.is_empty() {
        "Delega uma exploracao a um subagente e devolve o relatorio final.".to_owned()
    } else {
        format!(
            "Delega uma exploracao a um subagente (disponiveis: {}) e devolve o relatorio final.",
            agents.join(", ")
        )
    };
    ToolSpec {
        name: TOOL.to_owned(),
        description,
        parameters: r#"{"type":"object","properties":{"agent":{"type":"string"},"description":{"type":"string"},"prompt":{"type":"string"},"session_id":{"type":"string"}},"required":["agent","prompt"]}"#
            .to_owned(),
    }
}

/// Interpreta os argumentos JSON da tool `subagent`.
///
/// Argumentos obrigatórios em falta ou malformados devolvem uma mensagem
/// accionável (o loop transforma-a num `ToolOutcome` de erro).
pub fn parse_request(arguments: &str) -> Result<SubagentRequest, String> {
    let value: serde_json::Value = serde_json::from_str(arguments)
        .map_err(|error| format!("argumentos invalidos: {error}"))?;
    let agent = value
        .get("agent")
        .and_then(serde_json::Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| "argumento 'agent' em falta".to_owned())?;
    let prompt = value
        .get("prompt")
        .and_then(serde_json::Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| "argumento 'prompt' em falta".to_owned())?;
    let description = value
        .get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let session_id = value
        .get("session_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    Ok(SubagentRequest {
        agent: agent.to_owned(),
        description,
        prompt: prompt.to_owned(),
        session_id,
    })
}

/// Capa o relatório em escalares Unicode, preservando fronteiras de caractere.
#[must_use]
pub fn cap_report(text: &str, max_chars: usize) -> String {
    if max_chars == 0 || text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut capped: String = text.chars().take(max_chars).collect();
    capped.push_str("\n…(relatorio truncado)");
    capped
}

#[cfg(test)]
mod tests {
    use super::{cap_report, parse_request, spec, spec_with};

    #[test]
    fn spec_lists_the_available_agents() {
        assert!(!spec().description.contains("disponiveis"));
        let with = spec_with(&["explorer".to_owned(), "rev".to_owned()]);
        assert!(with.description.contains("disponiveis: explorer, rev"));
        assert_eq!(with.name, "subagent");
        assert_eq!(spec_with(&[]).description, spec().description);
    }

    #[test]
    fn parses_minimal_request() {
        let parsed = parse_request(r#"{"agent":"explorer","prompt":"mapeia"}"#);
        assert!(parsed.is_ok());
        if let Ok(request) = parsed {
            assert_eq!(request.agent, "explorer");
            assert_eq!(request.prompt, "mapeia");
            assert!(request.session_id.is_none());
        }
    }

    #[test]
    fn rejects_missing_prompt_and_bad_json() {
        assert!(parse_request(r#"{"agent":"explorer"}"#).is_err());
        assert!(parse_request("nao json").is_err());
    }

    #[test]
    fn caps_long_reports() {
        let text = "a".repeat(20);
        let capped = cap_report(&text, 5);
        assert!(capped.starts_with("aaaaa"));
        assert!(capped.contains("truncado"));
        assert!(capped.matches('a').count() < 20);
        assert_eq!(cap_report("curto", 5), "curto");
    }
}
