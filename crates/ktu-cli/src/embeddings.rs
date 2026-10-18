//! Ligação do canal semântico à memória (H8).
//!
//! Lê `<project>/.agents/memory.toml` e constrói o adaptador da porta
//! [`ktu_core::ports::Embedder`]. Sem `memory.toml` (ou com `provider = "none"`),
//! devolve `None` e o recall é lexical: a degradação é **silenciosa**, por desenho.

use std::path::Path;

use ktu_core::memory_config::{EmbeddingProvider, Embeddings, MemoryConfig};
use ktu_core::ports::Fs;
use ktu_provider::HttpEmbedder;

/// Lê a configuração da memória (ausente ⇒ *defaults*: canal desligado).
///
/// # Errors
/// Devolve mensagem legível se o ficheiro não for legível ou violar o schema.
pub(crate) fn load_config(fs: &dyn Fs, root: &Path) -> Result<MemoryConfig, String> {
    let path = MemoryConfig::path(root);
    if !fs.exists(&path) {
        return Ok(MemoryConfig::default());
    }
    let bytes = fs
        .read(&path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{} nao e UTF-8", path.display()))?;
    MemoryConfig::parse(&text)
}

/// Constrói o adaptador do canal semântico (`None` se o provedor está desligado).
///
/// # Errors
/// Devolve mensagem legível se o endpoint for inválido ou o cliente não arrancar.
pub(crate) fn build(config: &Embeddings) -> Result<Option<HttpEmbedder>, String> {
    match config.provider {
        EmbeddingProvider::None => Ok(None),
        EmbeddingProvider::Http => HttpEmbedder::from_config(config)
            .map(Some)
            .map_err(|error| error.to_string()),
    }
}
