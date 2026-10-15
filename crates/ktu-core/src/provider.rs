//! Porta do provider (o modelo) e tipos associados.
//!
//! O ktu só tem um provider *built-in*: llama.cpp. Outros entram pela GDK, atrás
//! da feature `gdk` (§2.5). O loop fala apenas desta porta.

use async_trait::async_trait;

use crate::cancel::CancelToken;
use crate::domain::{Completion, Message, ToolSpec};

/// Pedido ao modelo, já na projecção de *wire*.
#[derive(Clone, Debug)]
pub struct ProviderRequest {
    /// System prompt composto.
    pub system: String,
    /// Mensagens derivadas do log (I3).
    pub messages: Vec<Message>,
    /// Tools oferecidas neste pedido (função do `AgentSpec`).
    pub tools: Vec<ToolSpec>,
    /// Sinal observado durante o *streaming* (cancelamento a meio do passo).
    pub cancel: CancelToken,
}

/// Erro da porta do provider.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProviderError {
    /// Provider indisponível (processo, rede).
    #[error("provider indisponivel: {0}")]
    Unavailable(String),
    /// O provider recusou o pedido.
    #[error("pedido invalido: {0}")]
    Invalid(String),
    /// Cancelado a meio do passo (o loop fecha com `StopReason::Cancelled`).
    #[error("cancelado")]
    Cancelled,
}

/// Porta do provider: uma chamada devolve texto e/ou chamadas de tool.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Completa um pedido.
    async fn complete(&self, request: &ProviderRequest) -> Result<Completion, ProviderError>;
}
