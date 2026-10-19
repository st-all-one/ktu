//! Configuração em **dois níveis** (global + local) com precedência declarada.
//!
//! O **projecto é a autoridade** (M6): a config local ganha **chave a chave** sobre
//! a global. A resolução é **pura** (global + local → efectiva); a leitura e a
//! escrita de ficheiros ficam na borda. As credenciais vivem **só** no global
//! (`auth.toml`, 0600) e nunca entram no log nem no contexto do modelo.
//!
//! # Schema e migrações (F5)
//!
//! O documento declara a sua versão (`schema = 1`); a **ausência** significa **v0**
//! (o formato anterior, sem versão). A migração tem duas variantes de propósito:
//!
//! - [`read_migrate`] — **leitura**: devolve o documento na versão actual **em
//!   memória** e **nunca remove chaves** (outro chamador pode ainda consultá-las);
//! - [`migrate`] — **escrita**: o documento a persistir; pode **remover** as chaves
//!   que um passo declare obsoletas (hoje nenhuma) e diz se mudou algo
//!   (idempotência: correr duas vezes ⇒ `false`).
//!
//! Uma versão **futura** (`schema = 3` num binário v2) é **fail-closed**: erro
//! legível e nenhuma escrita — nunca se "migra para trás" sobre um ficheiro que não
//! se sabe ler.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use toml::Table;
use toml::Value;

/// Nome do ficheiro de configuração de comportamento.
pub const CONFIG_FILE: &str = "config.toml";
/// Nome do ficheiro de credenciais (só global).
pub const AUTH_FILE: &str = "auth.toml";
/// Nome do subdirectório de configuração global.
pub const GLOBAL_DIR: &str = "ktu";

/// Versão **actual** do schema de configuração.
pub const SCHEMA: u32 = 1;
/// Chave que declara a versão do schema no TOML.
pub const SCHEMA_KEY: &str = "schema";
/// Chave cujo formato mudou no v1 (string CSV → lista).
const DEFERRED_TOOLS: &str = "deferred_tools";

/// Chaves conhecidas do config, por ordem estável.
pub const KEYS: &[&str] = &[
    "provider",
    "base_url",
    "model",
    "effort",
    "max_steps",
    "deferred_tools",
];

/// Configuração de comportamento (chaves **fechadas**: `deny_unknown_fields`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Versão do schema declarada no ficheiro (`None` ⇒ v0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<u32>,
    /// Provider (`llama`|`opencode-go`|`mock`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// *Base URL* do provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Nome/alias do modelo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Esforço de raciocínio (`low|medium|high|xhigh`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    /// Tecto de passos do turno.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    /// Tools **diferidas**: fora do catálogo do modelo até `tool_search` as carregar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deferred_tools: Option<Vec<String>>,
}

impl Config {
    /// Interpreta um `config.toml` (chaves fechadas).
    ///
    /// # Errors
    /// Devolve mensagem legível se o TOML for inválido ou tiver chaves desconhecidas.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| format!("config invalida: {error}"))
    }

    /// Renderiza como TOML (omite as chaves não definidas).
    ///
    /// # Errors
    /// Devolve mensagem legível se a serialização falhar.
    pub fn render(&self) -> Result<String, String> {
        toml::to_string(self).map_err(|error| format!("config nao serializavel: {error}"))
    }

    /// Interpreta uma tabela TOML **já migrada** (chaves fechadas).
    ///
    /// # Errors
    /// Devolve mensagem legível se uma chave for desconhecida ou tiver o tipo errado.
    pub fn from_table(table: Table) -> Result<Self, String> {
        Value::Table(table)
            .try_into()
            .map_err(|error| format!("config invalida: {error}"))
    }

    /// Funde `local` sobre `global`, chave a chave (o projecto ganha — M6).
    #[must_use]
    pub fn merge(local: Self, global: Self) -> Self {
        Self {
            schema: local.schema.or(global.schema),
            provider: local.provider.or(global.provider),
            base_url: local.base_url.or(global.base_url),
            model: local.model.or(global.model),
            effort: local.effort.or(global.effort),
            max_steps: local.max_steps.or(global.max_steps),
            deferred_tools: local.deferred_tools.or(global.deferred_tools),
        }
    }

    /// Define uma chave (valida o tipo).
    ///
    /// # Errors
    /// Devolve mensagem legível se a chave for desconhecida ou o valor inválido.
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "provider" => self.provider = Some(value.to_owned()),
            "base_url" => self.base_url = Some(value.to_owned()),
            "model" => self.model = Some(value.to_owned()),
            "effort" => self.effort = Some(value.to_owned()),
            "max_steps" => {
                self.max_steps = Some(
                    value
                        .parse()
                        .map_err(|_| format!("max_steps nao e um inteiro: {value}"))?,
                );
            }
            "deferred_tools" => {
                self.deferred_tools = Some(
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .map(str::to_owned)
                        .collect(),
                );
            }
            other => return Err(format!("chave desconhecida: {other}")),
        }
        Ok(())
    }

    /// Remove uma chave.
    ///
    /// # Errors
    /// Devolve mensagem legível se a chave for desconhecida.
    pub fn unset(&mut self, key: &str) -> Result<(), String> {
        match key {
            "provider" => self.provider = None,
            "base_url" => self.base_url = None,
            "model" => self.model = None,
            "effort" => self.effort = None,
            "max_steps" => self.max_steps = None,
            "deferred_tools" => self.deferred_tools = None,
            other => return Err(format!("chave desconhecida: {other}")),
        }
        Ok(())
    }

    /// Valor de uma chave como texto, se definida.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "provider" => self.provider.clone(),
            "base_url" => self.base_url.clone(),
            "model" => self.model.clone(),
            "effort" => self.effort.clone(),
            "max_steps" => self.max_steps.map(|value| value.to_string()),
            "deferred_tools" => self.deferred_tools.as_ref().map(|tools| tools.join(",")),
            _ => None,
        }
    }

    /// Chaves definidas, por ordem estável.
    #[must_use]
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        KEYS.iter()
            .filter_map(|key| self.get(key).map(|value| (*key, value)))
            .collect()
    }
}

pub use migrate::{migrate, read_migrate, schema_of, table};

mod migrate;

#[cfg(test)]
mod tests;

/// Credenciais de providers (fonte: `auth.toml`, só global, 0600).
///
/// O `Debug` é **redigido** (M3): nunca imprime as chaves.
#[derive(Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Auth {
    /// Chaves de API por provider.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub providers: BTreeMap<String, String>,
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&String> = self.providers.keys().collect();
        f.debug_struct("Auth").field("providers", &names).finish()
    }
}

impl Auth {
    /// Interpreta um `auth.toml` (chaves fechadas).
    ///
    /// # Errors
    /// Devolve mensagem legível se o TOML for inválido.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| format!("auth invalida: {error}"))
    }

    /// Chave de API de `provider`, se existir.
    #[must_use]
    pub fn api_key(&self, provider: &str) -> Option<&str> {
        self.providers.get(provider).map(String::as_str)
    }
}

/// Directório de configuração global: `$XDG_CONFIG_HOME/ktu` ou `$HOME/.config/ktu`.
#[must_use]
pub fn global_dir(xdg: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    if let Some(xdg) = xdg.filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(xdg).join(GLOBAL_DIR));
    }
    home.filter(|value| !value.is_empty())
        .map(|home| PathBuf::from(home).join(".config").join(GLOBAL_DIR))
}

/// Caminho do `config.toml` global.
#[must_use]
pub fn global_config_path(xdg: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    global_dir(xdg, home).map(|dir| dir.join(CONFIG_FILE))
}

/// Caminho do `auth.toml` global.
#[must_use]
pub fn global_auth_path(xdg: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    global_dir(xdg, home).map(|dir| dir.join(AUTH_FILE))
}

/// Caminho do `config.toml` local do projecto.
#[must_use]
pub fn local_config_path(root: &Path) -> PathBuf {
    root.join(".agents").join(CONFIG_FILE)
}
