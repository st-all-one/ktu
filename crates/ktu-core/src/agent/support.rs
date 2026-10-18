//! Auxiliares do loop: estagnação, assinaturas e emissão de log.

use crate::domain::{StopReason, ToolCall};
use crate::events;
use crate::guard::Decision;
use crate::memory::{MemoryDraft, MemoryKind, compose_hints};
use crate::plugin::{HookPayload, HookVerdict};
use crate::ports::Level;
use crate::session::{Event, SessionLog};
use crate::tools::ToolOutcome;

pub(super) use super::logging::{log_event, log_guard_decision, log_tool_call, log_tool_result};

use super::Deps;

/// `true` se o log já tem um turno iniciado (sessão não é nova).
pub(super) fn has_turn(log: &SessionLog) -> bool {
    log.events()
        .iter()
        .any(|event| matches!(event, Event::TurnStart { .. }))
}

/// Despacha um *hook* pela porta de intercepção e regista a evidência (§2.9).
///
/// Sem plugins (`None`) devolve permissão imediata. O registo entra **no log da
/// sessão** (evidência) e na porta de log (contrato de máquina).
pub(super) async fn fire(
    deps: &Deps<'_>,
    log: &mut SessionLog,
    payload: &mut HookPayload,
) -> HookVerdict {
    let Some(hooks) = deps.capabilities.plugins else {
        return HookVerdict::pass(0);
    };
    let verdict = hooks.dispatch(payload).await;
    let event = payload.event().as_str();
    let generation = verdict.generation.to_string();
    let allowed = if verdict.allowed { "true" } else { "false" };
    let reason = verdict.reason.as_deref().unwrap_or("");
    log_event(
        deps,
        Level::Info,
        events::PLUGIN_DISPATCH,
        "hook despachado",
        &[
            ("event", event),
            ("generation", generation.as_str()),
            ("allowed", allowed),
            ("reason", reason),
        ],
    );
    log.push(Event::PluginDispatch {
        hook: event.to_owned(),
        generation: verdict.generation,
        allowed: verdict.allowed,
        reason: verdict.reason.clone(),
    });
    verdict
}

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

/// Resolve, como erro, as chamadas pendentes (para I2) após paragem abrupta,
/// emitindo também o `tool.result` correspondente pela porta de log.
pub(super) fn resolve_synthetic(
    deps: &Deps<'_>,
    log: &mut SessionLog,
    calls: &[ToolCall],
    why: &str,
) {
    for call in calls {
        let content = format!("sem execucao: {why}");
        log_tool_result(deps, &call.name, &ToolOutcome::error(content.as_str()));
        log.push(Event::ToolResult {
            tool_call_id: call.id.clone(),
            name: call.name.clone(),
            content,
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
        StopReason::Echo => "echo",
        StopReason::Cancelled => "cancelled",
        StopReason::Error => "error",
    }
}

/// Janela de eventos recentes comparados na detecção de eco.
const ECHO_WINDOW: usize = 6;

/// Comprimento mínimo (bytes) para considerar um prefixo comum como eco.
const ECHO_MIN_CHARS: usize = 40;

/// `true` se `text` é eco do prompt do turno ou de um texto recente (G5).
///
/// Compara com o *trim* e o colapso de espaços; igualdade ou prefixo longo
/// (≥ [`ECHO_MIN_CHARS`]) conta como eco. Uma resposta legítima (conteúdo novo)
/// não é marcada (controlo negativo no teste).
pub(super) fn is_echo(text: &str, log: &SessionLog) -> bool {
    let answer = normalize(text);
    if answer.is_empty() {
        return false;
    }
    recent_texts(log)
        .iter()
        .any(|candidate| echo_of(&answer, &normalize(candidate)))
}

/// Textos recentes com que comparar (prompt do turno + últimos resultados).
fn recent_texts(log: &SessionLog) -> Vec<&str> {
    let events = log.events();
    let mut out = Vec::new();
    if let Some(Event::TurnStart { prompt }) = events
        .iter()
        .rev()
        .find(|event| matches!(event, Event::TurnStart { .. }))
    {
        out.push(prompt.as_str());
    }
    for event in events.iter().rev().take(ECHO_WINDOW) {
        match event {
            Event::Assistant { content, .. } if !content.trim().is_empty() => {
                out.push(content.as_str());
            }
            Event::ToolResult { content, .. } => out.push(content.as_str()),
            _ => {}
        }
    }
    out
}

/// Normaliza para comparação: *trim* + colapso de espaços.
fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `true` se `answer` é igual a `candidate` ou um prefixo longo (nos dois sentidos).
fn echo_of(answer: &str, candidate: &str) -> bool {
    if candidate.is_empty() {
        return false;
    }
    if answer == candidate {
        return true;
    }
    let (long, short) = if answer.len() >= candidate.len() {
        (answer, candidate)
    } else {
        (candidate, answer)
    };
    short.len() >= ECHO_MIN_CHARS && long.starts_with(short)
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
    let memory = deps.capabilities.memory?;
    if has_turn(log) {
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

/// Máximo de escalares numa afirmação (contrato do `knudge`: ≤ 120).
const MAX_STATEMENT_SCALARS: usize = 120;

/// Gatilho `session_end`: grava **automaticamente** uma nota `meta` com o desfecho do turno.
///
/// Cumpre §2.7/§2.12: a persistência **não** depende de o modelo escolher chamar a
/// tool; falhas degradam com um warning (a memória é capacidade acessória, não a
/// verdade do turno).
pub(super) fn record_session_end(deps: &Deps<'_>, log: &SessionLog, reason: StopReason) {
    let Some(memory) = deps.capabilities.memory else {
        return;
    };
    if !matches!(reason, StopReason::EndTurn) {
        return;
    }
    let Some(answer) = last_answer(log) else {
        return;
    };
    let mut draft = MemoryDraft::new(
        MemoryKind::Meta,
        truncate_scalars(answer, MAX_STATEMENT_SCALARS),
    );
    answer.clone_into(&mut draft.body);
    draft.tags = vec!["session".to_owned(), "auto".to_owned()];
    match memory.write(&draft) {
        Ok(write) => {
            let id = write.id().to_owned();
            let action = write.action();
            log_event(
                deps,
                Level::Info,
                events::MEMORY_WRITE,
                "memoria gravada no fim do turno",
                &[("action", action), ("id", id.as_str())],
            );
        }
        Err(error) => {
            let message = error.to_string();
            log_event(
                deps,
                Level::Warn,
                events::KERNEL_ERROR,
                "falha ao gravar memoria no fim do turno",
                &[("erro", message.as_str())],
            );
        }
    }
}

/// Resposta final (texto não vazio) do **turno corrente**, nunca de um anterior.
fn last_answer(log: &SessionLog) -> Option<&str> {
    let start = log
        .events()
        .iter()
        .rposition(|event| matches!(event, Event::TurnStart { .. }))
        .map_or(0, |index| index.saturating_add(1));
    log.events()
        .get(start..)?
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::Assistant { content, .. } if !content.trim().is_empty() => {
                Some(content.as_str())
            }
            _ => None,
        })
}

/// Trunca em escalares Unicode, preservando fronteiras de caractere.
fn truncate_scalars(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_owned();
    }
    trimmed.chars().take(max).collect()
}
