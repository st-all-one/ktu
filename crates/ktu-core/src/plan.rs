//! Modo plano (§2.13): estado de sessão, artefacto persistente e tool `plan`.
//!
//! O read-only do modo plano é **estrutural** (conjunto de tools do `AgentSpec`);
//! a transição `plan → build` exige aprovação (I15) e é sempre logada. O artefacto
//! vive em `.ktu/plan/<sessão>.json`, gravado de forma atómica (`temp→fsync→rename`).

mod actions;
mod host;
mod store;

use serde::{Deserialize, Serialize};

pub use actions::{PlanAction, apply, parse_action, render, spec};
pub use host::FsPlanHost;
pub use store::{load, path, save};

/// Nome canónico da tool de planeamento.
pub const TOOL: &str = "plan";

/// Modo de operação da sessão.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Execução normal (todas as tools).
    #[default]
    Build,
    /// Planeamento read-only (só leitura + `plan` + `subagent`).
    Plan,
}

impl Mode {
    /// Texto estável do modo.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Plan => "plan",
        }
    }

    /// Interpreta o texto do modo.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "build" => Some(Self::Build),
            "plan" => Some(Self::Plan),
            _ => None,
        }
    }

    /// Aplica uma transição de modo com a regra de aprovação (I15).
    ///
    /// Restringir (`build → plan`) é sempre permitido; escalar (`plan → build`)
    /// exige aprovação explícita do utilizador.
    ///
    /// # Errors
    /// Recusa `plan → build` sem aprovação, nomeando o comando.
    pub fn transition(current: Self, target: Self, approval: Approval) -> Result<Self, String> {
        match (current, target) {
            (Self::Build, Self::Build) | (Self::Plan, Self::Plan) => Ok(target),
            (Self::Build, Self::Plan) => Ok(Self::Plan),
            (Self::Plan, Self::Build) if approval == Approval::Approved => Ok(Self::Build),
            (Self::Plan, Self::Build) => Err(
                "transicao plan->build exige aprovacao (use `ktu mode build --approve`)".to_owned(),
            ),
        }
    }
}

/// Aprovação explícita do utilizador para uma transição de modo (I15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Approval {
    /// Aprovada explicitamente (`--approve`).
    Approved,
    /// Sem aprovação (recusa a escalada `plan → build`).
    Pending,
}

/// Estado de uma tarefa do plano.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanState {
    /// Ainda por começar.
    Todo,
    /// Em curso.
    Doing,
    /// Concluída.
    Done,
}

impl PlanState {
    /// Texto estável do estado.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Doing => "doing",
            Self::Done => "done",
        }
    }

    /// Interpreta o texto do estado.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "todo" => Some(Self::Todo),
            "doing" => Some(Self::Doing),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}

/// Uma tarefa do plano.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanTask {
    /// Identificador estável.
    pub id: String,
    /// Objectivo da tarefa.
    pub objective: String,
    /// Estado corrente.
    pub state: PlanState,
    /// Dependências (ids de outras tarefas).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Critério de pronto.
    #[serde(default)]
    pub done_when: String,
}

/// Um estágio do plano (agrupa tarefas).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStage {
    /// Identificador do estágio.
    pub id: String,
    /// Título legível.
    pub title: String,
    /// Tarefas do estágio.
    #[serde(default)]
    pub tasks: Vec<PlanTask>,
}

/// Plano estruturado de uma sessão (estágios → tarefas).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Sessão a que o plano pertence.
    pub session: String,
    /// Estágios, por ordem.
    #[serde(default)]
    pub stages: Vec<PlanStage>,
}

impl Plan {
    /// Cria um plano vazio para uma sessão.
    #[must_use]
    pub fn new(session: impl Into<String>) -> Self {
        Self {
            session: session.into(),
            stages: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests;
