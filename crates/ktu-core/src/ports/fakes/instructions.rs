//! *Fake* da porta [`Instructions`](super::super::Instructions).

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};

use super::super::{Instructions, Local};

/// Instruções locais em memória, indexadas por **prefixo de caminho**.
///
/// O prefixo mais longo ganha (é a regra do "mais próximo"); empates são
/// desfeitos pela ordem do `BTreeMap` (determinístico).
#[derive(Debug, Default)]
pub struct MapInstructions {
    entries: Mutex<BTreeMap<String, Local>>,
}

impl MapInstructions {
    /// Cria um mapa vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra as instruções aplicáveis a caminhos começados por `prefix`
    /// (`""` = qualquer caminho).
    #[must_use]
    pub fn with_dir(self, prefix: &str, source: &str, text: &str) -> Self {
        let local = Local {
            source: source.to_owned(),
            text: text.to_owned(),
        };
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(prefix.to_owned(), local);
        self
    }
}

impl Instructions for MapInstructions {
    fn nearest(&self, path: &str) -> Option<Local> {
        let entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries
            .iter()
            .filter(|(prefix, _)| path.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, local)| local.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::MapInstructions;
    use crate::ports::Instructions;

    #[test]
    fn longest_prefix_wins_and_empty_matches_everything() {
        let map = MapInstructions::new()
            .with_dir("", "AGENTS.md", "raiz")
            .with_dir("a/", "a/AGENTS.md", "modulo a")
            .with_dir("a/b/", "a/b/AGENTS.md", "submodulo");
        assert_eq!(
            map.nearest("outro.rs").map(|local| local.text),
            Some("raiz".to_owned())
        );
        assert_eq!(
            map.nearest("a/x.rs").map(|local| local.text),
            Some("modulo a".to_owned())
        );
        assert_eq!(
            map.nearest("a/b/x.rs").map(|local| local.text),
            Some("submodulo".to_owned())
        );
        assert!(MapInstructions::new().nearest("x.rs").is_none());
    }
}
