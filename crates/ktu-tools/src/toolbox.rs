//! Agregador de ferramentas: um único [`ToolExecutor`] que despacha por nome.
//!
//! O loop fala de **uma** porta de tools; aqui juntam-se as de *filesystem* e a
//! de execução (`bash`).

use std::collections::BTreeSet;
use std::path::PathBuf;

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::memory::Memory;
use ktu_core::plan::{self, FsPlanHost};
use ktu_core::ports::{Clock, Fs};
use ktu_core::tools::{ToolExecutor, ToolOutcome};

use crate::bash::BashTool;
use crate::fs_tools::FsTools;
use crate::memory_tools::MemoryTools;
use crate::mutation::FileMutationQueue;
use crate::trash::TrashTool;

/// Conjunto de ferramentas da sessão.
pub struct Toolbox<'a> {
    fs: FsTools<'a>,
    bash: BashTool,
    trash: TrashTool<'a>,
    memory: Option<MemoryTools<'a>>,
    plan: Option<FsPlanHost<'a>>,
    deferred: BTreeSet<String>,
}

impl<'a> Toolbox<'a> {
    /// Cria o conjunto com o *root*, a porta `Fs`, a porta `Clock` e a memória
    /// (opcional: sem backend, `memory_ask`/`memory_write` não são oferecidas).
    #[must_use]
    pub fn new(
        root: impl Into<PathBuf>,
        fs: &'a dyn Fs,
        clock: &'a dyn Clock,
        memory: Option<&'a dyn Memory>,
        mutations: &'a FileMutationQueue,
    ) -> Self {
        let root = root.into();
        Self {
            fs: FsTools::new(root.clone(), fs, mutations),
            bash: BashTool::new(root.clone()),
            trash: TrashTool::new(root, fs, clock),
            memory: memory.map(MemoryTools::new),
            plan: None,
            deferred: BTreeSet::new(),
        }
    }

    /// Marca tools como **diferidas**: existem e executam, mas não vão no catálogo
    /// do modelo até `tool_search` as carregar (dado de `.agents/config.toml`).
    #[must_use]
    pub fn with_deferred(mut self, names: impl IntoIterator<Item = String>) -> Self {
        self.deferred = names.into_iter().collect();
        self
    }

    /// Liga a tool `plan` (modo plano) a um host persistente sobre `Fs`.
    #[must_use]
    pub fn with_plan(mut self, host: FsPlanHost<'a>) -> Self {
        self.plan = Some(host);
        self
    }
}

impl Toolbox<'_> {
    /// Todas as especificações do conjunto (declaradas e diferidas).
    fn all_specs(&self) -> Vec<ToolSpec> {
        let mut specs = self.fs.specs();
        specs.push(ToolSpec {
            name: "bash".to_owned(),
            description: "Executa um comando de shell no root da sessao (com timeout).".to_owned(),
            parameters:
                r#"{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}"#
                    .to_owned(),
        });
        specs.extend(self.trash.specs());
        if let Some(memory) = &self.memory {
            specs.extend(memory.specs());
        }
        if self.plan.is_some() {
            specs.push(plan::spec());
        }
        specs
    }
}

#[async_trait]
impl ToolExecutor for Toolbox<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        self.all_specs()
            .into_iter()
            .filter(|spec| !self.deferred.contains(&spec.name))
            .collect()
    }

    fn deferred(&self) -> Vec<ToolSpec> {
        self.all_specs()
            .into_iter()
            .filter(|spec| self.deferred.contains(&spec.name))
            .collect()
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        match call.name.as_str() {
            "bash" => self.bash.execute(call).await,
            "trash_file" => self.trash.execute(call).await,
            "memory_ask" | "memory_write" => match &self.memory {
                Some(memory) => memory.execute(call).await,
                None => ToolOutcome::error("memoria indisponivel nesta sessao"),
            },
            "plan" => match &self.plan {
                Some(plan) => plan.execute(&call.arguments),
                None => ToolOutcome::error("tool 'plan' indisponivel fora do modo plano"),
            },
            _ => self.fs.execute(call).await,
        }
    }
}
