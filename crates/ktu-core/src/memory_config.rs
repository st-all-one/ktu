//! Configuração da **memória** — `<project>/.agents/memory.toml`.
//!
//! O motor (`.agents/memory/notes.jsonl`) é a fonte da verdade; esta configuração
//! diz apenas **como** o canal semântico é servido (H8). As chaves `embeddings.*`
//! têm os **mesmos nomes** do knudge: as duas vistas projectam a mesma verdade e
//! nunca divergem.
//!
//! O *schema* é **fechado** (`deny_unknown_fields`) e o canal semântico é
//! **opcional**: por omissão `provider = "none"` e o recall é lexical.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Nome do ficheiro de configuração da memória.
pub const MEMORY_FILE: &str = "memory.toml";

/// Endpoint por omissão do servidor local de embeddings (`llama-server`).
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8889/v1/embeddings";
/// Modelo por omissão (o **mesmo** do knudge).
pub const DEFAULT_MODEL: &str = "google/embeddinggemma-2";
/// Revisão por omissão (pinada pelo worker).
pub const DEFAULT_REVISION: &str = "main";
/// Dimensão por omissão.
pub const DEFAULT_DIMENSIONS: usize = 768;
/// Prefixo por omissão (`auto` resolve pelo nome do modelo).
pub const DEFAULT_PREFIX: &str = "auto";
/// Tamanho de lote por omissão do dreno.
pub const DEFAULT_BATCH: usize = 32;

/// Provedor do canal semântico.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    /// Sem canal semântico (recall lexical; **desliga sem aviso**).
    #[default]
    None,
    /// Servidor HTTP OpenAI-compatible (`llama-server` local, porta 8889).
    Http,
}

impl EmbeddingProvider {
    /// Todos os provedores, em ordem canónica.
    pub const ALL: [Self; 2] = [Self::None, Self::Http];

    /// Rótulo estável (config/CLI).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Http => "http",
        }
    }

    /// Interpreta o rótulo.
    ///
    /// # Errors
    /// Devolve mensagem legível para valor desconhecido.
    pub fn parse(text: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|provider| provider.as_str() == text)
            .ok_or_else(|| format!("embeddings.provider invalido: {text} (use none|http)"))
    }
}

/// Ajustes do canal semântico (`[embeddings]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Embeddings {
    /// Provedor (`none` desliga).
    pub provider: EmbeddingProvider,
    /// Endpoint OpenAI-compatible do servidor de embeddings.
    pub endpoint: String,
    /// Modelo (entra na identidade do índice).
    pub model: String,
    /// Revisão do modelo (entra na identidade do índice).
    pub revision: String,
    /// Dimensão dos vetores (entra na identidade do índice).
    pub dimensions: usize,
    /// Prefixo de instrução de tarefa (`auto|none|embeddinggemma`).
    pub prefix: String,
    /// Textos por lote no dreno.
    pub batch: usize,
}

impl Default for Embeddings {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::None,
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            model: DEFAULT_MODEL.to_owned(),
            revision: DEFAULT_REVISION.to_owned(),
            dimensions: DEFAULT_DIMENSIONS,
            prefix: DEFAULT_PREFIX.to_owned(),
            batch: DEFAULT_BATCH,
        }
    }
}

impl Embeddings {
    /// `true` se há um provedor configurado (o canal semântico pode ser usado).
    #[must_use]
    pub const fn enabled(&self) -> bool {
        !matches!(self.provider, EmbeddingProvider::None)
    }

    /// Lote efectivo (nunca `0`).
    #[must_use]
    pub const fn batch(&self) -> usize {
        if self.batch == 0 {
            DEFAULT_BATCH
        } else {
            self.batch
        }
    }
}

/// Configuração da memória (raiz do `memory.toml`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemoryConfig {
    /// Ajustes do canal semântico.
    pub embeddings: Embeddings,
}

impl MemoryConfig {
    /// Caminho do `memory.toml` para um *root* de projecto.
    #[must_use]
    pub fn path(root: &Path) -> PathBuf {
        root.join(".agents").join(MEMORY_FILE)
    }

    /// Interpreta um `memory.toml` (chaves fechadas).
    ///
    /// # Errors
    /// Devolve mensagem legível se o TOML for inválido ou tiver chaves desconhecidas.
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| format!("memory.toml invalido: {error}"))
    }

    /// Renderiza como TOML.
    ///
    /// # Errors
    /// Devolve mensagem legível se a serialização falhar.
    pub fn render(&self) -> Result<String, String> {
        toml::to_string(self).map_err(|error| format!("memory.toml nao serializavel: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_BATCH, DEFAULT_DIMENSIONS, EmbeddingProvider, Embeddings, MemoryConfig};

    #[test]
    fn default_is_lexical_and_compatible_with_the_knudge() {
        let config = MemoryConfig::default();
        assert_eq!(config.embeddings.provider, EmbeddingProvider::None);
        assert!(!config.embeddings.enabled());
        assert_eq!(config.embeddings.model, "google/embeddinggemma-2");
        assert_eq!(config.embeddings.dimensions, DEFAULT_DIMENSIONS);
        assert_eq!(config.embeddings.batch(), DEFAULT_BATCH);
    }

    #[test]
    fn empty_document_parses_to_defaults() -> Result<(), String> {
        let config = MemoryConfig::parse("")?;
        assert_eq!(config, MemoryConfig::default());
        let config = MemoryConfig::parse("[embeddings]\nprovider = \"http\"\n")?;
        assert!(config.embeddings.enabled());
        assert_eq!(config.embeddings.endpoint, super::DEFAULT_ENDPOINT);
        Ok(())
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(MemoryConfig::parse("[embeddings]\nnope = 1\n").is_err());
        assert!(MemoryConfig::parse("[nope]\n").is_err());
        assert!(MemoryConfig::parse("[embeddings]\nprovider = \"wat\"\n").is_err());
    }

    #[test]
    fn render_round_trips() -> Result<(), String> {
        let mut config = MemoryConfig::default();
        config.embeddings.provider = EmbeddingProvider::Http;
        config.embeddings.batch = 0;
        let text = config.render()?;
        assert_eq!(MemoryConfig::parse(&text)?, config);
        assert_eq!(
            config.embeddings.batch(),
            DEFAULT_BATCH,
            "lote 0 volta ao default"
        );
        Ok(())
    }

    #[test]
    fn provider_labels_are_canonical() -> Result<(), String> {
        assert_eq!(EmbeddingProvider::None.as_str(), "none");
        assert_eq!(EmbeddingProvider::Http.as_str(), "http");
        assert_eq!(EmbeddingProvider::parse("http")?, EmbeddingProvider::Http);
        assert!(EmbeddingProvider::parse("nope").is_err());
        assert_eq!(Embeddings::default().provider, EmbeddingProvider::None);
        Ok(())
    }
}
