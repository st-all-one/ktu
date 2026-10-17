//! *Dispatcher* de plugins: ordem determinística e regras de fail-closed (I11).

use std::sync::Arc;

use async_trait::async_trait;

use super::{HookPayload, Mode, Plugin, PluginVerdict};

/// Resultado agregado de um *dispatch*.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookVerdict {
    /// `false` se algum gate negou.
    pub allowed: bool,
    /// Motivo da negação (do primeiro gate que negou).
    pub reason: Option<String>,
    /// Geração dos plugins no momento do *dispatch* (I12).
    pub generation: u32,
    /// Número de transformações aplicadas.
    pub transforms: u32,
}

impl HookVerdict {
    /// Veredicto de permissão, sem transformações.
    #[must_use]
    pub const fn pass(generation: u32) -> Self {
        Self {
            allowed: true,
            reason: None,
            generation,
            transforms: 0,
        }
    }
}

/// Porta de intercepção: o kernel despacha por aqui (§2.9).
#[async_trait]
pub trait Hooks: Send + Sync {
    /// Despacha um payload (pode transformá-lo) e devolve o veredicto agregado.
    async fn dispatch(&self, payload: &mut HookPayload) -> HookVerdict;

    /// Geração corrente dos plugins (para o log de evidência).
    fn generation(&self) -> u32;
}

/// Porta de intercepção vazia (sem plugins): tudo passa.
pub struct NoHooks;

#[async_trait]
impl Hooks for NoHooks {
    async fn dispatch(&self, _payload: &mut HookPayload) -> HookVerdict {
        HookVerdict::pass(0)
    }

    fn generation(&self) -> u32 {
        0
    }
}

/// Conjunto determinístico de plugins de uma geração.
pub struct Dispatcher {
    plugins: Vec<Arc<dyn Plugin>>,
    generation: u32,
}

impl Dispatcher {
    /// Cria um *dispatcher* com os plugins dados (ordenados por nome).
    #[must_use]
    pub fn new(mut plugins: Vec<Arc<dyn Plugin>>, generation: u32) -> Self {
        plugins.sort_by(|left, right| left.name().cmp(right.name()));
        Self {
            plugins,
            generation,
        }
    }

    /// Nomes dos plugins, por ordem de execução.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.plugins.iter().map(|plugin| plugin.name()).collect()
    }
}

#[async_trait]
impl Hooks for Dispatcher {
    async fn dispatch(&self, payload: &mut HookPayload) -> HookVerdict {
        let event = payload.event();
        let mut verdict = HookVerdict::pass(self.generation);
        for plugin in &self.plugins {
            if !plugin.subscribes(event) {
                continue;
            }
            match plugin.handle(payload).await {
                PluginVerdict::Pass => {}
                PluginVerdict::Replace { text } => {
                    if plugin.mode() == Mode::Transform {
                        payload.set_text(text);
                        verdict.transforms = verdict.transforms.saturating_add(1);
                    }
                }
                PluginVerdict::Deny { reason } => {
                    // Só um `gate` nega; observe/transform nunca restringem (I11).
                    if plugin.mode() == Mode::Gate {
                        verdict.allowed = false;
                        verdict.reason = Some(reason);
                        return verdict;
                    }
                }
            }
        }
        verdict
    }

    fn generation(&self) -> u32 {
        self.generation
    }
}
