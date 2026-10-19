//! *Host* de plugins: carrega, recarrega por **geração** (I12) e despacha.
//!
//! Dois níveis (M8): o **global** do utilizador (`<user_dir>/plugins`) e o do
//! **projecto** (`.ktu/plugins`). São **aditivos** — um plugin do projecto com o
//! mesmo nome **não** desliga o global; os dois correm, e a listagem diz de onde
//! vem cada um.

use std::path::{Path, PathBuf};
use std::sync::{Arc, PoisonError, RwLock};

use async_trait::async_trait;
use ktu_core::Origin;
use ktu_core::plugin::{Dispatcher, HookPayload, HookVerdict, Hooks, Plugin};

use crate::manifest::Manifest;
use crate::process::ProcessPlugin;

/// Directório dos plugins **globais** (por baixo do directório do utilizador).
pub const PLUGINS_DIR: &str = "plugins";

/// Um plugin activo e a sua origem (nome + nível).
pub type Entry = (String, Origin);

/// *Host* de plugins carregados de `.ktu/plugins/<nome>/plugin.toml`.
pub struct PluginHost {
    dirs: Vec<(Origin, PathBuf)>,
    dispatcher: RwLock<Arc<Dispatcher>>,
    entries: RwLock<Vec<Entry>>,
    errors: RwLock<Vec<String>>,
}

impl PluginHost {
    /// Carrega os plugins de **um** directório (`<dir>/<nome>/plugin.toml`), como
    /// nível local. Plugins inválidos são desactivados (registados em
    /// [`Self::errors`]); a sessão continua.
    #[must_use]
    pub fn load(dir: PathBuf) -> Self {
        Self::layered(vec![(Origin::Local, dir)])
    }

    /// Carrega os plugins de **vários** níveis, por ordem (o global primeiro).
    ///
    /// Os níveis são **aditivos** (M8): um plugin do projecto com o mesmo nome
    /// **não** desliga o global — os dois correm, e [`Self::entries`] diz de onde
    /// vem cada um. Plugins inválidos desactivam-se; a sessão continua.
    #[must_use]
    pub fn layered(levels: Vec<(Origin, PathBuf)>) -> Self {
        let (plugins, entries, errors) = scan(&levels);
        Self {
            dirs: levels,
            dispatcher: RwLock::new(Arc::new(Dispatcher::new(plugins, 1))),
            entries: RwLock::new(entries),
            errors: RwLock::new(errors),
        }
    }

    /// Recarrega os plugins, incrementando a geração (I12). Devolve os erros.
    ///
    /// Os eventos **em voo** completam na geração anterior: o `dispatch` fixa o
    /// `Arc<Dispatcher>` no início, e esta troca só afecta chamadas futuras.
    pub fn reload(&self) -> Vec<String> {
        let (plugins, entries, errors) = scan(&self.dirs);
        let next = self.generation().saturating_add(1);
        {
            let mut guard = self
                .dispatcher
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            *guard = Arc::new(Dispatcher::new(plugins, next));
        }
        {
            let mut guard = self.entries.write().unwrap_or_else(PoisonError::into_inner);
            *guard = entries;
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

    /// Nomes dos plugins activos, por ordem de execução (pode repetir: níveis
    /// distintos com o mesmo nome são ambos activos).
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.entries()
            .into_iter()
            .map(|(name, _origin)| name)
            .collect()
    }

    /// Plugins activos com a **origem** de cada um (M8), por ordem de execução.
    #[must_use]
    pub fn entries(&self) -> Vec<Entry> {
        self.entries
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
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

/// Resultado de uma descoberta: plugins válidos, entradas (nome + origem) e erros.
type ScanResult = (Vec<Arc<dyn Plugin>>, Vec<Entry>, Vec<String>);

/// Descobre e constrói os plugins dos directórios dados, por ordem (global primeiro).
fn scan(dirs: &[(Origin, PathBuf)]) -> ScanResult {
    let mut plugins: Vec<Arc<dyn Plugin>> = Vec::new();
    let mut entries: Vec<Entry> = Vec::new();
    let mut errors = Vec::new();
    for (origin, dir) in dirs {
        let mut children: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir())
                    .collect()
            })
            .unwrap_or_default();
        children.sort();
        for plugin_dir in children {
            match load_plugin(&plugin_dir) {
                Ok(plugin) => {
                    entries.push((plugin.name().to_owned(), *origin));
                    plugins.push(plugin);
                }
                Err(error) => errors.push(error),
            }
        }
    }
    (plugins, entries, errors)
}

/// Carrega um plugin de um directório, nomeando-o em caso de erro.
fn load_plugin(dir: &Path) -> Result<Arc<dyn Plugin>, String> {
    let manifest = Manifest::load(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let plugin = ProcessPlugin::new(&manifest, dir.to_path_buf())?;
    Ok(Arc::new(plugin))
}
