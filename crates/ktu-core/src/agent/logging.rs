//! Emissão de eventos do loop pela porta [`Logger`](crate::ports::Logger).
//!
//! Centraliza os ids estáveis de [`crate::events`] e os campos estruturados: o
//! contrato de máquina do log fica num só lugar e o `support` guarda só a
//! lógica de turno.

use crate::domain::Completion;
use crate::events;
use crate::ports::{Level, LogRecord};
use crate::provider::ProviderError;
use crate::tools::ToolOutcome;

use super::Deps;

/// Emite um evento estruturado pela porta de log.
pub(super) fn log_event(
    deps: &Deps<'_>,
    level: Level,
    event: &str,
    message: &str,
    fields: &[(&str, &str)],
) {
    deps.telemetry.logger.log(&LogRecord {
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
