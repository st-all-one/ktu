//! Ferramentas de memória (`memory_ask`, `memory_write`) sobre a porta `Memory`.
//!
//! `memory_ask` devolve **ponteiros** (id + afirmação + score); `memory_write`
//! aplica o protocolo idempotente de dedup do backend (cria/funde/rejeita).

use std::fmt::Write as _;

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::memory::{Memory, MemoryDraft, MemoryKind, MemoryWrite};
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use serde_json::Value;

/// Ferramentas de memória para o agente persistir e relembrar conhecimento.
pub struct MemoryTools<'a> {
    memory: &'a dyn Memory,
}

impl<'a> MemoryTools<'a> {
    /// Cria o conjunto sobre uma porta de memória.
    #[must_use]
    pub fn new(memory: &'a dyn Memory) -> Self {
        Self { memory }
    }

    fn ask(&self, args: &Value) -> ToolOutcome {
        let Some(query) = args.get("query").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'query' em falta");
        };
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .map_or(5, |value| usize::try_from(value).unwrap_or(5));
        match self.memory.recall(query, limit) {
            Ok(hints) if hints.is_empty() => ToolOutcome::ok("nenhuma nota de memoria relevante"),
            Ok(hints) => {
                let mut out = String::new();
                for hint in &hints {
                    let _ignored =
                        writeln!(out, "- {}: {} ({:.3})", hint.id, hint.statement, hint.score);
                }
                ToolOutcome::ok(out)
            }
            Err(error) => ToolOutcome::error(format!("memoria indisponivel: {error}")),
        }
    }

    fn write(&self, args: &Value) -> ToolOutcome {
        let statement = args
            .get("statement")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if statement.trim().is_empty() {
            return ToolOutcome::error("argumento 'statement' obrigatorio");
        }
        let kind = args
            .get("kind")
            .and_then(Value::as_str)
            .and_then(MemoryKind::parse)
            .unwrap_or(MemoryKind::Fact);
        let mut draft = MemoryDraft::new(kind, statement);
        let body = args
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        draft.body = body;
        draft.tags = string_list(args.get("tags"));
        draft.anchors = string_list(args.get("anchors"));
        match self.memory.write(&draft) {
            Ok(MemoryWrite::Created { id }) => ToolOutcome::ok(format!("memorizado ({id})")),
            Ok(MemoryWrite::Merged { id, score }) => {
                ToolOutcome::ok(format!("fundido em {id} ({score:.3})"))
            }
            Ok(MemoryWrite::Rejected { id, score }) => ToolOutcome::ok(format!(
                "duplicata forte de {id} ({score:.3}); nada gravado"
            )),
            Ok(MemoryWrite::Unchanged { id }) => ToolOutcome::ok(format!("ja memorizado ({id})")),
            Err(error) => ToolOutcome::error(format!("nao foi possivel memorizar: {error}")),
        }
    }
}

/// Lê uma lista de strings de um campo JSON (ausente → vazio).
fn string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[async_trait]
impl ToolExecutor for MemoryTools<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![
            ToolSpec {
                name: "memory_ask".to_owned(),
                description:
                    "Recupera ponteiros (id + afirmacao) da memoria do projecto.".to_owned(),
                parameters: r#"{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer"}},"required":["query"]}"#
                    .to_owned(),
            },
            ToolSpec {
                name: "memory_write".to_owned(),
                description:
                    "Memoriza uma afirmacao (uma so) para sessoes futuras do agente.".to_owned(),
                parameters: r#"{"type":"object","properties":{"kind":{"type":"string"},"statement":{"type":"string"},"body":{"type":"string"},"tags":{"type":"array","items":{"type":"string"}},"anchors":{"type":"array","items":{"type":"string"}}},"required":["statement"]}"#
                    .to_owned(),
            },
        ]
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        let args: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
        match call.name.as_str() {
            "memory_ask" => self.ask(&args),
            "memory_write" => self.write(&args),
            other => ToolOutcome::error(format!("tool desconhecida: {other}")),
        }
    }
}
