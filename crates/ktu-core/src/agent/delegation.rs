//! Suporte à delegação `subagent` no loop (§2.6): fan-out, guard e efeitos.

use futures::future::join_all;

use super::{Driver, support};
use crate::domain::{ToolCall, ToolSpec};
use crate::guard::{self, AgentId, Decision, DenyReason, Fact};
use crate::plugin::HookPayload;
use crate::session::Event;
use crate::subagent;
use crate::tools::ToolOutcome;

/// Efeito pendente de uma chamada de um passo.
enum Pending {
    /// Corre a chamada como subagente (em fan-out com as restantes).
    Delegate,
    /// Executa a chamada como tool normal, já com a decisão do guardrail.
    Effect(Decision),
    /// Resultado já pronto (ex.: recusada por cap), sem execução.
    Ready(ToolOutcome),
}

/// Tools oferecidas ao modelo: as do executor, mais `subagent` quando há host.
pub(super) fn driver_specs(deps: &super::Deps<'_>) -> Vec<ToolSpec> {
    let mut specs = deps.ports.tools.specs();
    if deps.capabilities.subagents.is_some() {
        specs.push(subagent::spec());
    }
    specs
}

/// Executa uma delegação `subagent` pela porta do host e capa o relatório.
pub(super) async fn run_subagent(
    deps: &super::Deps<'_>,
    call: &ToolCall,
    report_chars: usize,
) -> ToolOutcome {
    let Some(host) = deps.capabilities.subagents else {
        return ToolOutcome::error("subagente indisponivel nesta sessao");
    };
    let request = match subagent::parse_request(&call.arguments) {
        Ok(request) => request,
        Err(error) => return ToolOutcome::error(error),
    };
    match host.run(request).await {
        Ok(report) => ToolOutcome::ok(subagent::cap_report(&report.text, report_chars)),
        Err(error) => ToolOutcome::error(format!("subagente falhou: {error}")),
    }
}

impl Driver<'_, '_> {
    /// Corre as chamadas de um passo: subagentes em **fan-out** (paralelo, com
    /// *join*) e as restantes **sequenciais**; os resultados entram no log na
    /// ordem original das chamadas (determinismo).
    pub(super) async fn run_calls(&mut self, calls: &[ToolCall], remaining_steps: u32) {
        let deps = self.deps;
        let mut decisions = Vec::with_capacity(calls.len());
        for call in calls {
            decisions.push(self.evaluate_guard(call, remaining_steps).await);
        }
        let max_subagents =
            usize::try_from(self.config.max_subagents_per_turn).unwrap_or(usize::MAX);
        let mut pending = Vec::with_capacity(calls.len());
        let mut delegated = 0usize;
        for (call, decision) in calls.iter().zip(&decisions) {
            if call.name == subagent::TOOL && matches!(decision, Decision::Allow) {
                if delegated < max_subagents {
                    delegated = delegated.saturating_add(1);
                    support::log_tool_call(deps, &call.name);
                    pending.push(Pending::Delegate);
                } else {
                    pending.push(Pending::Ready(ToolOutcome::error(
                        "limite de subagentes por turno atingido",
                    )));
                }
            } else {
                pending.push(Pending::Effect(*decision));
            }
        }
        let delegated_calls: Vec<&ToolCall> = calls
            .iter()
            .zip(&pending)
            .filter(|(_, item)| matches!(item, Pending::Delegate))
            .map(|(call, _)| call)
            .collect();
        let reports = join_all(
            delegated_calls
                .iter()
                .map(|call| run_subagent(deps, call, self.config.subagent_report_chars)),
        )
        .await;
        let mut reports = reports.into_iter();
        let mut outcomes = Vec::with_capacity(calls.len());
        for (call, item) in calls.iter().zip(pending) {
            let outcome = match item {
                Pending::Ready(outcome) => outcome,
                Pending::Delegate => reports
                    .next()
                    .unwrap_or_else(|| ToolOutcome::error("sem relatorio de subagente")),
                Pending::Effect(decision) => self.execute_effect(call, decision).await,
            };
            outcomes.push(outcome);
        }
        for (call, outcome) in calls.iter().zip(outcomes) {
            self.push_result(call, outcome);
        }
    }

    /// Avalia o guardrail de uma chamada e regista a decisão (evidência).
    ///
    /// Ordem fixa (I11): guardrail estrutural **primeiro**; só se for `Allow` os
    /// *gates* de plugin são consultados — nunca podem transformar `Deny` em `Allow`.
    async fn evaluate_guard(&mut self, call: &ToolCall, remaining_steps: u32) -> Decision {
        let deps = self.deps;
        let fact = Fact {
            agent: AgentId::Primary,
            tool: call.name.clone(),
            requested_path: guard::path_argument(&call.arguments),
            remaining_steps,
            depth: 0,
        };
        let mut decision = deps.ports.guard.evaluate(&fact);
        let mut detail: Option<String> = None;
        if matches!(decision, Decision::Allow) {
            let mut payload = HookPayload::ActionBefore {
                tool: call.name.clone(),
                arguments: call.arguments.clone(),
            };
            let verdict = support::fire(deps, self.log, &mut payload).await;
            if !verdict.allowed {
                decision = Decision::Deny(DenyReason::Plugin);
                detail = verdict.reason;
            }
        }
        let reason = detail.map_or_else(
            || support::decision_reason(decision),
            |message| format!("deny:plugin:{message}"),
        );
        self.log.push(Event::GuardDecision {
            tool: call.name.clone(),
            allowed: matches!(decision, Decision::Allow),
            reason: reason.clone(),
        });
        support::log_guard_decision(deps, &call.name, &reason);
        let mut payload = HookPayload::GuardDecision {
            tool: call.name.clone(),
            decision: reason,
        };
        support::fire(deps, self.log, &mut payload).await;
        decision
    }

    /// Executa o efeito de uma chamada já decidida (nunca para `subagent`).
    async fn execute_effect(&mut self, call: &ToolCall, decision: Decision) -> ToolOutcome {
        let deps = self.deps;
        if deps.ports.cancel.is_cancelled() {
            return ToolOutcome::error("cancelado pelo utilizador");
        }
        match decision {
            Decision::Allow => {
                support::log_tool_call(deps, &call.name);
                let mut outcome = deps.ports.tools.execute(call).await;
                let mut payload = HookPayload::ActionAfter {
                    tool: call.name.clone(),
                    result: outcome.content.clone(),
                    is_error: outcome.is_error,
                };
                support::fire(deps, self.log, &mut payload).await;
                if let HookPayload::ActionAfter { result, .. } = payload {
                    outcome.content = result;
                }
                outcome
            }
            Decision::Deny(reason) => ToolOutcome::error(format!("negado: {}", reason.as_str())),
            Decision::Escalate => ToolOutcome::error("requer aprovacao (indisponivel nesta fase)"),
        }
    }
}
