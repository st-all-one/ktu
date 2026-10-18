//! Especificação de agente e registo (§2.6).
//!
//! O **conjunto de tools** é o enforcement estrutural do read-only: o loop só
//! oferece ao modelo o que o `AgentSpec` declara, e a thread de tools rejeita
//! por capacidade qualquer tool fora do conjunto (defesa em dupla camada).

use std::collections::{BTreeMap, BTreeSet};

use crate::guard::AgentId;

/// Conjunto de capacidades de um agente.
///
/// `max_depth` é a profundidade **máxima de subagentes** que este agente pode
/// criar quando corre: o primário cria filhos (`1`), o `explorer` não cria
/// nenhum (`0`) — é assim que a profundidade máxima 1 fica travada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentSpec {
    id: AgentId,
    tools: BTreeSet<String>,
    max_depth: u8,
}

impl AgentSpec {
    /// Cria uma especificação com o conjunto de tools permitidas.
    #[must_use]
    pub fn new(id: AgentId, tools: impl IntoIterator<Item = String>, max_depth: u8) -> Self {
        Self {
            id,
            tools: tools.into_iter().collect(),
            max_depth,
        }
    }

    /// Identificador do agente.
    #[must_use]
    pub const fn id(&self) -> &AgentId {
        &self.id
    }

    /// Profundidade máxima de subagentes permitida.
    #[must_use]
    pub const fn max_depth(&self) -> u8 {
        self.max_depth
    }

    /// Indica se a tool pertence ao conjunto do agente.
    #[must_use]
    pub fn allows(&self, tool: &str) -> bool {
        self.tools.contains(tool)
    }

    /// Nomes das tools, em ordem determinística.
    #[must_use]
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.iter().cloned().collect()
    }
}

/// Registo de agentes disponíveis (primário sempre presente; `explorer` opcional;
/// agentes **declarativos** do projecto por nome).
#[derive(Clone, Debug)]
pub struct AgentRegistry {
    primary: AgentSpec,
    explorer: Option<AgentSpec>,
    named: BTreeMap<String, AgentSpec>,
}

impl AgentRegistry {
    /// Cria o registo com o agente primário e, opcionalmente, o `explorer`.
    #[must_use]
    pub fn new(primary: AgentSpec, explorer: Option<AgentSpec>) -> Self {
        Self {
            primary,
            explorer,
            named: BTreeMap::new(),
        }
    }

    /// Acrescenta agentes declarativos (`.agents/agents/*.md`), por nome.
    ///
    /// Não substitui o primário: para isso existe [`AgentRegistry::with_primary`].
    #[must_use]
    pub fn with_named(mut self, named: BTreeMap<String, AgentSpec>) -> Self {
        self.named = named;
        self
    }

    /// Substitui a especificação do agente primário (agente declarativo `primary`).
    #[must_use]
    pub fn with_primary(mut self, primary: AgentSpec) -> Self {
        self.primary = primary;
        self
    }

    /// Agentes declarativos, por ordem determinística de nome.
    #[must_use]
    pub fn named(&self) -> &BTreeMap<String, AgentSpec> {
        &self.named
    }

    /// Nomes dos subagentes disponíveis (`explorer` + declarativos), por ordem.
    #[must_use]
    pub fn subagent_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.named.keys().cloned().collect();
        if self.explorer.is_some() {
            names.push(AgentId::Explorer.as_str().to_owned());
        }
        names.sort();
        names
    }

    /// Especificação do agente primário.
    #[must_use]
    pub const fn primary(&self) -> &AgentSpec {
        &self.primary
    }

    /// Especificação do subagente `explorer`, se registado.
    #[must_use]
    pub const fn explorer(&self) -> Option<&AgentSpec> {
        self.explorer.as_ref()
    }

    /// Procura a especificação de um agente pelo id.
    #[must_use]
    pub fn get(&self, id: &AgentId) -> Option<&AgentSpec> {
        match id {
            AgentId::Primary => Some(&self.primary),
            AgentId::Explorer => self.explorer.as_ref(),
            AgentId::Named(name) => self.named.get(name.as_ref()),
        }
    }

    /// Procura a especificação de um agente pelo nome (`ktu`, `explorer` ou
    /// declarativo).
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&AgentSpec> {
        match AgentId::from_name(name) {
            Some(id) => self.get(&id),
            None => self.named.get(name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentRegistry, AgentSpec};
    use crate::guard::AgentId;

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn spec_only_allows_declared_tools() {
        let spec = AgentSpec::new(AgentId::Explorer, names(&["read_file", "glob"]), 0);
        assert!(spec.allows("read_file"));
        assert!(!spec.allows("write_file"));
        assert!(!spec.allows("bash"));
        assert_eq!(spec.tool_names(), names(&["glob", "read_file"]));
        assert_eq!(spec.max_depth(), 0);
    }

    #[test]
    fn registry_resolves_primary_and_explorer() {
        let primary = AgentSpec::new(AgentId::Primary, names(&["read_file", "bash"]), 1);
        let explorer = AgentSpec::new(AgentId::Explorer, names(&["read_file"]), 0);
        let registry = AgentRegistry::new(primary, Some(explorer));
        assert!(registry.primary().allows("bash"));
        assert!(registry.get(&AgentId::Explorer).is_some());
        assert!(registry.get_by_name("explorer").is_some());
        assert!(registry.get_by_name("desconhecido").is_none());
        assert_eq!(
            registry.get(&AgentId::Primary).map(AgentSpec::max_depth),
            Some(1)
        );
    }

    #[test]
    fn registry_without_explorer_denies_it() {
        let primary = AgentSpec::new(AgentId::Primary, names(&["read_file"]), 1);
        let registry = AgentRegistry::new(primary, None);
        assert!(registry.get(&AgentId::Explorer).is_none());
        assert!(registry.get_by_name("explorer").is_none());
    }
}
