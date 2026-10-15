//! Porta de execução de tools (corre na thread de tools, T2 — §2.2).
//!
//! A thread de tools conhece os efeitos, nunca a sessão. Nenhuma tool corre sem
//! `Guard::evaluate` em `Allow` (I10); erros viram [`ToolOutcome::error`], nunca
//! pânico.

use async_trait::async_trait;

use crate::domain::{ToolCall, ToolSpec};

/// Resultado de uma tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolOutcome {
    /// Conteúdo do resultado.
    pub content: String,
    /// Se o resultado é um erro (a tool falhou, não o turno).
    pub is_error: bool,
}

impl ToolOutcome {
    /// Resultado bem-sucedido.
    #[must_use]
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
        }
    }

    /// Resultado de erro.
    #[must_use]
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
        }
    }
}

/// Porta de execução de tools.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Tools oferecidas ao modelo (função do `AgentSpec` em execução).
    fn specs(&self) -> Vec<ToolSpec>;

    /// Executa uma chamada. Erros viram [`ToolOutcome::error`], nunca pânico.
    async fn execute(&self, call: &ToolCall) -> ToolOutcome;
}
