//! Erros do núcleo.
//!
//! Erros são **valores** (`enum Error`, `thiserror`), nunca `Box<dyn Error>` na
//! API pública. Nenhum caminho usa `unwrap`/`expect`/`panic` (I7).

use std::path::Path;

/// Erro do núcleo do ktu.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Erro de I/O, com o caminho em contexto.
    #[error("io em {path}: {source}")]
    Io {
        /// Caminho em que o erro ocorreu.
        path: String,
        /// Causa subjacente.
        #[source]
        source: std::io::Error,
    },
    /// Erro de serialização/desserialização JSON.
    #[error("json ({context}): {source}")]
    Json {
        /// Contexto do erro.
        context: String,
        /// Causa subjacente.
        #[source]
        source: serde_json::Error,
    },
    /// Entrada inválida (config, guardrails, permissões): o arranque **falha
    /// alto** em vez de degradar (I16).
    #[error("entrada invalida: {message}")]
    Invalid {
        /// Descrição legível do problema (nomeia o ficheiro quando aplicável).
        message: String,
    },
}

impl Error {
    /// Cria um erro de I/O com o caminho em contexto.
    #[must_use]
    pub fn io(path: &Path, message: impl Into<String>) -> Self {
        Self::Io {
            path: path.display().to_string(),
            source: std::io::Error::other(message.into()),
        }
    }

    /// Cria um erro JSON com contexto.
    #[must_use]
    pub fn json(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Json {
            context: context.into(),
            source,
        }
    }

    /// Cria um erro de entrada inválida (fail-closed).
    #[must_use]
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid {
            message: message.into(),
        }
    }
}

/// Resultado do núcleo, com o `Error` do crate.
pub type Result<T> = std::result::Result<T, Error>;
