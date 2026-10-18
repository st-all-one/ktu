//! Provider de **guião** (`--provider scripted`): replay determinístico sem modelo.
//!
//! O guião é um JSONL com uma `Completion` por linha (`ktu_core::testing::parse_script`).
//! É um fluxo do **processo**, não do turno: cada passo do modelo consome a linha
//! seguinte, mesmo entre turnos da mesma sessão — é o que permite provar uma cadeia
//! longa (`--batch --sequential`, REPL) **sem** depender de inferência.
//!
//! Determinístico por construção (I6): o mesmo guião, duas vezes ⇒ o mesmo log.
//! Em `--batch` paralelo a ordem deixa de ser determinística — use `--sequential`.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use ktu_core::domain::Completion;
use ktu_core::provider::{Provider, ProviderError, ProviderRequest};
use ktu_core::testing::parse_script;

/// Cursor do guião no processo (partilhado entre turnos).
static CURSOR: AtomicUsize = AtomicUsize::new(0);

/// Provider que devolve as completions de um guião, pela ordem do ficheiro.
#[derive(Debug)]
pub(crate) struct ScriptedProvider {
    steps: Vec<Completion>,
}

impl ScriptedProvider {
    /// Carrega o guião do caminho dado (o ficheiro é lido no arranque do turno).
    ///
    /// # Errors
    /// Devolve mensagem legível se o ficheiro não puder ser lido ou o JSONL for inválido.
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("nao foi possivel ler {}: {error}", path.display()))?;
        let steps = parse_script(&text)?;
        Ok(Self { steps })
    }
}

#[async_trait]
impl Provider for ScriptedProvider {
    async fn complete(&self, _request: &ProviderRequest) -> Result<Completion, ProviderError> {
        let index = CURSOR.fetch_add(1, Ordering::SeqCst);
        self.steps.get(index).cloned().ok_or_else(|| {
            ProviderError::Unavailable(format!(
                "guiao esgotado no passo {index} (o ficheiro tem {} passos)",
                self.steps.len()
            ))
        })
    }
}
