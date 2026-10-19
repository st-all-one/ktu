//! Suporte à delegação `subagent` no loop (§2.6): fan-out, guard e efeitos.

use futures::future::join_all;

use super::{Driver, LOCAL_INSTRUCTION_TOOLS, logging, support};
use crate::domain::{ToolCall, ToolSpec};
use crate::events;
use crate::guard::{self, AgentId, Decision, DenyReason, Fact};
use crate::plugin::HookPayload;
use crate::ports::Level;
use crate::session::Event;
use crate::subagent;
use crate::tool_search;
use crate::tools::ToolOutcome;

/// Efeito pendente de uma chamada de um passo.
enum Pending {
    /// Corre a chamada como subagente (em fan-out com as restantes).
    Delegate,
    /// Executa a chamada como tool normal, já com a decisão e o remédio do guardrail.
    Effect {
        /// Decisão do guardrail.
        decision: Decision,
        /// Remédio acionável (o que fazer a seguir).
        remedy: Option<String>,
    },
    /// Resultado já pronto (ex.: recusada por cap), sem execução.
    Ready(ToolOutcome),
}

/// Tools oferecidas ao modelo: as declaradas pelo executor, mais `subagent` quando
/// há host e `tool_search` quando existem tools **diferidas**.
pub(super) fn driver_specs(deps: &super::Deps<'_>) -> Vec<ToolSpec> {
    let mut specs = deps.ports.tools.specs();
    if let Some(host) = deps.capabilities.subagents {
        specs.push(subagent::spec_with(&host.agents()));
    }
    if !deps.ports.tools.deferred().is_empty() {
        specs.push(tool_search::spec());
    }
    specs
}

impl Driver<'_, '_> {
    /// Anexa as **instruções locais** (`AGENTS.md` mais próximo) ao resultado de
    /// uma leitura de ficheiro: uma vez por turno e por fonte, para não repetir
    /// contexto. Fica no resultado da tool — logo, no log (*model-visible ⟺ logged*).
    pub(super) fn with_local_instructions(
        &mut self,
        call: &ToolCall,
        outcome: ToolOutcome,
    ) -> ToolOutcome {
        if outcome.is_error || !LOCAL_INSTRUCTION_TOOLS.contains(&call.name.as_str()) {
            return outcome;
        }
        let deps = self.deps;
        let Some(instructions) = deps.capabilities.instructions else {
            return outcome;
        };
        let Some(path) = guard::path_argument(&call.arguments) else {
            return outcome;
        };
        let Some(local) = instructions.nearest(&path) else {
            return outcome;
        };
        if !self.injected.insert(local.source.clone()) {
            return outcome;
        }
        logging::log_event(
            deps,
            Level::Info,
            events::CONTEXT_LOCAL,
            "instrucoes locais anexadas",
            &[("source", local.source.as_str())],
        );
        ToolOutcome {
            content: format!(
                "{}\n\n[instrucoes locais: {}]\n{}",
                outcome.content, local.source, local.text
            ),
            is_error: false,
        }
    }

    /// Corre `tool_search`: ranqueia as tools diferidas (BM25) e **promove** as
    /// melhores ao catálogo — ficam declaradas no **próximo** pedido e o
    /// carregamento é registado no log (`tool.loaded`), nunca em silêncio.
    pub(super) fn search_deferred(&mut self, call: &ToolCall) -> ToolOutcome {
        let deps = self.deps;
        let query = match tool_search::parse_query(&call.arguments) {
            Ok(query) => query,
            Err(error) => return ToolOutcome::error(error),
        };
        let found: Vec<ToolSpec> = tool_search::rank(&query, &self.deferred, tool_search::TOP_K)
            .into_iter()
            .filter_map(|index| self.deferred.get(index).cloned())
            .collect();
        for spec in &found {
            if !self.specs.iter().any(|known| known.name == spec.name) {
                self.specs.push(spec.clone());
            }
            logging::log_event(
                deps,
                Level::Info,
                events::TOOL_LOADED,
                "tool diferida carregada",
                &[("tool", spec.name.as_str())],
            );
        }
        let promoted: Vec<String> = found.iter().map(|spec| spec.name.clone()).collect();
        self.deferred.retain(|spec| !promoted.contains(&spec.name));
        ToolOutcome::ok(tool_search::render(&found, self.deferred.len()))
    }
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
        for (call, (decision, remedy)) in calls.iter().zip(&decisions) {
            if call.name == tool_search::TOOL && matches!(decision, Decision::Allow) {
                support::log_tool_call(deps, &call.name);
                pending.push(Pending::Ready(self.search_deferred(call)));
            } else if call.name == subagent::TOOL && matches!(decision, Decision::Allow) {
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
                pending.push(Pending::Effect {
                    decision: *decision,
                    remedy: remedy.clone(),
                });
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
                Pending::Effect { decision, remedy } => {
                    self.execute_effect(call, decision, remedy.as_deref()).await
                }
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
    async fn evaluate_guard(
        &mut self,
        call: &ToolCall,
        remaining_steps: u32,
    ) -> (Decision, Option<String>) {
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
        let remedy = if matches!(decision, Decision::Allow) {
            None
        } else if matches!(decision, Decision::Deny(DenyReason::Plugin)) {
            Some(DenyReason::Plugin.remedy().to_owned())
        } else {
            deps.ports.guard.remedy(&fact)
        };
        (decision, remedy)
    }

    /// Executa o efeito de uma chamada já decidida (nunca para `subagent`).
    async fn execute_effect(
        &mut self,
        call: &ToolCall,
        decision: Decision,
        remedy: Option<&str>,
    ) -> ToolOutcome {
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
            Decision::Deny(reason) => {
                // O remedio **da regra** junta-se ao estrutural, nunca o substitui:
                // quem nega diz o que fazer, e o motivo diz onde procurar a politica.
                let structural = reason.remedy();
                let message = match remedy {
                    Some(own) if own != structural => {
                        format!("negado: {} — {own} ({structural})", reason.as_str())
                    }
                    _ => format!("negado: {} — {structural}", reason.as_str()),
                };
                ToolOutcome::error(message)
            }
            Decision::Escalate => ToolOutcome::error(format!(
                "requer aprovacao: {}",
                remedy.unwrap_or("requer aprovacao humana")
            )),
        }
    }
}
