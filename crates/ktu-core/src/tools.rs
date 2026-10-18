//! Porta de execução de tools (corre na thread de tools, T2 — §2.2).
//!
//! A thread de tools conhece os efeitos, nunca a sessão. Nenhuma tool corre sem
//! `Guard::evaluate` em `Allow` (I10); erros viram [`ToolOutcome::error`], nunca
//! pânico.

use async_trait::async_trait;

use std::collections::BTreeSet;

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
    /// Tools **declaradas** ao modelo (função do `AgentSpec` em execução).
    fn specs(&self) -> Vec<ToolSpec>;

    /// Tools **diferidas**: existem e são executáveis, mas não vão no catálogo até
    /// o modelo as pedir por [`tool_search`](crate::tool_search).
    ///
    /// Por omissão, nenhuma (todo o catálogo é declarado).
    fn deferred(&self) -> Vec<ToolSpec> {
        Vec::new()
    }

    /// Executa uma chamada. Erros viram [`ToolOutcome::error`], nunca pânico.
    async fn execute(&self, call: &ToolCall) -> ToolOutcome;
}

/// Visão **filtrada** de um [`ToolExecutor`] pelo conjunto de tools de um agente.
///
/// Segunda camada de defesa (§2.6): mesmo que o modelo alucine uma tool fora do
/// `AgentSpec`, a execução é recusada por capacidade. O `specs()` só oferece o
/// que o conjunto declara.
pub struct ScopedTools<'a> {
    inner: &'a dyn ToolExecutor,
    allowed: BTreeSet<String>,
}

impl<'a> ScopedTools<'a> {
    /// Restringe um executor a um conjunto de nomes de tool.
    #[must_use]
    pub fn new(inner: &'a dyn ToolExecutor, allowed: impl IntoIterator<Item = String>) -> Self {
        Self {
            inner,
            allowed: allowed.into_iter().collect(),
        }
    }
}

#[async_trait]
impl ToolExecutor for ScopedTools<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        self.inner
            .specs()
            .into_iter()
            .filter(|spec| self.allowed.contains(&spec.name))
            .collect()
    }

    fn deferred(&self) -> Vec<ToolSpec> {
        self.inner
            .deferred()
            .into_iter()
            .filter(|spec| self.allowed.contains(&spec.name))
            .collect()
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        if !self.allowed.contains(&call.name) {
            return ToolOutcome::error(format!("negado: capability ({})", call.name));
        }
        self.inner.execute(call).await
    }
}

#[cfg(test)]
mod tests {
    use super::{ScopedTools, ToolExecutor};
    use crate::domain::{ToolCall, ToolCallId, ToolSpec};
    use crate::testing::FakeTools;
    use crate::tools::ToolOutcome;

    fn spec(name: &str) -> ToolSpec {
        ToolSpec {
            name: name.to_owned(),
            description: String::new(),
            parameters: "{}".to_owned(),
        }
    }

    fn call(name: &str) -> ToolCall {
        ToolCall {
            id: ToolCallId::new("1"),
            name: name.to_owned(),
            arguments: "{}".to_owned(),
        }
    }

    #[tokio::test]
    async fn scoped_tools_filters_specs_and_denies_off_spec() {
        let inner = FakeTools::new(
            vec![spec("read_file"), spec("write_file")],
            vec![ToolOutcome::ok("lido")],
        );
        let scoped = ScopedTools::new(&inner, ["read_file".to_owned()]);
        assert_eq!(scoped.specs().len(), 1);

        let denied = scoped.execute(&call("write_file")).await;
        assert!(denied.is_error);
        assert_eq!(inner.recorded_calls().len(), 0);

        let allowed = scoped.execute(&call("read_file")).await;
        assert!(!allowed.is_error);
        assert_eq!(inner.recorded_calls().len(), 1);
    }
}
