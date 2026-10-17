//! Manifesto `plugin.toml` de um plugin de script (§2.9).

use std::collections::BTreeSet;
use std::path::Path;

use ktu_core::plugin::{HookEvent, Mode};
use serde::Deserialize;

/// Manifesto declarado em `.ktu/plugins/<nome>/plugin.toml`.
#[derive(Clone, Debug, Deserialize)]
pub struct Manifest {
    /// Nome estável do plugin.
    pub name: String,
    /// Linha de comando do *entrypoint* (relativa ao directório do plugin).
    pub entrypoint: String,
    /// Eventos subscritos (nomes estáveis de [`HookEvent`]).
    pub events: Vec<String>,
    /// Modo (`observe|transform|gate`).
    #[serde(default = "default_mode")]
    pub mode: String,
    /// *Timeout* por chamada, em milissegundos.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Se é um daemon de longa duração (informativo nesta fase).
    #[serde(default)]
    pub daemon: bool,
}

fn default_mode() -> String {
    "observe".to_owned()
}

const fn default_timeout() -> u64 {
    2000
}

impl Manifest {
    /// Interpreta um manifesto TOML.
    ///
    /// # Errors
    /// Devolve mensagem se o TOML for inválido.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| format!("plugin.toml invalido: {error}"))
    }

    /// Carrega o manifesto de `<dir>/plugin.toml`.
    ///
    /// # Errors
    /// Devolve mensagem se o ficheiro não puder ser lido ou interpretado.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let path = dir.join("plugin.toml");
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Self::parse(&text)
    }

    /// Eventos tipados subscritos.
    ///
    /// # Errors
    /// Devolve mensagem se algum nome de evento for desconhecido.
    pub fn events(&self) -> Result<BTreeSet<HookEvent>, String> {
        let mut set = BTreeSet::new();
        for name in &self.events {
            let event =
                HookEvent::from_name(name).ok_or_else(|| format!("evento desconhecido: {name}"))?;
            let _inserted = set.insert(event);
        }
        Ok(set)
    }

    /// Modo tipado.
    ///
    /// # Errors
    /// Devolve mensagem se o modo for desconhecido.
    pub fn mode(&self) -> Result<Mode, String> {
        match self.mode.as_str() {
            "observe" => Ok(Mode::Observe),
            "transform" => Ok(Mode::Transform),
            "gate" => Ok(Mode::Gate),
            other => Err(format!("modo desconhecido: {other}")),
        }
    }
}
