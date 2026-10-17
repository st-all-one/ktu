//! O loop agêntico (§2.4), implementado do zero.
//!
//! Propriedades deliberadas: **sequencial** (sem paralelismo de tools no
//! início), **derivado do log** (I3) e **cancelável** (I4). O turno termina
//! sempre com uma `StopReason` (I5), nunca em silêncio.

mod delegation;
mod deps;
mod logging;
mod spec;
mod support;

pub use deps::{Capabilities, Deps, LoopConfig, Ports, Telemetry};
pub use spec::{AgentRegistry, AgentSpec};

use crate::compact::keep_from_for_turns;
use crate::domain::{Message, StopReason, ToolCall, ToolSpec};
use crate::events;
use crate::plugin::HookPayload;
use crate::ports::Level;
use crate::provider::{ProviderError, ProviderRequest};
use crate::session::{Event, SessionLog};
use crate::tools::ToolOutcome;

use delegation::driver_specs;
use logging::{
    log_event, log_kernel_error, log_model_output, log_model_request, log_provider_error,
};
use support::{
    Stagnation, augment_with_memory, fire, has_turn, reason_str, record_session_end,
    resolve_synthetic, signature,
};

/// Turnos recentes mantidos verbatim quando o contexto é compactado.
const KEEP_TURNS: usize = 4;

/// Corre um turno completo e devolve a razão de término (I5).
pub async fn run_turn(
    log: &mut SessionLog,
    deps: &Deps<'_>,
    system: &str,
    prompt: &str,
    config: &LoopConfig,
) -> StopReason {
    if !has_turn(log) {
        fire(deps, log, &mut HookPayload::SessionStart).await;
    }
    let mut prompt = prompt.to_owned();
    let mut input = HookPayload::UserInput {
        text: prompt.clone(),
    };
    fire(deps, log, &mut input).await;
    if let HookPayload::UserInput { text } = input {
        prompt = text;
    }
    let augmented = augment_with_memory(deps, log, system, &prompt);
    let system = augmented.as_deref().unwrap_or(system);
    log.push(Event::TurnStart {
        prompt: prompt.clone(),
    });
    log_event(deps, Level::Info, events::TURN_START, "turno iniciado", &[]);
    let reason = {
        let mut driver = Driver {
            log,
            deps,
            system,
            config,
            specs: driver_specs(deps),
            stagnation: Stagnation::new(config.max_stagnation),
        };
        driver.run().await
    };
    log.push(Event::TurnEnd { reason });
    log_event(
        deps,
        Level::Info,
        events::TURN_END,
        "turno terminado",
        &[("reason", reason_str(reason))],
    );
    record_session_end(deps, log, reason);
    let mut end = HookPayload::SessionEnd {
        reason: reason_str(reason).to_owned(),
    };
    fire(deps, log, &mut end).await;
    reason
}

/// Resultado de um passo do loop.
enum Step {
    /// Continua para o passo seguinte.
    Continue,
    /// Termina o turno.
    Stop(StopReason),
}

/// Estado mutável do loop, com as dependências injectadas.
struct Driver<'a, 'd> {
    log: &'a mut SessionLog,
    deps: &'a Deps<'d>,
    system: &'a str,
    config: &'a LoopConfig,
    specs: Vec<ToolSpec>,
    stagnation: Stagnation,
}

impl Driver<'_, '_> {
    async fn run(&mut self) -> StopReason {
        let max_steps = self.config.max_steps;
        for index in 0..max_steps {
            if let Step::Stop(reason) = self.step(index).await {
                return reason;
            }
        }
        StopReason::MaxSteps
    }

    async fn step(&mut self, index: u32) -> Step {
        let deps = self.deps;
        if deps.ports.cancel.is_cancelled() {
            return Step::Stop(StopReason::Cancelled);
        }
        let Some(messages) = self.fit_messages() else {
            log_kernel_error(deps, "orcamento de contexto excedido");
            return Step::Stop(StopReason::Error);
        };
        let request = ProviderRequest {
            system: self.system.to_owned(),
            messages,
            tools: self.specs.clone(),
            cancel: deps.ports.cancel.clone(),
        };
        log_model_request(deps, request.messages.len(), request.tools.len());
        let mut hook = HookPayload::ModelRequest {
            messages: request.messages.len(),
            tools: request.tools.len(),
        };
        let verdict = fire(deps, self.log, &mut hook).await;
        if !verdict.allowed {
            log_kernel_error(deps, "hook negou o pedido ao modelo");
            return Step::Stop(StopReason::Error);
        }
        let mut completion = match deps.ports.provider.complete(&request).await {
            Ok(completion) => completion,
            Err(ProviderError::Cancelled) => return Step::Stop(StopReason::Cancelled),
            Err(error) => {
                log_provider_error(deps, &error);
                let mut hook = HookPayload::KernelError {
                    message: error.to_string(),
                };
                fire(deps, self.log, &mut hook).await;
                return Step::Stop(StopReason::Error);
            }
        };
        log_model_output(deps, &completion);
        let mut hook = HookPayload::ModelOutput {
            text: completion.text.clone(),
        };
        fire(deps, self.log, &mut hook).await;
        if let HookPayload::ModelOutput { text } = hook {
            completion.text = text;
        }
        if completion.tool_calls.is_empty() {
            self.log.push(Event::Assistant {
                content: completion.text,
                reasoning: completion.reasoning,
                tool_calls: Vec::new(),
            });
            return Step::Stop(StopReason::EndTurn);
        }
        self.stagnation.observe(signature(&completion.tool_calls));
        self.log.push(Event::Assistant {
            content: completion.text.clone(),
            reasoning: completion.reasoning.clone(),
            tool_calls: completion.tool_calls.clone(),
        });
        if self.stagnation.is_stalled() {
            resolve_synthetic(deps, self.log, &completion.tool_calls, "estagnacao");
            return Step::Stop(StopReason::Stalled);
        }
        let remaining = self.config.max_steps.saturating_sub(index);
        self.run_calls(&completion.tool_calls, remaining).await;
        Step::Continue
    }

    /// Projecta as mensagens e garante que cabem no orçamento, compactando uma vez
    /// se necessário. `None` quando nem compactado cabe.
    fn fit_messages(&mut self) -> Option<Vec<Message>> {
        let deps = self.deps;
        let mut messages = crate::instrument!(
            deps.telemetry.metrics,
            deps.telemetry.clock,
            "derive_messages",
            self.log.derive_messages()
        );
        if !deps.ports.budget.fits(self.system, &messages) && self.compact_once() {
            messages = crate::instrument!(
                deps.telemetry.metrics,
                deps.telemetry.clock,
                "derive_messages",
                self.log.derive_messages()
            );
        }
        deps.ports
            .budget
            .fits(self.system, &messages)
            .then_some(messages)
    }

    /// Compacta o contexto uma vez (mantém os últimos `KEEP_TURNS`). `true` se mudou.
    fn compact_once(&mut self) -> bool {
        let Some(from) = keep_from_for_turns(self.log.events(), KEEP_TURNS) else {
            return false;
        };
        if self.log.compaction_from() == Some(from) {
            return false;
        }
        self.log.set_compaction(from);
        let from_text = from.to_string();
        log_event(
            self.deps,
            Level::Info,
            events::SESSION_COMPACT,
            "contexto compactado",
            &[("keep_from", from_text.as_str())],
        );
        true
    }

    fn push_result(&mut self, call: &ToolCall, outcome: ToolOutcome) {
        // Ponto único de emissão de `tool.result`: garante o evento pela porta de
        // log para **todo** o desfecho (executado, negado, cap, sintético).
        support::log_tool_result(self.deps, &call.name, &outcome);
        self.log.push(Event::ToolResult {
            tool_call_id: call.id.clone(),
            name: call.name.clone(),
            content: outcome.content,
            is_error: outcome.is_error,
        });
    }
}
