//! Adaptador da porta `Instructions`: o `AGENTS.md` **mais próximo** de um ficheiro.
//!
//! Quando o modelo lê `sub/x.md`, as instruções de `sub/` (e, na falta, as dos
//! pais até à raiz) são anexadas ao resultado — uma vez por turno e por fonte.
//! É leitura pela porta `Fs`, com a resolução pura de
//! [`agents_md::nearest`].

use std::path::PathBuf;

use ktu_core::agents_md;
use ktu_core::ports::{Instructions, Local};
use ktu_core::prompt::condense;
use ktu_tools::StdFs;

/// Instruções locais do projecto.
pub(crate) struct LocalInstructions {
    root: PathBuf,
}

impl LocalInstructions {
    /// Cria o adaptador ligado à raiz do projecto.
    pub(crate) fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Instructions for LocalInstructions {
    fn nearest(&self, path: &str) -> Option<Local> {
        let fs = StdFs;
        let doc = agents_md::nearest(&fs, &self.root, &self.root.join(path))?;
        let text = condense(&doc.text);
        if text.is_empty() {
            return None;
        }
        Some(Local {
            source: agents_md::label(&self.root, &doc.path),
            text,
        })
    }
}
