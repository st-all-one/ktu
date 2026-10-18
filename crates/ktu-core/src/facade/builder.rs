//! Construtor de [`Ktu`]: dependências explícitas, uma a uma (DI sem structs do kernel).

use std::path::{Path, PathBuf};

use super::{Ktu, system_prompt_in};
use crate::agent::{AgentRegistry, AgentSpec, LoopConfig, Steering};
use crate::budget::BudgetGate;
use crate::cancel::CancelToken;
use crate::guardrails::{self, DeclarativeGuard};
use crate::memory::Memory;
use crate::permission::{self, PermissionGuard};
use crate::plugin::Hooks;
use crate::ports::{Clock, Fs, Instructions, Logger, Metrics};
use crate::provider::Provider;
use crate::subagent::SubagentHost;
use crate::tools::ToolExecutor;
use crate::{Error, Result};

/// Construtor de [`Ktu`]: dependências explícitas, uma a uma (DI sem structs do kernel).
#[derive(Default)]
pub struct KtuBuilder<'a> {
    root: Option<PathBuf>,
    fs: Option<&'a dyn Fs>,
    clock: Option<&'a dyn Clock>,
    logger: Option<&'a dyn Logger>,
    metrics: Option<&'a dyn Metrics>,
    provider: Option<&'a dyn Provider>,
    tools: Option<&'a dyn ToolExecutor>,
    budget: Option<&'a BudgetGate>,
    cancel: Option<&'a CancelToken>,
    registry: Option<&'a AgentRegistry>,
    plan_spec: Option<&'a AgentSpec>,
    user_dir: Option<&'a Path>,
    memory: Option<&'a dyn Memory>,
    hooks: Option<&'a dyn Hooks>,
    subagents: Option<&'a dyn SubagentHost>,
    steering: Option<&'a dyn Steering>,
    instructions: Option<&'a dyn Instructions>,
    max_steps: Option<u32>,
}

impl<'a> KtuBuilder<'a> {
    /// Raiz do projecto (`.agents/`, `.ktu/` e `AGENTS.md`).
    #[must_use]
    pub fn root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Porta de sistema de ficheiros (guardrails, permissões, `AGENTS.md`, sessões).
    #[must_use]
    pub fn fs(mut self, fs: &'a dyn Fs) -> Self {
        self.fs = Some(fs);
        self
    }

    /// Porta de tempo (métricas de duração e carimbos de sessão).
    #[must_use]
    pub fn clock(mut self, clock: &'a dyn Clock) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Porta de log (os eventos do turno nunca são silenciados).
    #[must_use]
    pub fn logger(mut self, logger: &'a dyn Logger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Porta de métricas.
    #[must_use]
    pub fn metrics(mut self, metrics: &'a dyn Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Provider do modelo.
    #[must_use]
    pub fn provider(mut self, provider: &'a dyn Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Executor de tools (T2).
    #[must_use]
    pub fn tools(mut self, tools: &'a dyn ToolExecutor) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Dono único do tecto de contexto (I14) — partilhado com os subagentes.
    #[must_use]
    pub fn budget(mut self, budget: &'a BudgetGate) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Sinal de cancelamento (I4) — partilhado com a borda que cancela.
    #[must_use]
    pub fn cancel(mut self, cancel: &'a CancelToken) -> Self {
        self.cancel = Some(cancel);
        self
    }

    /// Registo de agentes (o primário define as tools e o *enforcement* estrutural).
    #[must_use]
    pub fn registry(mut self, registry: &'a AgentRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Especificação read-only do modo plano (sem ela, o modo plano é recusado).
    #[must_use]
    pub fn plan_spec(mut self, spec: &'a AgentSpec) -> Self {
        self.plan_spec = Some(spec);
        self
    }

    /// Directório de configuração do **utilizador** (`~/.config/ktu`), onde se
    /// procuram `SYSTEM.md`/`APPEND_SYSTEM.md` (o projecto tem precedência).
    #[must_use]
    pub fn user_dir(mut self, dir: &'a Path) -> Self {
        self.user_dir = Some(dir);
        self
    }

    /// Memória persistente (recall no arranque da sessão; `None` desliga).
    #[must_use]
    pub fn memory(mut self, memory: &'a dyn Memory) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Intercepção de plugins (§2.9; `None` desliga o *dispatch*).
    #[must_use]
    pub fn hooks(mut self, hooks: &'a dyn Hooks) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Host de subagentes (`None` desliga a tool `subagent`).
    #[must_use]
    pub fn subagents(mut self, subagents: &'a dyn SubagentHost) -> Self {
        self.subagents = Some(subagents);
        self
    }

    /// *Steering* do utilizador durante o turno (`None` desliga).
    #[must_use]
    pub fn steering(mut self, steering: &'a dyn Steering) -> Self {
        self.steering = Some(steering);
        self
    }

    /// Instruções locais por directório (`AGENTS.md` mais próximo de um ficheiro
    /// lido; `None` desliga a anexação ao resultado da tool).
    #[must_use]
    pub fn instructions(mut self, instructions: &'a dyn Instructions) -> Self {
        self.instructions = Some(instructions);
        self
    }

    /// Tecto de passos por turno (default: [`LoopConfig::default`]).
    #[must_use]
    pub fn max_steps(mut self, max_steps: u32) -> Self {
        self.max_steps = Some(max_steps);
        self
    }

    /// Monta o kernel: valida as portas e resolve o projecto.
    ///
    /// Guardrails e permissões inválidos **impedem o arranque** (I16) — a fachada
    /// nunca degrada em silêncio.
    ///
    /// # Errors
    /// Devolve [`Error::Invalid`] se faltar uma porta obrigatória ou se os
    /// guardrails/permissões do projecto não carregarem.
    pub fn build(self) -> Result<Ktu<'a>> {
        let root = required(self.root, "root")?;
        let fs = required(self.fs, "porta Fs")?;
        // Guardrails **globais** primeiro (piso aditivo, M5): a cadeia só consulta a
        // regra seguinte se a anterior deixar passar, pelo que um ficheiro do projecto
        // nunca desliga uma regra global — só pode acrescentar.
        let mut rules = match self.user_dir {
            Some(dir) => guardrails::load(fs, &dir.join("guardrails")).map_err(|error| {
                Error::invalid(format!("guardrails globais invalidos: {error}"))
            })?,
            None => Vec::new(),
        };
        rules.extend(
            guardrails::load(fs, &root.join(".agents").join("guardrails"))
                .map_err(|error| Error::invalid(format!("guardrails invalidos: {error}")))?,
        );
        let permissions = permission::load(fs, &root)
            .map_err(|error| Error::invalid(format!("erro nas permissoes: {error}")))?;
        let system = system_prompt_in(fs, &root, self.user_dir);
        let default = LoopConfig::default();
        Ok(Ktu {
            root,
            fs,
            clock: required(self.clock, "porta Clock")?,
            logger: required(self.logger, "porta Logger")?,
            metrics: required(self.metrics, "porta Metrics")?,
            provider: required(self.provider, "provider")?,
            tools: required(self.tools, "porta de tools")?,
            budget: required(self.budget, "porta de orcamento")?,
            cancel: required(self.cancel, "porta de cancelamento")?,
            registry: required(self.registry, "registo de agentes")?,
            plan_spec: self.plan_spec,
            memory: self.memory,
            hooks: self.hooks,
            subagents: self.subagents,
            steering: self.steering,
            instructions: self.instructions,
            declarative: DeclarativeGuard::new(rules),
            permission: PermissionGuard::new(permissions),
            system,
            loop_config: LoopConfig {
                max_steps: self.max_steps.unwrap_or(default.max_steps),
                ..default
            },
        })
    }
}

/// Extrai um campo obrigatório do construtor, nomeando-o no erro.
fn required<T>(value: Option<T>, name: &str) -> Result<T> {
    value.ok_or_else(|| Error::invalid(format!("fachada sem {name}")))
}
