//! Auxiliares do loop: estagnação, assinaturas e emissão de log.

use crate::domain::{Completion, StopReason, ToolCall};
use crate::events;
use crate::guard::Decision;
use crate::memory::compose_hints;
use crate::ports::{Level, LogRecord};
use crate::provider::ProviderError;
use crate::session::{Event, SessionLog};
use crate::tools::ToolOutcome;

use super::Deps;

/// Estado de detecção de estagnação (sem progresso repetido).
#[derive(Debug)]
pub(super) struct Stagnation {
    last: Option<String>,
    repeats: u32,
    threshold: u32,
}

impl Stagnation {
    pub(super) fn new(threshold: u32) -> Self {
        Self {
            last: None,
            repeats: 0,
            threshold,
        }
    }

    pub(super) fn observe(&mut self, signature: String) {
        if self.last.as_deref() == Some(signature.as_str()) {
            self.repeats = self.repeats.saturating_add(1);
        } else {
            self.repeats = 0;
            self.last = Some(signature);
        }
    }

    pub(super) fn is_stalled(&self) -> bool {
        self.threshold > 0 && self.repeats >= self.threshold
    }
}

/// Resolve, como erro, as chamadas pendentes (para I2) após paragem abrupta.
pub(super) fn resolve_synthetic(log: &mut SessionLog, calls: &[ToolCall], why: &str) {
    for call in calls {
        log.push(Event::ToolResult {
            tool_call_id: call.id.clone(),
            name: call.name.clone(),
            content: format!("sem execucao: {why}"),
            is_error: true,
        });
    }
}

/// Assinatura determinística de um conjunto de chamadas (para estagnação).
pub(super) fn signature(calls: &[ToolCall]) -> String {
    let mut parts: Vec<String> = calls
        .iter()
        .map(|call| format!("{}:{}", call.name, call.arguments))
        .collect();
    parts.sort();
    parts.join("|")
}

/// Texto estável do motivo de uma decisão de guardrail.
pub(super) fn decision_reason(decision: Decision) -> String {
    match decision {
        Decision::Allow => "allow".to_owned(),
        Decision::Deny(reason) => format!("deny:{}", reason.as_str()),
        Decision::Escalate => "escalate".to_owned(),
    }
}

/// Texto estável de uma razão de término.
pub(super) fn reason_str(reason: StopReason) -> &'static str {
    match reason {
        StopReason::EndTurn => "end_turn",
        StopReason::MaxSteps => "max_steps",
        StopReason::Stalled => "stalled",
        StopReason::Cancelled => "cancelled",
        StopReason::Error => "error",
    }
}

/// Emite um evento estruturado pela porta de log.
pub(super) fn log_event(
    deps: &Deps<'_>,
    level: Level,
    event: &str,
    message: &str,
    fields: &[(&str, &str)],
) {
    deps.logger.log(&LogRecord {
        level,
        event,
        message,
        fields,
    });
}

/// Emite um erro de provider pela porta de log.
pub(super) fn log_provider_error(deps: &Deps<'_>, error: &ProviderError) {
    let message = error.to_string();
    log_event(
        deps,
        Level::Error,
        events::KERNEL_ERROR,
        "erro do provider",
        &[("erro", message.as_str())],
    );
}

/// Regista o pedido ao modelo (nº de mensagens e de tools).
pub(super) fn log_model_request(deps: &Deps<'_>, messages: usize, tools: usize) {
    let messages = messages.to_string();
    let tools = tools.to_string();
    log_event(
        deps,
        Level::Debug,
        events::MODEL_REQUEST,
        "pedido ao modelo",
        &[("messages", messages.as_str()), ("tools", tools.as_str())],
    );
}

/// Regista a saída do modelo (tamanhos e nº de tool calls).
pub(super) fn log_model_output(deps: &Deps<'_>, completion: &Completion) {
    let content = completion.text.len().to_string();
    let reasoning = completion.reasoning.len().to_string();
    let calls = completion.tool_calls.len().to_string();
    log_event(
        deps,
        Level::Debug,
        events::MODEL_OUTPUT,
        "saida do modelo",
        &[
            ("content_bytes", content.as_str()),
            ("reasoning_bytes", reasoning.as_str()),
            ("tool_calls", calls.as_str()),
        ],
    );
}

/// Regista a decisão de guardrail.
pub(super) fn log_guard_decision(deps: &Deps<'_>, tool: &str, reason: &str) {
    log_event(
        deps,
        Level::Debug,
        events::GUARD_DECISION,
        "decisao de guardrail",
        &[("tool", tool), ("reason", reason)],
    );
}

/// Regista uma chamada de tool.
pub(super) fn log_tool_call(deps: &Deps<'_>, tool: &str) {
    log_event(
        deps,
        Level::Debug,
        events::TOOL_CALL,
        "tool chamada",
        &[("tool", tool)],
    );
}

/// Regista o resultado de uma tool.
pub(super) fn log_tool_result(deps: &Deps<'_>, tool: &str, outcome: &ToolOutcome) {
    log_event(
        deps,
        Level::Debug,
        events::TOOL_RESULT,
        "resultado de tool",
        &[
            ("tool", tool),
            ("is_error", if outcome.is_error { "true" } else { "false" }),
        ],
    );
}

/// Regista um erro do kernel.
pub(super) fn log_kernel_error(deps: &Deps<'_>, message: &str) {
    log_event(deps, Level::Error, events::KERNEL_ERROR, message, &[]);
}

/// Máximo de ponteiros de memória recordados no arranque de sessão.
const RECALL_LIMIT: usize = 5;

/// Recorda memória no **início de sessão** (log ainda vazio) e devolve o *system*
/// aumentado com os ponteiros relevantes. Fora do primeiro turno devolve `None`
/// (prefixo estável → cache preservada).
pub(super) fn augment_with_memory(
    deps: &Deps<'_>,
    log: &SessionLog,
    system: &str,
    prompt: &str,
) -> Option<String> {
    let memory = deps.memory?;
    if !log.events().is_empty() {
        return None;
    }
    let hints = memory.recall(prompt, RECALL_LIMIT).ok()?;
    if hints.is_empty() {
        return None;
    }
    let count = hints.len().to_string();
    log_event(
        deps,
        Level::Info,
        events::MEMORY_RECALL,
        "memoria recordada no arranque da sessao",
        &[("hints", count.as_str())],
    );
    Some(format!("{system}\n\n{}", compose_hints(&hints)))
}
