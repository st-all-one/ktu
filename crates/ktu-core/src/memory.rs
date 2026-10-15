//! Porta de memória persistente (§2.7) e o *gate* de memória do loop.
//!
//! O contrato é o do `knudge`, reduzido ao essencial: o **recall devolve
//! ponteiros** (`id` + `statement` + `score`) — nunca o corpo — e a **escrita é
//! idempotente por conteúdo** (`<0.75` cria, `0.75–0.92` funde, `≥0.92` rejeita).
//! O backend é substituível: o núcleo só conhece esta porta; o adaptador (JSONL +
//! motor `knudge-core`) vive na borda (`ktu-memory`).

/// Espécie de nota de memória (subconjunto estável do contrato do `knudge`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    /// Facto observado.
    Fact,
    /// Decisão tomada.
    Decision,
    /// Erro/debug registado.
    Error,
    /// Risco identificado.
    Risk,
    /// Pergunta em aberto.
    Question,
    /// Definição de conceito.
    Def,
    /// Snippet de código.
    Snippet,
    /// Ligação externa.
    Link,
    /// Metadado de operação.
    Meta,
}

impl MemoryKind {
    /// Todas as espécies, em ordem canónica.
    pub const ALL: [Self; 9] = [
        Self::Fact,
        Self::Decision,
        Self::Error,
        Self::Risk,
        Self::Question,
        Self::Def,
        Self::Snippet,
        Self::Link,
        Self::Meta,
    ];

    /// Texto estável (contrato de log / JSONL / CLI).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Decision => "decision",
            Self::Error => "error",
            Self::Risk => "risk",
            Self::Question => "question",
            Self::Def => "def",
            Self::Snippet => "snippet",
            Self::Link => "link",
            Self::Meta => "meta",
        }
    }

    /// Interpreta o texto estável, se conhecido.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == text)
    }
}

/// Ponteiro de recall: `id` + afirmação + `score`. **Nunca** o corpo.
#[derive(Clone, Debug, PartialEq)]
pub struct Hint {
    /// Identificador da nota (endereçado por conteúdo).
    pub id: String,
    /// Afirmação da nota (o `statement`).
    pub statement: String,
    /// Similaridade/score do recall.
    pub score: f64,
}

impl Hint {
    /// Cria um ponteiro de recall.
    #[must_use]
    pub fn new(id: impl Into<String>, statement: impl Into<String>, score: f64) -> Self {
        Self {
            id: id.into(),
            statement: statement.into(),
            score,
        }
    }
}

/// Rascunho a memorizar (campos tipados → nota).
#[derive(Clone, Debug)]
pub struct MemoryDraft {
    /// Espécie.
    pub kind: MemoryKind,
    /// Afirmação (≤ 120 escalares, uma só afirmação).
    pub statement: String,
    /// Corpo (contexto que o `statement` não diz). Pode ser vazio.
    pub body: String,
    /// Tags declaradas.
    pub tags: Vec<String>,
    /// Âncoras (paths/globs ligados ao código).
    pub anchors: Vec<String>,
}

impl MemoryDraft {
    /// Rascunho com espécie e afirmação.
    #[must_use]
    pub fn new(kind: MemoryKind, statement: impl Into<String>) -> Self {
        Self {
            kind,
            statement: statement.into(),
            body: String::new(),
            tags: Vec::new(),
            anchors: Vec::new(),
        }
    }
}

/// Desfecho idempotente de uma escrita por conteúdo.
#[derive(Clone, Debug, PartialEq)]
pub enum MemoryWrite {
    /// Nota nova (nada semelhante).
    Created {
        /// Id criado.
        id: String,
    },
    /// Fundida numa nota existente.
    Merged {
        /// Id mantido.
        id: String,
        /// Similaridade.
        score: f64,
    },
    /// Duplicata forte: rejeitada.
    Rejected {
        /// Id existente.
        id: String,
        /// Similaridade.
        score: f64,
    },
    /// Retry idempotente: nada mudou.
    Unchanged {
        /// Id existente.
        id: String,
    },
}

impl MemoryWrite {
    /// Id envolvido.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Created { id }
            | Self::Merged { id, .. }
            | Self::Rejected { id, .. }
            | Self::Unchanged { id } => id,
        }
    }

    /// Rótulo estável da acção (`created`/`merged`/`rejected`/`unchanged`).
    #[must_use]
    pub const fn action(&self) -> &'static str {
        match self {
            Self::Created { .. } => "created",
            Self::Merged { .. } => "merged",
            Self::Rejected { .. } => "rejected",
            Self::Unchanged { .. } => "unchanged",
        }
    }
}

/// Erros da porta de memória (distintos dos do núcleo: schema/indisponibilidade).
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MemoryError {
    /// Erro de I/O, com o caminho em contexto.
    #[error("io em {path}: {source}")]
    Io {
        /// Caminho em que o erro ocorreu.
        path: String,
        /// Causa subjacente.
        #[source]
        source: std::io::Error,
    },
    /// Violação de schema (afirmação vazia/longa, espécie inválida).
    #[error("schema de memoria: {message}")]
    Schema {
        /// Detalhe do erro.
        message: String,
    },
    /// Backend indisponível (degradação graciosa do chamador).
    #[error("memoria indisponivel: {message}")]
    Unavailable {
        /// Detalhe do erro.
        message: String,
    },
}

impl MemoryError {
    /// Cria um erro de schema.
    #[must_use]
    pub fn schema(message: impl Into<String>) -> Self {
        Self::Schema {
            message: message.into(),
        }
    }

    /// Cria um erro de indisponibilidade.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::Unavailable {
            message: message.into(),
        }
    }
}

/// Resultado da porta de memória.
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Porta de memória (backend substituível — JSONL + `knudge-core` hoje).
pub trait Memory: Send + Sync {
    /// Recupera ponteiros relevantes para `query` (máximo `limit`).
    ///
    /// # Errors
    /// Devolve [`MemoryError`] se o backend falhar.
    fn recall(&self, query: &str, limit: usize) -> MemoryResult<Vec<Hint>>;

    /// Memoriza um rascunho, aplicando o protocolo idempotente de dedup.
    ///
    /// # Errors
    /// Devolve [`MemoryError`] se o rascunho for inválido ou o backend falhar.
    fn write(&self, draft: &MemoryDraft) -> MemoryResult<MemoryWrite>;
}

/// Formata os *hints* como bloco de contexto — **ponteiros**, nunca o corpo.
#[must_use]
pub fn compose_hints(hints: &[Hint]) -> String {
    use std::fmt::Write as _;
    if hints.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "Conhecimento relevante da memoria (ponteiros; use memory_ask para o corpo):\n",
    );
    for hint in hints {
        out.push_str("- ");
        out.push_str(&hint.id);
        out.push_str(": ");
        out.push_str(&hint.statement);
        let _ignored = writeln!(out, " (score {:.3})", hint.score);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Hint, MemoryKind, MemoryWrite, compose_hints};

    #[test]
    fn kind_round_trips_through_text() {
        for kind in MemoryKind::ALL {
            assert_eq!(MemoryKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(MemoryKind::parse("nope"), None);
    }

    #[test]
    fn write_reports_id_and_action() {
        let write = MemoryWrite::Merged {
            id: "fact_01".to_owned(),
            score: 0.8,
        };
        assert_eq!(write.id(), "fact_01");
        assert_eq!(write.action(), "merged");
    }

    #[test]
    fn hints_are_pointers_only() {
        let hints = vec![Hint::new("fact_01", "a memoria vive em jsonl", 0.9)];
        let block = compose_hints(&hints);
        assert!(block.contains("fact_01"));
        assert!(block.contains("a memoria vive em jsonl"));
        assert!(!compose_hints(&[]).contains("memoria"));
    }
}
