//! Fachada de incorporação: monta o kernel a partir de **portas injectadas**.
//!
//! É o **único** caminho suportado de integração num outro projecto Rust (uso #1):
//! quem embute o ktu vê [`Ktu`], [`TurnOutcome`], [`SessionLog`] e [`Event`] — nunca
//! `Deps`/`Ports`/`Capabilities`/`LoopConfig`
//! nem `run_turn`. A fachada faz a composição **pura** (guardrails, permissões,
//! system prompt, orçamento, modo, sessão) e delega no loop; todos os efeitos
//! continuam a entrar por portas (M2).
//!
//! Como o `ktu-core` é puro, a fachada **não** constrói adaptadores reais: as
//! portas e as capacidades com efeitos (`Provider`, `ToolExecutor`, `Memory`,
//! `Hooks`, `SubagentHost`, `Steering`) são injectadas pela borda — o `ktu-cli`
//! monta as reais, os testes montam os *fakes*.
//!
//! ```
//! use ktu_core::agent::{AgentRegistry, AgentSpec};
//! use ktu_core::budget::BudgetGate;
//! use ktu_core::cancel::CancelToken;
//! use ktu_core::domain::Completion;
//! use ktu_core::facade::{Ktu, TurnOutcome};
//! use ktu_core::guard::AgentId;
//! use ktu_core::ports::{FixedClock, MemFs, NoopLogger, NoopMetrics, Timestamp};
//! use ktu_core::session::SessionLog;
//! use ktu_core::testing::{FakeProvider, FakeTools};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let fs = MemFs::new();
//! let clock = FixedClock::new(Timestamp::from_millis(0));
//! let logger = NoopLogger;
//! let metrics = NoopMetrics;
//! let provider = FakeProvider::new(vec![Completion {
//!     text: "feito".to_owned(),
//!     reasoning: String::new(),
//!     tool_calls: Vec::new(),
//! }]);
//! let tools = FakeTools::new(Vec::new(), Vec::new());
//! let registry = AgentRegistry::new(AgentSpec::new(AgentId::Primary, Vec::new(), 1), None);
//! let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
//! let cancel = CancelToken::new();
//!
//! let ktu = Ktu::builder()
//!     .root("/projeto")
//!     .fs(&fs)
//!     .clock(&clock)
//!     .logger(&logger)
//!     .metrics(&metrics)
//!     .provider(&provider)
//!     .tools(&tools)
//!     .budget(&budget)
//!     .cancel(&cancel)
//!     .registry(&registry)
//!     .build()?;
//!
//! let mut log = SessionLog::new();
//! let runtime = tokio::runtime::Builder::new_current_thread().build()?;
//! runtime.block_on(async {
//!     let outcome: TurnOutcome = ktu.turn(&mut log, "faz X").await?;
//!     assert_eq!(outcome.text, "feito");
//!     Ok::<(), ktu_core::Error>(())
//! })?;
//! # Ok(())
//! # }
//! ```

use std::path::{Path, PathBuf};

use crate::agent::{
    AgentRegistry, AgentSpec, Capabilities, Deps, LoopConfig, Ports, Steering, Telemetry, run_turn,
};
use crate::agents_md;
use crate::budget::BudgetGate;
use crate::cancel::CancelToken;
use crate::domain::StopReason;
use crate::guard::{Decision, Fact, Guard, StructuralGuard};
use crate::guardrails::{self, ChainGuard, DeclarativeGuard, Rule};
use crate::layout::{AGENTS_DIR, SESSIONS_DIR};
use crate::memory::Memory;
use crate::permission::{self, PermissionGuard};
use crate::plan::Mode;
use crate::plugin::Hooks;
use crate::ports::{Clock, Fs, Instructions, Logger, Metrics};
use crate::prompt::{BASE, Directive, compose_layers};
use crate::provider::Provider;
use crate::session::{Event, SessionLog};
use crate::skill;
use crate::store;
use crate::subagent::SubagentHost;
use crate::tool_search;
use crate::tools::{ScopedTools, ToolExecutor};
use crate::{Error, Result};

/// Resultado estável de um turno: a razão de término (I5) e a resposta final.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnOutcome {
    /// Razão de término do turno — nunca em silêncio (I5).
    pub reason: StopReason,
    /// Resposta final do turno (último passo sem `tool_calls`; vazio se não houve).
    pub text: String,
}

impl TurnOutcome {
    /// `true` quando o turno terminou sem erro (`StopReason::Error`).
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.reason != StopReason::Error
    }
}

mod builder;

pub use builder::KtuBuilder;

/// Kernel montado: portas, capacidades e a composição pura do projecto.
pub struct Ktu<'a> {
    root: PathBuf,
    fs: &'a dyn Fs,
    clock: &'a dyn Clock,
    logger: &'a dyn Logger,
    metrics: &'a dyn Metrics,
    provider: &'a dyn Provider,
    tools: &'a dyn ToolExecutor,
    budget: &'a BudgetGate,
    cancel: &'a CancelToken,
    registry: &'a AgentRegistry,
    plan_spec: Option<&'a AgentSpec>,
    memory: Option<&'a dyn Memory>,
    hooks: Option<&'a dyn Hooks>,
    subagents: Option<&'a dyn SubagentHost>,
    steering: Option<&'a dyn Steering>,
    instructions: Option<&'a dyn Instructions>,
    declarative: DeclarativeGuard,
    permission: PermissionGuard,
    system: String,
    loop_config: LoopConfig,
}

impl Ktu<'_> {
    /// Tecto de contexto por turno (estimativa conservadora ≈ 4 bytes/token) —
    /// dono único: [`BudgetGate`] (I14).
    pub const CONTEXT_BUDGET_TOKENS: usize = 120_000;

    /// Construtor com opções.
    #[must_use]
    pub fn builder<'a>() -> KtuBuilder<'a> {
        KtuBuilder::default()
    }

    /// Raiz do projecto (`.agents/`, `.ktu/` e `AGENTS.md`).
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// System prompt efectivo (base do agente + `AGENTS.md` condensado).
    #[must_use]
    pub fn system(&self) -> &str {
        &self.system
    }

    /// Corre **um turno** sobre `log` — o log **é** a sessão (I3).
    ///
    /// # Errors
    /// Devolve [`Error::Invalid`] quando a sessão está em modo plano e a fachada
    /// não recebeu uma especificação read-only: o modo plano nunca escala para o
    /// agente primário (I15).
    pub async fn turn(&self, log: &mut SessionLog, prompt: &str) -> Result<TurnOutcome> {
        let spec = self.spec_for(log.mode())?;
        // O conjunto do agente é a **capacidade** (o guard conhece-o todo); a
        // exposição é do catálogo: tools diferidas não vão ao modelo até
        // `tool_search` — que, por isso, entra no conjunto quando existem.
        let mut names = spec.tool_names();
        if !self.tools.deferred().is_empty() {
            names.push(tool_search::TOOL.to_owned());
        }
        let scoped = ScopedTools::new(self.tools, names.clone());
        let structural = StructuralGuard::new(&self.root, names, spec.max_depth());
        let inner = ChainGuard::new(&self.declarative, &self.permission);
        let guard = ChainGuard::new(&structural, &inner);
        let deps = Deps {
            ports: Ports {
                provider: self.provider,
                tools: &scoped,
                guard: &guard,
                budget: self.budget,
                cancel: self.cancel,
            },
            telemetry: Telemetry {
                logger: self.logger,
                metrics: self.metrics,
                clock: self.clock,
            },
            capabilities: Capabilities {
                memory: self.memory,
                subagents: self.subagents,
                plugins: self.hooks,
                steering: self.steering,
                instructions: self.instructions,
            },
        };
        let reason = run_turn(log, &deps, &self.system, prompt, &self.loop_config).await;
        Ok(TurnOutcome {
            reason,
            text: final_text(log).unwrap_or_default(),
        })
    }

    /// Carrega o log de uma sessão persistida (`<root>/.ktu/sessions/<id>.jsonl`).
    ///
    /// A última linha parcial (escrita interrompida) é tolerada; qualquer outra
    /// linha malformada é erro.
    ///
    /// # Errors
    /// Propaga erros de I/O e de JSON da porta.
    pub fn session(&self, id: &str) -> Result<Option<SessionLog>> {
        store::load_recovering(self.fs, &self.sessions_dir(), id)
    }

    /// Grava o log de uma sessão persistida de forma durável (`temp` + `rename`).
    ///
    /// # Errors
    /// Propaga erros de I/O da porta.
    pub fn save_session(&self, id: &str, log: &SessionLog) -> Result<()> {
        store::save(self.fs, &self.sessions_dir(), id, log)
    }

    /// Directório das sessões do projecto.
    #[must_use]
    pub fn sessions_dir(&self) -> PathBuf {
        self.root.join(SESSIONS_DIR)
    }

    /// Especificação do agente para o modo da sessão (read-only é estrutural).
    fn spec_for(&self, mode: Mode) -> Result<&AgentSpec> {
        match mode {
            Mode::Build => Ok(self.registry.primary()),
            Mode::Plan => self.plan_spec.ok_or_else(|| {
                Error::invalid("modo plano sem especificacao read-only (plan_spec)")
            }),
        }
    }
}

/// Compõe o system prompt do projecto: base do agente + `AGENTS.md` condensado.
///
/// É a **mesma** função que a fachada usa, para que a borda possa compor o prompt
/// antes de a montar (ex.: passá-lo a um host de subagentes). Não lê a directriz
/// do utilizador: para isso existe [`system_prompt_in`].
#[must_use]
pub fn system_prompt(fs: &dyn Fs, root: &Path) -> String {
    system_prompt_in(fs, root, None)
}

/// Compõe o system prompt com a directriz do **utilizador** (`user_dir`) e do
/// **projecto** (`<root>/.agents/`).
///
/// Precedência (homónimos não se combinam): `SYSTEM.md`/`APPEND_SYSTEM.md` do
/// projecto ganham aos do utilizador; a ausência de ambos deixa a base do agente.
#[must_use]
pub fn system_prompt_in(fs: &dyn Fs, root: &Path, user_dir: Option<&Path>) -> String {
    let user = user_dir.map_or_else(Directive::default, |dir| Directive::load(fs, dir));
    let project = Directive::load(fs, &root.join(AGENTS_DIR));
    let directive = Directive::resolve(user, project);
    // `AGENTS.md` da raiz + aninhados (raiz primeiro, o mais fundo mais tarde).
    let docs = agents_md::discover(fs, root);
    let instructions = agents_md::render(&docs, root);
    // Skills dos dois níveis (M8): local ganha por nome; o corpo compõe o prompt.
    let (skills, _problems) = skill::load(fs, user_dir, root);
    let skills = skill::section(&skills);
    let base = directive.system().unwrap_or(BASE);
    compose_layers(
        base,
        instructions.as_deref(),
        skills.as_deref(),
        directive.append(),
    )
}

/// Avalia **um facto isolado** com o **mesmo** encadeamento de guardrails que a
/// fachada usa num turno (estrutural → declarativo → permissões).
///
/// Serve a borda para decidir efeitos que acontecem **fora** de um turno — hoje, o
/// *shell* embutido de um comando do projecto (`.agents/commands/*.md`), que assim
/// passa pelo guard em vez de o contornar (I10). Fail-closed: guardrails ou
/// permissões inválidos devolvem erro.
///
/// # Errors
/// Devolve [`Error::Invalid`] se os guardrails/permissões do projecto não carregarem.
pub fn evaluate_fact(
    spec: &AgentSpec,
    root: &Path,
    user_dir: Option<&Path>,
    fs: &dyn Fs,
    fact: &Fact,
) -> Result<Decision> {
    let rules = declarative_rules(fs, root, user_dir)?;
    let permissions = permission::load(fs, root)
        .map_err(|error| Error::invalid(format!("erro nas permissoes: {error}")))?;
    let structural = StructuralGuard::for_spec(root, spec);
    let declarative = DeclarativeGuard::new(rules);
    let permission = PermissionGuard::new(permissions);
    let inner = ChainGuard::new(&declarative, &permission);
    let chain = ChainGuard::new(&structural, &inner);
    Ok(chain.evaluate(fact))
}

/// Regras declarativas dos **dois** níveis, com o piso global primeiro (M5).
///
/// É a **mesma** composição no `build()` e no [`evaluate_fact`]: uma só função, para
/// que o encadeamento da fachada e o da borda não possam divergir.
///
/// # Errors
/// Devolve [`Error::Invalid`] se uma regra de qualquer dos níveis não carregar (I16).
pub(super) fn declarative_rules(
    fs: &dyn Fs,
    root: &Path,
    user_dir: Option<&Path>,
) -> Result<Vec<Rule>> {
    guardrails::load_layered(fs, user_dir, root)
        .map(|rules| rules.into_iter().map(|(_, rule)| rule).collect())
        .map_err(|error| Error::invalid(format!("guardrails invalidos: {error}")))
}

/// Resposta final do **turno corrente** (último passo sem `tool_calls`).
#[must_use]
pub fn final_text(log: &SessionLog) -> Option<String> {
    let events = log.events();
    let start = events
        .iter()
        .rposition(|event| matches!(event, Event::TurnStart { .. }))
        .map_or(0, |index| index.saturating_add(1));
    events
        .get(start..)
        .unwrap_or_default()
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::Assistant {
                content,
                tool_calls,
                ..
            } if tool_calls.is_empty() => Some(content.clone()),
            _ => None,
        })
}

#[cfg(test)]
mod tests;
