//! *Host* de plugins: carrega, recarrega por **geração** (I12) e despacha.

use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock};

use async_trait::async_trait;
use ktu_core::plugin::{Dispatcher, HookPayload, HookVerdict, Hooks, Plugin};

use crate::manifest::Manifest;
use crate::process::ProcessPlugin;

/// *Host* de plugins carregados de `.ktu/plugins/<nome>/plugin.toml`.
pub struct PluginHost {
    dir: PathBuf,
    dispatcher: RwLock<Arc<Dispatcher>>,
    errors: RwLock<Vec<String>>,
}

impl PluginHost {
    /// Carrega os plugins de `dir` (primeira geração). Plugins inválidos são
    /// desactivados (registados em [`Self::errors`]); a sessão continua.
    #[must_use]
    pub fn load(dir: PathBuf) -> Self {
        let (plugins, errors) = scan(&dir);
        Self {
            dir,
            dispatcher: RwLock::new(Arc::new(Dispatcher::new(plugins, 1))),
            errors: RwLock::new(errors),
        }
    }

    /// Recarrega os plugins, incrementando a geração (I12). Devolve os erros.
    ///
    /// Os eventos **em voo** completam na geração anterior: o `dispatch` fixa o
    /// `Arc<Dispatcher>` no início, e esta troca só afecta chamadas futuras.
    pub fn reload(&self) -> Vec<String> {
        let (plugins, errors) = scan(&self.dir);
        let next = self.generation().saturating_add(1);
        {
            let mut guard = self
                .dispatcher
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            *guard = Arc::new(Dispatcher::new(plugins, next));
        }
        {
            let mut guard = self.errors.write().unwrap_or_else(PoisonError::into_inner);
            guard.clone_from(&errors);
        }
        errors
    }

    /// Erros da última carga/recarga.
    #[must_use]
    pub fn errors(&self) -> Vec<String> {
        self.errors
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Nomes dos plugins activos, por ordem de execução.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        let guard = self
            .dispatcher
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        guard
            .names()
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl Hooks for PluginHost {
    async fn dispatch(&self, payload: &mut HookPayload) -> HookVerdict {
        // Fixa a geração: recarregar a meio não afecta este *dispatch* (I12).
        let dispatcher = {
            let guard = self
                .dispatcher
                .read()
                .unwrap_or_else(PoisonError::into_inner);
            Arc::clone(&guard)
        };
        dispatcher.dispatch(payload).await
    }

    fn generation(&self) -> u32 {
        let guard = self
            .dispatcher
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        guard.generation()
    }
}

/// Resultado de uma descoberta: plugins válidos e erros dos inválidos.
type ScanResult = (Vec<Arc<dyn Plugin>>, Vec<String>);

/// Descobre e constrói os plugins de `<dir>/<nome>/plugin.toml`.
fn scan(dir: &Path) -> ScanResult {
    let mut plugins: Vec<Arc<dyn Plugin>> = Vec::new();
    let mut errors = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (plugins, errors);
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    for plugin_dir in dirs {
        match load_plugin(&plugin_dir) {
            Ok(plugin) => plugins.push(plugin),
            Err(error) => errors.push(error),
        }
    }
    (plugins, errors)
}

/// Carrega um plugin de um directório, nomeando-o em caso de erro.
fn load_plugin(dir: &Path) -> Result<Arc<dyn Plugin>, String> {
    let manifest = Manifest::load(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let plugin = ProcessPlugin::new(&manifest, dir.to_path_buf())?;
    Ok(Arc::new(plugin))
}
