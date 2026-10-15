//! O loop agêntico (§2.4), implementado do zero.
//!
//! Propriedades deliberadas: **sequencial** (sem paralelismo de tools no
//! início), **derivado do log** (I3) e **cancelável** (I4). O turno termina
//! sempre com uma `StopReason` (I5), nunca em silêncio.

mod support;

use crate::budget::BudgetGate;
use crate::cancel::CancelToken;
use crate::compact::keep_from_for_turns;
use crate::domain::{Message, StopReason, ToolCall, ToolSpec};
use crate::events;
use crate::guard::{self, AgentId, Decision, Fact, Guard};
use crate::memory::Memory;
use crate::ports::{Clock, Level, Logger, Metrics};
use crate::provider::{Provider, ProviderError, ProviderRequest};
use crate::session::{Event, SessionLog};
use crate::tools::{ToolExecutor, ToolOutcome};

use support::{
    Stagnation, augment_with_memory, decision_reason, log_event, log_guard_decision,
    log_kernel_error, log_model_output, log_model_request, log_provider_error, log_tool_call,
    log_tool_result, reason_str, record_session_end, resolve_synthetic, signature,
};

/// Configuração do loop.
#[derive(Debug, Clone, Copy)]
pub struct LoopConfig {
    /// Tecto de passos por turno.
    pub max_steps: u32,
    /// Repetições da mesma assinatura de tools antes de considerar estagnação.
    pub max_stagnation: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_steps: 24,
            max_stagnation: 3,
        }
    }
}

/// Dependências do loop (injecção explícita — DI).
pub struct Deps<'d> {
    /// Provider do modelo.
    pub provider: &'d dyn Provider,
    /// Executor de tools (thread T2).
    pub tools: &'d dyn ToolExecutor,
    /// Guardrail estrutural.
    pub guard: &'d dyn Guard,
    /// Dono único do tecto de contexto (I14).
    pub budget: &'d BudgetGate,
    /// Porta de log.
    pub logger: &'d dyn Logger,
    /// Porta de métricas.
    pub metrics: &'d dyn Metrics,
    /// Porta de tempo.
    pub clock: &'d dyn Clock,
    /// Sinal de cancelamento (I4).
    pub cancel: &'d CancelToken,
    /// Porta de memória persistente (recall no início de sessão; `None` desliga).
    pub memory: Option<&'d dyn Memory>,
}

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
    let augmented = augment_with_memory(deps, log, system, prompt);
    let system = augmented.as_deref().unwrap_or(system);
    log.push(Event::TurnStart {
        prompt: prompt.to_owned(),
    });
    log_event(deps, Level::Info, events::TURN_START, "turno iniciado", &[]);
    let reason = {
        let mut driver = Driver {
            log,
            deps,
            system,
            config,
            specs: deps.tools.specs(),
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
        if deps.cancel.is_cancelled() {
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
            cancel: deps.cancel.clone(),
        };
        log_model_request(deps, request.messages.len(), request.tools.len());
        let completion = match deps.provider.complete(&request).await {
            Ok(completion) => completion,
            Err(ProviderError::Cancelled) => return Step::Stop(StopReason::Cancelled),
            Err(error) => {
                log_provider_error(deps, &error);
                return Step::Stop(StopReason::Error);
            }
        };
        log_model_output(deps, &completion);
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
            resolve_synthetic(self.log, &completion.tool_calls, "estagnacao");
            return Step::Stop(StopReason::Stalled);
        }
        let remaining = self.config.max_steps.saturating_sub(index);
        for call in &completion.tool_calls {
            self.run_call(call, remaining).await;
        }
        Step::Continue
    }

    async fn run_call(&mut self, call: &ToolCall, remaining_steps: u32) {
        let deps = self.deps;
        if deps.cancel.is_cancelled() {
            self.push_result(call, ToolOutcome::error("cancelado pelo utilizador"));
            return;
        }
        let fact = Fact {
            agent: AgentId::Primary,
            tool: call.name.clone(),
            requested_path: guard::path_argument(&call.arguments),
            remaining_steps,
            depth: 0,
        };
        let decision = deps.guard.evaluate(&fact);
        let reason = decision_reason(decision);
        self.log.push(Event::GuardDecision {
            tool: call.name.clone(),
            allowed: matches!(decision, Decision::Allow),
            reason: reason.clone(),
        });
        log_guard_decision(deps, &call.name, &reason);
        match decision {
            Decision::Allow => {
                log_tool_call(deps, &call.name);
                let outcome = deps.tools.execute(call).await;
                log_tool_result(deps, &call.name, &outcome);
                self.push_result(call, outcome);
            }
            Decision::Deny(reason) => {
                self.push_result(
                    call,
                    ToolOutcome::error(format!("negado: {}", reason.as_str())),
                );
            }
            Decision::Escalate => {
                self.push_result(
                    call,
                    ToolOutcome::error("requer aprovacao (indisponivel nesta fase)"),
                );
            }
        }
    }

    /// Projecta as mensagens e garante que cabem no orçamento, compactando uma vez
    /// se necessário. `None` quando nem compactado cabe.
    fn fit_messages(&mut self) -> Option<Vec<Message>> {
        let deps = self.deps;
        let mut messages = crate::instrument!(
            deps.metrics,
            deps.clock,
            "derive_messages",
            self.log.derive_messages()
        );
        if !deps.budget.fits(self.system, &messages) && self.compact_once() {
            messages = crate::instrument!(
                deps.metrics,
                deps.clock,
                "derive_messages",
                self.log.derive_messages()
            );
        }
        deps.budget.fits(self.system, &messages).then_some(messages)
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
        self.log.push(Event::ToolResult {
            tool_call_id: call.id.clone(),
            name: call.name.clone(),
            content: outcome.content,
            is_error: outcome.is_error,
        });
    }
}
