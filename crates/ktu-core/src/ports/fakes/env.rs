//! *Fakes* da porta [`Env`](super::super::Env).

use std::collections::BTreeMap;

use super::super::Env;

/// Ambiente determinístico: um mapa fixo de variáveis.
#[derive(Debug, Default, Clone)]
pub struct MapEnv {
    vars: BTreeMap<String, String>,
}

impl MapEnv {
    /// Cria um ambiente vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define uma variável (construtor de teste).
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let _replaced = self.vars.insert(key.into(), value.into());
        self
    }
}

impl Env for MapEnv {
    fn var(&self, key: &str) -> Option<String> {
        self.vars.get(key).cloned()
    }
}
