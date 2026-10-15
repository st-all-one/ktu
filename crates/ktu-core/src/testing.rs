//! Dublês determinísticos para testes: provider falso e tools falsas.
//!
//! O provider falso faz *replay* determinístico de um script de completions
//! (I6); as tools falsas devolvem respostas pré-definidas e registam as chamadas.

use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;

use crate::domain::{Completion, ToolCall, ToolSpec};
use crate::provider::{Provider, ProviderError, ProviderRequest};
use crate::tools::{ToolExecutor, ToolOutcome};

/// Provider falso: replay determinístico de um script de completions.
#[derive(Debug)]
pub struct FakeProvider {
    script: Vec<Completion>,
    cursor: Mutex<usize>,
}

impl FakeProvider {
    /// Cria um provider que devolve o script pela ordem dada.
    #[must_use]
    pub fn new(script: Vec<Completion>) -> Self {
        Self {
            script,
            cursor: Mutex::new(0),
        }
    }

    fn cursor(&self) -> MutexGuard<'_, usize> {
        self.cursor.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl Provider for FakeProvider {
    async fn complete(&self, _request: &ProviderRequest) -> Result<Completion, ProviderError> {
        let mut cursor = self.cursor();
        let index = *cursor;
        *cursor = index.saturating_add(1);
        self.script
            .get(index)
            .cloned()
            .ok_or_else(|| ProviderError::Unavailable(format!("script esgotado no passo {index}")))
    }
}

/// Tools falsas: respostas pré-definidas e registo das chamadas.
#[derive(Debug)]
pub struct FakeTools {
    specs: Vec<ToolSpec>,
    responses: Mutex<VecDeque<ToolOutcome>>,
    calls: Mutex<Vec<ToolCall>>,
}

impl FakeTools {
    /// Cria tools com especificações e respostas (consumidas por ordem).
    #[must_use]
    pub fn new(specs: Vec<ToolSpec>, responses: Vec<ToolOutcome>) -> Self {
        Self {
            specs,
            responses: Mutex::new(responses.into()),
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Devolve as chamadas registadas, por ordem.
    #[must_use]
    pub fn recorded_calls(&self) -> Vec<ToolCall> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl ToolExecutor for FakeTools {
    fn specs(&self) -> Vec<ToolSpec> {
        self.specs.clone()
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(call.clone());
        self.responses
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .unwrap_or_else(|| ToolOutcome::error("sem resposta preparada"))
    }
}
