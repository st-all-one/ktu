//! Acções da tool `plan`: parse, aplicação e render.

use std::fmt::Write as _;

use serde_json::Value;

use super::{Plan, PlanState, TOOL};
use crate::domain::ToolSpec;

/// Acção pedida pela tool `plan`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanAction {
    /// Acrescenta um estágio.
    AddStage {
        /// Id do estágio.
        id: String,
        /// Título.
        title: String,
    },
    /// Acrescenta uma tarefa a um estágio.
    AddTask {
        /// Estágio alvo.
        stage: String,
        /// Id da tarefa.
        id: String,
        /// Objectivo.
        objective: String,
        /// Dependências.
        depends_on: Vec<String>,
        /// Critério de pronto.
        done_when: String,
    },
    /// Muda o estado de uma tarefa.
    SetTaskState {
        /// Id da tarefa.
        id: String,
        /// Novo estado.
        state: PlanState,
    },
    /// Lista o plano corrente.
    List,
}

/// Especificação da tool `plan`.
#[must_use]
pub fn spec() -> ToolSpec {
    ToolSpec {
        name: TOOL.to_owned(),
        description:
            "Mantem o plano estruturado (estagios e tarefas) em .ktu/plan/<sessao>.json. \
             Acoes: stage_add{id,title}, task_add{stage,id,objective,depends_on,done_when}, \
             task_state{id,state}, list."
                .to_owned(),
        parameters: r#"{"type":"object","properties":{"action":{"type":"string"},"id":{"type":"string"},"title":{"type":"string"},"stage":{"type":"string"},"objective":{"type":"string"},"depends_on":{"type":"array","items":{"type":"string"}},"done_when":{"type":"string"},"state":{"type":"string"}},"required":["action"]}"#
            .to_owned(),
    }
}

/// Interpreta os argumentos JSON da tool `plan`.
///
/// # Errors
/// Devolve mensagem accionável em argumentos em falta ou malformados.
pub fn parse_action(arguments: &str) -> Result<PlanAction, String> {
    let value: Value = serde_json::from_str(arguments)
        .map_err(|error| format!("argumentos invalidos: {error}"))?;
    let action = text(&value, "action").ok_or_else(|| "argumento 'action' em falta".to_owned())?;
    match action.as_str() {
        "stage_add" => Ok(PlanAction::AddStage {
            id: required(&value, "id")?,
            title: text(&value, "title").unwrap_or_default(),
        }),
        "task_add" => Ok(PlanAction::AddTask {
            stage: text(&value, "stage").unwrap_or_else(|| "main".to_owned()),
            id: required(&value, "id")?,
            objective: required(&value, "objective")?,
            depends_on: string_list(&value, "depends_on"),
            done_when: text(&value, "done_when").unwrap_or_default(),
        }),
        "task_state" => {
            let id = required(&value, "id")?;
            let state_text = required(&value, "state")?;
            let state = PlanState::parse(&state_text)
                .ok_or_else(|| format!("estado desconhecido: {state_text}"))?;
            Ok(PlanAction::SetTaskState { id, state })
        }
        "list" => Ok(PlanAction::List),
        other => Err(format!("acao desconhecida: {other}")),
    }
}

/// Aplica uma acção ao plano e devolve um resumo textual.
///
/// # Errors
/// Estágio/tarefa inexistente ou id duplicado.
pub fn apply(plan: &mut Plan, action: PlanAction) -> Result<String, String> {
    match action {
        PlanAction::AddStage { id, title } => {
            if plan.stages.iter().any(|stage| stage.id == id) {
                return Err(format!("estagio ja existe: {id}"));
            }
            plan.stages.push(super::PlanStage {
                id: id.clone(),
                title,
                tasks: Vec::new(),
            });
            Ok(format!("estagio {id} adicionado"))
        }
        PlanAction::AddTask {
            stage,
            id,
            objective,
            depends_on,
            done_when,
        } => {
            let target = plan
                .stages
                .iter_mut()
                .find(|candidate| candidate.id == stage)
                .ok_or_else(|| format!("estagio desconhecido: {stage} (cria com stage_add)"))?;
            if target.tasks.iter().any(|task| task.id == id) {
                return Err(format!("tarefa ja existe: {id}"));
            }
            target.tasks.push(super::PlanTask {
                id: id.clone(),
                objective,
                state: PlanState::Todo,
                depends_on,
                done_when,
            });
            Ok(format!("tarefa {id} adicionada ao estagio {stage}"))
        }
        PlanAction::SetTaskState { id, state } => {
            for stage in &mut plan.stages {
                if let Some(task) = stage.tasks.iter_mut().find(|task| task.id == id) {
                    task.state = state;
                    return Ok(format!("tarefa {id} -> {}", state.as_str()));
                }
            }
            Err(format!("tarefa desconhecida: {id}"))
        }
        PlanAction::List => Ok(render(plan)),
    }
}

/// Renderiza o plano como texto legível.
#[must_use]
pub fn render(plan: &Plan) -> String {
    let mut out = format!("plano da sessao {}\n", plan.session);
    for stage in &plan.stages {
        let _ignored = writeln!(out, "[{}] {}", stage.id, stage.title);
        for task in &stage.tasks {
            let _ignored = writeln!(
                out,
                "  - {} ({}) {}",
                task.id,
                task.state.as_str(),
                task.objective
            );
        }
    }
    out
}

fn text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|item| !item.trim().is_empty())
        .map(str::to_owned)
}

fn required(value: &Value, key: &str) -> Result<String, String> {
    text(value, key).ok_or_else(|| format!("argumento '{key}' em falta"))
}

fn string_list(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
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
