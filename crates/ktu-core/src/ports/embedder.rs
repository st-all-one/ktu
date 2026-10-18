//! Porta do provedor de **embeddings** — o canal semântico opcional (H8).
//!
//! O canal semântico é **derivado e opcional**: sem provedor, o recall é lexical
//! (BM25 + âncoras) e tudo funciona. A identidade do provedor/modelo entra no
//! índice derivado; mudá-la **invalida** o índice (nunca se comparam vetores de
//! modelos diferentes).
//!
//! A porta é **síncrona** porque o recall do kernel ([`crate::memory::Memory`]) é
//! síncrono: o adaptador real (HTTP para um `llama-server` local) bloqueia a
//! chamada; os testes usam [`fakes::FakeEmbedder`](super::fakes::FakeEmbedder).

/// Identidade do provedor/modelo — o que **invalida** o índice derivado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddingMeta {
    /// Provedor (`http`, `fake`, `lightweight`).
    pub provider: String,
    /// Modelo (ex.: `google/embeddinggemma-2`).
    pub model: String,
    /// Revisão/commit pinado do modelo.
    pub revision: String,
    /// Dimensão dos vetores (deve casar com o modelo).
    pub dimensions: usize,
}

impl EmbeddingMeta {
    /// Constrói a identidade (dimensão `0` é normalizada para `1`).
    #[must_use]
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        revision: impl Into<String>,
        dimensions: usize,
    ) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            revision: revision.into(),
            dimensions: dimensions.max(1),
        }
    }
}

/// Erro da porta de embeddings.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EmbedError {
    /// Provedor indisponível (servidor fora, rede, processo morto).
    #[error("embeddings indisponiveis: {0}")]
    Unavailable(String),
    /// O provedor recusou o pedido (resposta inválida ou dimensão errada).
    #[error("pedido de embeddings invalido: {0}")]
    Invalid(String),
}

impl EmbedError {
    /// Cria um erro de indisponibilidade.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::Unavailable(message.into())
    }

    /// Cria um erro de pedido inválido.
    #[must_use]
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }
}

/// Porta do provedor de embeddings.
///
/// Uma falha **nunca** descarta a nota: o chamador marca-a como `pending` e tenta
/// de novo depois (degradação graciosa — o recall lexical continua).
pub trait Embedder: Send + Sync {
    /// Identidade do provedor/modelo (mudança invalida o índice derivado).
    fn meta(&self) -> &EmbeddingMeta;

    /// Embute um lote de textos, **preservando a ordem** de entrada.
    ///
    /// # Errors
    /// [`EmbedError::Unavailable`] se o provedor estiver fora; [`EmbedError::Invalid`]
    /// se a resposta não tiver um vetor por texto ou a dimensão divergir.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError>;
}
