//! Configuração do loop e dependências injectadas (DI) em grupos coesos.

use crate::budget::BudgetGate;
use crate::cancel::CancelToken;
use crate::guard::Guard;
use crate::memory::Memory;
use crate::plugin::Hooks;
use crate::ports::{Clock, Logger, Metrics};
use crate::provider::Provider;
use crate::subagent::SubagentHost;
use crate::tools::ToolExecutor;

/// Configuração do loop.
#[derive(Debug, Clone, Copy)]
pub struct LoopConfig {
    /// Tecto de passos por turno.
    pub max_steps: u32,
    /// Repetições da mesma assinatura de tools antes de considerar estagnação.
    pub max_stagnation: u32,
    /// Tecto (em escalares) do relatório devolvido por um subagente.
    pub subagent_report_chars: usize,
    /// Número máximo de subagentes evocados por turno (fan-out).
    pub max_subagents_per_turn: u32,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_steps: 24,
            max_stagnation: 3,
            subagent_report_chars: 8192,
            max_subagents_per_turn: 4,
        }
    }
}

/// Portas de infraestrutura (obrigatórias) do turno.
pub struct Ports<'d> {
    /// Provider do modelo.
    pub provider: &'d dyn Provider,
    /// Executor de tools (T2).
    pub tools: &'d dyn ToolExecutor,
    /// Guardrail estrutural.
    pub guard: &'d dyn Guard,
    /// Dono único do tecto de contexto (I14).
    pub budget: &'d BudgetGate,
    /// Sinal de cancelamento (I4).
    pub cancel: &'d CancelToken,
}

/// Observabilidade e tempo (portas de log, métricas e relógio).
pub struct Telemetry<'d> {
    /// Porta de log.
    pub logger: &'d dyn Logger,
    /// Porta de métricas.
    pub metrics: &'d dyn Metrics,
    /// Porta de tempo.
    pub clock: &'d dyn Clock,
}

/// Capacidades opcionais (podem estar desligadas num turno concreto).
#[derive(Default)]
pub struct Capabilities<'d> {
    /// Memória persistente (recall no início de sessão; `None` desliga).
    pub memory: Option<&'d dyn Memory>,
    /// Delegação para subagentes (`None` desliga a tool `subagent`).
    pub subagents: Option<&'d dyn SubagentHost>,
    /// Intercepção de plugins (`None` desliga o *dispatch* de *hooks* — §2.9).
    pub plugins: Option<&'d dyn Hooks>,
}

/// Dependências do loop (injecção explícita — DI), em três grupos coesos.
pub struct Deps<'d> {
    /// Portas de infraestrutura.
    pub ports: Ports<'d>,
    /// Observabilidade e tempo.
    pub telemetry: Telemetry<'d>,
    /// Capacidades opcionais.
    pub capabilities: Capabilities<'d>,
}
