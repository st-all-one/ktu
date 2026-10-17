//! Pontos de intercepção (§2.9) e o *dispatcher* de plugins.
//!
//! O catálogo de eventos é **tipado e único** (não há dois catálogos): serve o
//! *dispatch* interno e o protocolo externo. O guardrail estrutural é sempre
//! avaliado **primeiro** (I11); plugins só podem **acrescentar** restrição e
//! **transformar** dados — nunca transformar `Deny` em `Allow`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod dispatch;

pub use dispatch::{Dispatcher, HookVerdict, Hooks, NoHooks};

/// Ponto de intercepção do loop (§2.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    /// Sessão nova (pode injectar instruções/contexto).
    SessionStart,
    /// Fim adequado da sessão (flush, resumo).
    SessionEnd,
    /// Input do utilizador (inclui *steering*).
    UserInput,
    /// Antes de enviar o pedido ao modelo (contexto, system, tools).
    ModelRequest,
    /// Saída do modelo (texto, pensamento, tool-call).
    ModelOutput,
    /// Antes do efeito de uma tool.
    ActionBefore,
    /// Depois do efeito de uma tool (resultado tipado).
    ActionAfter,
    /// Cada decisão de guardrail (evidência da tese §2.8).
    GuardDecision,
    /// Erro interno do kernel.
    KernelError,
    /// Ciclo de vida dos plugins (carregado, recarregado, errado).
    PluginLifecycle,
}

impl HookEvent {
    /// Catálogo completo, por ordem estável.
    pub const ALL: &'static [Self] = &[
        Self::SessionStart,
        Self::SessionEnd,
        Self::UserInput,
        Self::ModelRequest,
        Self::ModelOutput,
        Self::ActionBefore,
        Self::ActionAfter,
        Self::GuardDecision,
        Self::KernelError,
        Self::PluginLifecycle,
    ];

    /// Nome estável (contrato de manifesto e de log).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionStart => "session.start",
            Self::SessionEnd => "session.end",
            Self::UserInput => "user.input",
            Self::ModelRequest => "model.request",
            Self::ModelOutput => "model.output",
            Self::ActionBefore => "action.before",
            Self::ActionAfter => "action.after",
            Self::GuardDecision => "guard.decision",
            Self::KernelError => "kernel.error",
            Self::PluginLifecycle => "plugin.lifecycle",
        }
    }

    /// Interpreta o nome estável de um evento (`None` se desconhecido).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|event| event.as_str() == name)
    }
}

/// Modo de um plugin, por evento subscrito.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Só observa; a falha passa adiante.
    Observe,
    /// Transforma dados; a falha passa adiante sem alteração.
    Transform,
    /// Decide; timeout/erro é `Deny` (**fail-closed**, I11).
    Gate,
}

/// Payload tipado entregue a um plugin.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HookPayload {
    /// Sessão nova.
    SessionStart,
    /// Fim de sessão, com a razão.
    SessionEnd {
        /// Razão de término (texto estável).
        reason: String,
    },
    /// Input do utilizador (transformável).
    UserInput {
        /// Texto do input.
        text: String,
    },
    /// Pedido ao modelo (contagens).
    ModelRequest {
        /// Número de mensagens enviadas.
        messages: usize,
        /// Número de tools oferecidas.
        tools: usize,
    },
    /// Saída do modelo (transformável).
    ModelOutput {
        /// Texto emitido.
        text: String,
    },
    /// Antes do efeito de uma tool.
    ActionBefore {
        /// Nome da tool.
        tool: String,
        /// Argumentos JSON.
        arguments: String,
    },
    /// Depois do efeito de uma tool (resultado transformável).
    ActionAfter {
        /// Nome da tool.
        tool: String,
        /// Conteúdo do resultado.
        result: String,
        /// Se o resultado é um erro.
        is_error: bool,
    },
    /// Decisão de guardrail.
    GuardDecision {
        /// Tool avaliada.
        tool: String,
        /// Decisão (texto estável).
        decision: String,
    },
    /// Erro interno do kernel.
    KernelError {
        /// Mensagem do erro.
        message: String,
    },
    /// Ciclo de vida de um plugin.
    PluginLifecycle {
        /// Nome do plugin.
        plugin: String,
        /// Estado (`loaded`, `reloaded`, `error`).
        state: String,
    },
}

impl HookPayload {
    /// Evento a que este payload corresponde.
    #[must_use]
    pub const fn event(&self) -> HookEvent {
        match self {
            Self::SessionStart => HookEvent::SessionStart,
            Self::SessionEnd { .. } => HookEvent::SessionEnd,
            Self::UserInput { .. } => HookEvent::UserInput,
            Self::ModelRequest { .. } => HookEvent::ModelRequest,
            Self::ModelOutput { .. } => HookEvent::ModelOutput,
            Self::ActionBefore { .. } => HookEvent::ActionBefore,
            Self::ActionAfter { .. } => HookEvent::ActionAfter,
            Self::GuardDecision { .. } => HookEvent::GuardDecision,
            Self::KernelError { .. } => HookEvent::KernelError,
            Self::PluginLifecycle { .. } => HookEvent::PluginLifecycle,
        }
    }

    /// Texto transformável deste payload, se existir.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::UserInput { text } | Self::ModelOutput { text } => Some(text),
            Self::ActionAfter { result, .. } => Some(result),
            _ => None,
        }
    }

    /// Substitui o texto transformável (no-op se o payload não o tiver).
    pub fn set_text(&mut self, value: String) {
        match self {
            Self::UserInput { text } | Self::ModelOutput { text } => *text = value,
            Self::ActionAfter { result, .. } => *result = value,
            _ => {}
        }
    }
}

/// Veredicto de um plugin sobre um payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum PluginVerdict {
    /// Não altera nada.
    Pass,
    /// Substitui o texto transformável.
    Replace {
        /// Novo texto.
        text: String,
    },
    /// Nega a acção (só efectivo em modo `gate`).
    Deny {
        /// Motivo (texto estável).
        reason: String,
    },
}

/// Um plugin: um subscritor de eventos do loop.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Nome estável (ordem determinística do *dispatch*).
    fn name(&self) -> &str;

    /// Modo do plugin.
    fn mode(&self) -> Mode;

    /// Se o plugin subscreve este evento.
    fn subscribes(&self, event: HookEvent) -> bool;

    /// Trata um payload. Erros devem virar `Pass` (observe/transform) ou `Deny`
    /// (**gate**, fail-closed) — nunca pânico.
    async fn handle(&self, payload: &HookPayload) -> PluginVerdict;
}

#[cfg(test)]
mod tests;
