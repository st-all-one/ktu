//! Permissões por tool (`Always`/`Ask`/`Never`) — política **explícita** do projecto.
//!
//! Guardadas em `.agents/permissions.toml` (project-bound; sobrevive a `resume`).
//! O guard avalia o **nome** da tool sobre factos tipados (o mesmo *seam* do
//! [`Guard`]): `Always` permite, `Ask` escala (aprovação humana), `Never` nega.
//! Uma permissão nunca **alarga** o guard estrutural nem os guardrails (I16).

use std::collections::BTreeMap;
use std::path::Path;

use crate::guard::{Decision, DenyReason, Fact, Guard};
use crate::ports::Fs;

/// Ficheiro das permissões (relativo ao *root*).
pub const FILE: &str = ".agents/permissions.toml";

/// Permissão de uma tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    /// Executa sem pedir.
    Always,
    /// Requer aprovação humana (`Escalate`).
    Ask,
    /// Nunca executa.
    Never,
}

impl Permission {
    /// Texto estável.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Ask => "ask",
            Self::Never => "never",
        }
    }

    /// Interpreta o texto estável.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "always" => Some(Self::Always),
            "ask" => Some(Self::Ask),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

/// Conjunto de permissões por tool (determinístico: `BTreeMap`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
    entries: BTreeMap<String, Permission>,
}

impl Store {
    /// Interpreta o TOML (`[permissions]` com `tool = "always|ask|never"`).
    ///
    /// # Errors
    /// Devolve mensagem legível se o TOML for inválido ou tiver permissão desconhecida.
    pub fn parse(text: &str) -> Result<Self, String> {
        let parsed: toml::Value = toml::from_str(text).map_err(|error| error.to_string())?;
        let mut entries = BTreeMap::new();
        if let Some(table) = parsed.get("permissions").and_then(toml::Value::as_table) {
            for (tool, value) in table {
                let text = value
                    .as_str()
                    .ok_or_else(|| format!("permissao invalida para {tool}"))?;
                let permission = Permission::parse(text)
                    .ok_or_else(|| format!("permissao desconhecida para {tool}: {text}"))?;
                entries.insert(tool.clone(), permission);
            }
        }
        Ok(Self { entries })
    }

    /// Renderiza o TOML (ordenado por tool).
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::from("[permissions]\n");
        for (tool, permission) in &self.entries {
            out.push_str(tool);
            out.push_str(" = \"");
            out.push_str(permission.as_str());
            out.push_str("\"\n");
        }
        out
    }

    /// Permissão de uma tool (por omissão `Always`).
    #[must_use]
    pub fn get(&self, tool: &str) -> Permission {
        self.entries
            .get(tool)
            .copied()
            .unwrap_or(Permission::Always)
    }

    /// Define a permissão de uma tool.
    pub fn set(&mut self, tool: &str, permission: Permission) {
        entries_insert(&mut self.entries, tool, permission);
    }

    /// Remove a permissão de uma tool (volta ao *default*).
    pub fn unset(&mut self, tool: &str) {
        entries_remove(&mut self.entries, tool);
    }

    /// Entradas ordenadas por tool.
    #[must_use]
    pub fn entries(&self) -> Vec<(&str, Permission)> {
        self.entries
            .iter()
            .map(|(tool, permission)| (tool.as_str(), *permission))
            .collect()
    }

    /// `true` se não há permissões explícitas.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Insere uma entrada (evita expor o `BTreeMap` no resto do crate).
fn entries_insert(map: &mut BTreeMap<String, Permission>, tool: &str, permission: Permission) {
    map.insert(tool.to_owned(), permission);
}

/// Remove uma entrada.
fn entries_remove(map: &mut BTreeMap<String, Permission>, tool: &str) {
    map.remove(tool);
}

/// Carrega as permissões de `root` (ausente ⇒ vazio).
///
/// # Errors
/// Propaga o erro de leitura ou de *parsing*.
pub fn load(fs: &dyn Fs, root: &Path) -> Result<Store, String> {
    let path = root.join(FILE);
    if !fs.exists(&path) {
        return Ok(Store::default());
    }
    let bytes = fs
        .read(&path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let text = String::from_utf8(bytes).map_err(|_| format!("{}: nao e UTF-8", path.display()))?;
    Store::parse(&text).map_err(|error| format!("{}: {error}", path.display()))
}

/// Guard que aplica as permissões por tool (último elo da cadeia).
pub struct PermissionGuard {
    store: Store,
}

impl PermissionGuard {
    /// Cria o guard a partir das permissões carregadas.
    #[must_use]
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Número de permissões explícitas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.store.entries.len()
    }

    /// `true` se não há permissões explícitas.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

impl Guard for PermissionGuard {
    fn evaluate(&self, fact: &Fact) -> Decision {
        match self.store.get(&fact.tool) {
            Permission::Always => Decision::Allow,
            Permission::Ask => Decision::Escalate,
            Permission::Never => Decision::Deny(DenyReason::Policy),
        }
    }

    fn remedy(&self, fact: &Fact) -> Option<String> {
        match self.store.get(&fact.tool) {
            Permission::Always => None,
            Permission::Ask => Some(format!(
                "aprove a tool {0} com `ktu permission set {0} always`",
                fact.tool
            )),
            Permission::Never => Some(format!("a tool {0} esta proibida em {FILE}", fact.tool)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Permission, PermissionGuard, Store};
    use crate::guard::{AgentId, Decision, DenyReason, Fact, Guard};
    use crate::ports::MemFs;

    fn fact(tool: &str) -> Fact {
        Fact {
            agent: AgentId::Primary,
            tool: tool.to_owned(),
            requested_path: None,
            remaining_steps: 1,
            depth: 0,
        }
    }

    #[test]
    fn parses_and_renders_deterministically() {
        let store = Store::parse("[permissions]\nbash = \"ask\"\nwrite_file = \"never\"\n")
            .unwrap_or_default();
        assert_eq!(store.get("bash"), Permission::Ask);
        assert_eq!(store.get("write_file"), Permission::Never);
        assert_eq!(store.get("read_file"), Permission::Always);
        assert_eq!(
            store.render(),
            "[permissions]\nbash = \"ask\"\nwrite_file = \"never\"\n"
        );
    }

    #[test]
    fn rejects_unknown_permissions() {
        assert!(Store::parse("[permissions]\nbash = \"maybe\"\n").is_err());
    }

    #[test]
    fn guard_maps_permissions_to_decisions() {
        let mut store = Store::default();
        store.set("bash", Permission::Ask);
        store.set("write_file", Permission::Never);
        let guard = PermissionGuard::new(store);
        assert_eq!(guard.evaluate(&fact("read_file")), Decision::Allow);
        assert_eq!(guard.evaluate(&fact("bash")), Decision::Escalate);
        assert_eq!(
            guard.evaluate(&fact("write_file")),
            Decision::Deny(DenyReason::Policy)
        );
        assert!(guard.remedy(&fact("bash")).is_some());
        assert!(guard.remedy(&fact("read_file")).is_none());
    }

    #[test]
    fn unset_returns_to_default() {
        let mut store = Store::default();
        store.set("bash", Permission::Never);
        store.unset("bash");
        assert!(store.is_empty());
        assert_eq!(store.get("bash"), Permission::Always);
    }

    #[test]
    fn load_is_empty_when_absent() {
        let root = PathBuf::from("/tmp/ktu-permission-absent");
        let fs = MemFs::default();
        let store = super::load(&fs, &root).unwrap_or_default();
        assert!(store.is_empty());
    }
}
