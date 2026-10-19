//! O loop agêntico (§2.4), implementado do zero.
//!
//! Propriedades deliberadas: **sequencial** (sem paralelismo de tools no
//! início), **derivado do log** (I3) e **cancelável** (I4). O turno termina
//! sempre com uma `StopReason` (I5), nunca em silêncio.

use std::collections::BTreeSet;

mod delegation;
mod deps;
mod logging;
mod spec;
mod steering;
mod support;
mod toolcall;

pub use deps::{Capabilities, Deps, LoopConfig, Ports, Steering, Telemetry};
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
    Stagnation, augment_with_memory, fire, has_turn, is_echo, reason_str, record_session_end,
    resolve_synthetic, signature,
};

/// Turnos recentes mantidos verbatim quando o contexto é compactado.
const KEEP_TURNS: usize = 4;

/// Tools de **leitura de ficheiro** que anexam as instruções locais do directório.
pub(super) const LOCAL_INSTRUCTION_TOOLS: &[&str] = &["read_file", "edit_file"];

/// Instrução do passo final (`MaxSteps`): resumo em texto, sem tools.
const SUMMARY_INSTRUCTION: &str = "Limite de passos atingido. Responde apenas com texto: o que ficou feito, o que falta e o que recomendas. Nao uses tools.";

/// *Nudge* de eco (um retry): pede conteudo novo.
const ECHO_NUDGE: &str =
    "A tua resposta repetiu o pedido ou uma resposta anterior. Responde com conteudo novo.";

/// Nota visível quando o eco persiste — o eco **nunca** é aceite (G5).
const ECHO_NOTE: &str = "[eco: o modelo repetiu o pedido; resposta nao aceite]";

/// Corre um turno completo e devolve a razão de término (I5).
///
/// É a API de **baixo nível** (DI explícita). Quem embute o ktu usa a fachada
/// [`crate::facade::Ktu`], que faz esta composição — este caminho é interno.
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
            deferred: deps.ports.tools.deferred(),
            injected: BTreeSet::new(),
            stagnation: Stagnation::new(config.max_stagnation),
            nudge: None,
            echo_retried: false,
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

/// Natureza do passo: normal (tools activas) ou final (`MaxSteps`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Passo normal.
    Normal,
    /// Passo final: tools desactivadas e pedido de resumo.
    Final,
}

/// Estado mutável do loop, com as dependências injectadas.
struct Driver<'a, 'd> {
    log: &'a mut SessionLog,
    deps: &'a Deps<'d>,
    system: &'a str,
    config: &'a LoopConfig,
    specs: Vec<ToolSpec>,
    deferred: Vec<ToolSpec>,
    injected: BTreeSet<String>,
    stagnation: Stagnation,
    nudge: Option<String>,
    echo_retried: bool,
}

impl Driver<'_, '_> {
    async fn run(&mut self) -> StopReason {
        let max_steps = self.config.max_steps;
        for index in 0..max_steps {
            let final_step = index.saturating_add(1) >= max_steps;
            let phase = if final_step {
                Phase::Final
            } else {
                Phase::Normal
            };
            if final_step {
                self.nudge = Some(SUMMARY_INSTRUCTION.to_owned());
            }
            if let Step::Stop(reason) = self.step(index, phase).await {
                return reason;
            }
        }
        StopReason::MaxSteps
    }

    async fn step(&mut self, index: u32, phase: Phase) -> Step {
        let final_step = phase == Phase::Final;
        let deps = self.deps;
        if deps.ports.cancel.is_cancelled() {
            return Step::Stop(StopReason::Cancelled);
        }
        self.inject_steering(deps);
        let Some(messages) = self.fit_messages() else {
            log_kernel_error(deps, "orcamento de contexto excedido");
            return Step::Stop(StopReason::Error);
        };
        let tools = if final_step {
            Vec::new()
        } else {
            self.specs.clone()
        };
        let request = ProviderRequest {
            system: self.system.to_owned(),
            messages,
            tools,
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
        if completion.tool_calls.is_empty()
            && let Some(declared) = toolcall::decode(&completion.text, &self.specs)
        {
            // O texto declarado **nao** entra no log como mensagem do assistente.
            completion.tool_calls = declared;
            completion.text.clear();
        }
        if completion.tool_calls.is_empty() {
            let echoed = is_echo(&completion.text, self.log);
            if echoed && !self.echo_retried {
                self.echo_retried = true;
                self.nudge = Some(ECHO_NUDGE.to_owned());
                self.log.push(Event::Assistant {
                    content: completion.text,
                    reasoning: completion.reasoning,
                    tool_calls: Vec::new(),
                });
                return Step::Continue;
            }
            let content = if echoed {
                ECHO_NOTE.to_owned()
            } else {
                completion.text
            };
            self.log.push(Event::Assistant {
                content,
                reasoning: completion.reasoning,
                tool_calls: Vec::new(),
            });
            let reason = if final_step {
                StopReason::MaxSteps
            } else if echoed {
                StopReason::Echo
            } else {
                StopReason::EndTurn
            };
            return Step::Stop(reason);
        }
        if final_step {
            // Tools desactivadas no passo final: nunca executa, mesmo que o modelo insista.
            self.log.push(Event::Assistant {
                content: completion.text,
                reasoning: completion.reasoning,
                tool_calls: completion.tool_calls.clone(),
            });
            resolve_synthetic(deps, self.log, &completion.tool_calls, "limite de passos");
            return Step::Stop(StopReason::MaxSteps);
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
        let repaired = self.repaired(&completion.tool_calls);
        self.run_calls(&repaired, remaining).await;
        Step::Continue
    }

    /// Entrega o *steering* admitido e admite o que chegou agora (M7).
    ///
    /// A ordem e a durabilidade vivem em [`steering::inject`]: entrega primeiro o que já
    /// estava admitido (inclui uma admissão que sobreviveu a um *crash*), admite depois o
    /// que chegou — a admissão fica no log **antes** da entrega.
    fn inject_steering(&mut self, deps: &Deps<'_>) {
        steering::inject(self.log, deps);
    }

    /// Repara os argumentos das chamadas contra o schema (só quando inequívoco).
    fn repaired(&self, calls: &[ToolCall]) -> Vec<ToolCall> {
        calls
            .iter()
            .map(|call| {
                let spec = self.specs.iter().find(|spec| spec.name == call.name);
                ToolCall {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    arguments: toolcall::repair_arguments(&call.arguments, spec),
                }
            })
            .collect()
    }

    /// Projecta as mensagens e garante que cabem no orçamento, compactando uma vez
    /// se necessário. `None` quando nem compactado cabe.
    fn fit_messages(&mut self) -> Option<Vec<Message>> {
        let deps = self.deps;
        let mut messages = self.derive();
        if !deps.ports.budget.fits(self.system, &messages) && self.compact_once() {
            messages = self.derive();
        }
        deps.ports
            .budget
            .fits(self.system, &messages)
            .then_some(messages)
    }

    /// Deriva as mensagens do log e junta o *nudge* pendente (eco/passo final).
    fn derive(&self) -> Vec<Message> {
        let deps = self.deps;
        let mut messages = crate::instrument!(
            deps.telemetry.metrics,
            deps.telemetry.clock,
            "derive_messages",
            self.log.derive_messages()
        );
        if let Some(nudge) = &self.nudge {
            messages.push(Message::User {
                content: nudge.clone(),
            });
        }
        messages
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
        let outcome = self.with_local_instructions(call, outcome);
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
