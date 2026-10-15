//! Guardrails estruturais: o **único** caminho para qualquer efeito (I10).
//!
//! A tese (§2.8): *enforcement* são regras determinísticas sobre **factos
//! tipados**, nunca prompts. Fail-closed: desconhecido nunca é permissão.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// Agente em execução (o conjunto de tools vem do seu `AgentSpec`, §2.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentId {
    /// Agente primário (vê todas as tools do seu conjunto).
    Primary,
    /// Subagente `explorer` (só leitura, estruturalmente).
    Explorer,
}

impl AgentId {
    /// Nome estável do agente.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "ktu",
            Self::Explorer => "explorer",
        }
    }
}

/// Facto tipado sobre o qual o guardrail decide.
#[derive(Clone, Debug)]
pub struct Fact {
    /// Agente que pede o efeito.
    pub agent: AgentId,
    /// Nome da tool.
    pub tool: String,
    /// Path pedido (se a tool for de caminho), antes de resolver.
    pub requested_path: Option<String>,
    /// Passos restantes no turno.
    pub remaining_steps: u32,
    /// Profundidade de subagente.
    pub depth: u8,
}

/// Motivo tipado de negação (texto estável, contrato de log).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DenyReason {
    /// Tool fora do conjunto do `AgentSpec`.
    Capability,
    /// Path resolvido sai do *root*.
    PathEscape,
    /// Tecto de passos atingido.
    StepCap,
    /// Profundidade de subagente excedida.
    DepthExceeded,
}

impl DenyReason {
    /// Texto estável do motivo.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Capability => "capability",
            Self::PathEscape => "path_escape",
            Self::StepCap => "step_cap",
            Self::DepthExceeded => "depth_exceeded",
        }
    }
}

/// Decisão do guardrail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Efeito permitido.
    Allow,
    /// Efeito negado, com motivo.
    Deny(DenyReason),
    /// Requer aprovação humana (front-end só pode restringir — §2.8).
    Escalate,
}

/// Porta do guardrail: avaliada **antes** de qualquer efeito.
pub trait Guard: Send + Sync {
    /// Avalia um facto.
    fn evaluate(&self, fact: &Fact) -> Decision;
}

/// Guard estrutural mínimo: capacidade, confinamento de path e caps.
pub struct StructuralGuard {
    allowed_tools: BTreeSet<String>,
    root: PathBuf,
    max_depth: u8,
}

impl StructuralGuard {
    /// Cria um guard com o conjunto de tools permitidas e o *root* da sessão.
    #[must_use]
    pub fn new(
        root: impl Into<PathBuf>,
        allowed_tools: impl IntoIterator<Item = String>,
        max_depth: u8,
    ) -> Self {
        Self {
            allowed_tools: allowed_tools.into_iter().collect(),
            root: root.into(),
            max_depth,
        }
    }
}

impl Guard for StructuralGuard {
    fn evaluate(&self, fact: &Fact) -> Decision {
        if fact.depth > self.max_depth {
            return Decision::Deny(DenyReason::DepthExceeded);
        }
        if fact.remaining_steps == 0 {
            return Decision::Deny(DenyReason::StepCap);
        }
        if !self.allowed_tools.contains(&fact.tool) {
            return Decision::Deny(DenyReason::Capability);
        }
        if let Some(path) = &fact.requested_path
            && resolve_within(&self.root, path).is_none()
        {
            return Decision::Deny(DenyReason::PathEscape);
        }
        Decision::Allow
    }
}

/// Extrai o valor de `path` dos argumentos JSON, se presente.
#[must_use]
pub fn path_argument(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    value
        .get("path")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

/// Resolve `requested` dentro de `root`, recusando caminhos absolutos e `..` que
/// escapem. É a base do confinamento de path (testes adversariais no §4).
#[must_use]
pub fn resolve_within(root: &Path, requested: &str) -> Option<PathBuf> {
    let candidate = Path::new(requested);
    if candidate.is_absolute() {
        return None;
    }
    let mut resolved = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(root.join(resolved))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Decision, DenyReason, Fact, Guard, StructuralGuard, resolve_within};
    use crate::guard::AgentId;

    fn fact(tool: &str, path: Option<&str>) -> Fact {
        Fact {
            agent: AgentId::Primary,
            tool: tool.to_owned(),
            requested_path: path.map(str::to_owned),
            remaining_steps: 10,
            depth: 0,
        }
    }

    #[test]
    fn rejects_absolute_and_escaping_paths() {
        let root = Path::new("/r");
        assert!(resolve_within(root, "/etc/passwd").is_none());
        assert!(resolve_within(root, "../../etc/passwd").is_none());
        assert!(resolve_within(root, "a/../../b").is_none());
    }

    #[test]
    fn resolves_nested_and_inner_parent() {
        let root = Path::new("/r");
        assert_eq!(
            resolve_within(root, "a/b.txt"),
            Some(PathBuf::from("/r/a/b.txt"))
        );
        assert_eq!(
            resolve_within(root, "a/../b").unwrap_or_default(),
            PathBuf::from("/r/b")
        );
    }

    #[test]
    fn guard_denies_capability_and_path_escape() {
        let guard = StructuralGuard::new("/r", ["read_file".to_owned()], 1);
        assert_eq!(
            guard.evaluate(&fact("read_file", Some("a"))),
            Decision::Allow
        );
        assert_eq!(
            guard.evaluate(&fact("write_file", None)),
            Decision::Deny(DenyReason::Capability)
        );
        assert_eq!(
            guard.evaluate(&fact("read_file", Some("../x"))),
            Decision::Deny(DenyReason::PathEscape)
        );
    }
}
