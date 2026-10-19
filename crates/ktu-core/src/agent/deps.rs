//! Configuração do loop e dependências injectadas (DI) em grupos coesos.

use crate::budget::BudgetGate;
use crate::cancel::CancelToken;
use crate::guard::Guard;
use crate::memory::Memory;
use crate::plugin::Hooks;
use crate::ports::{Clock, Instructions, Logger, Metrics};
use crate::provider::Provider;
use crate::session::SteerMessage;
use crate::subagent::SubagentHost;
use crate::tools::ToolExecutor;

/// Porta de *steering*: mensagens do utilizador entregues numa **fronteira segura**
/// (início de um passo), sem matar o turno.
///
/// A mensagem é **admitida** no log ([`crate::session::Event::SteerQueued`]) e só depois
/// entregue ([`crate::session::Event::Steer`], no passo seguinte) — preservando
/// *model-visible ⟺ logged* e tornando a admissão **durável** (M7). O `id` vem da
/// **borda**: repetir o mesmo `id` não duplica a mensagem.
pub trait Steering: Send + Sync {
    /// Retira as mensagens admitidas e ainda não entregues, por ordem de chegada.
    fn take(&self) -> Vec<SteerMessage>;
}

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
    /// *Steering*: mensagens do utilizador durante o turno (`None` desliga).
    pub steering: Option<&'d dyn Steering>,
    /// Instruções locais por directório: `AGENTS.md` mais próximo de um ficheiro
    /// lido (`None` desliga a anexação ao resultado).
    pub instructions: Option<&'d dyn Instructions>,
}

/// Dependências do loop (injecção explícita — DI), em três grupos coesos.
///
/// É a API de **baixo nível**: quem embute o ktu usa a fachada
/// [`crate::facade::Ktu`] em vez de montar este struct.
pub struct Deps<'d> {
    /// Portas de infraestrutura.
    pub ports: Ports<'d>,
    /// Observabilidade e tempo.
    pub telemetry: Telemetry<'d>,
    /// Capacidades opcionais.
    pub capabilities: Capabilities<'d>,
}
