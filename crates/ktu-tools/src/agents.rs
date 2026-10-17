//! Registo de agentes do ktu (§2.6).
//!
//! O agente primário vê o conjunto completo; o `explorer` só vê leitura — e essa
//! restrição é **estrutural** (conjunto de tools), não um pedido no prompt.

use ktu_core::agent::{AgentRegistry, AgentSpec};
use ktu_core::guard::AgentId;

use crate::fs_tools::tool_names;

/// Tools de leitura do subagente `explorer` (nunca escreve, nunca executa).
///
/// `memory_ask` é leitura; `memory_write` está ausente de propósito.
pub const EXPLORER_TOOLS: &[&str] = &["read_file", "list_dir", "glob", "grep", "memory_ask"];

/// Nomes das tools de leitura, em ordem determinística.
#[must_use]
pub fn read_only_tool_names() -> Vec<String> {
    EXPLORER_TOOLS
        .iter()
        .map(|name| (*name).to_owned())
        .collect()
}

/// Nome canónico da tool de delegação (espelha `ktu_core::subagent::TOOL`).
pub const SUBAGENT_TOOL: &str = "subagent";

/// Constrói o registo default: primário `ktu` (todas as tools + `subagent`) e
/// `explorer` read-only. A profundidade do primário é 1; a do `explorer` é 0
/// (sem netos).
#[must_use]
pub fn default_registry() -> AgentRegistry {
    let mut primary_tools = tool_names();
    primary_tools.push(SUBAGENT_TOOL.to_owned());
    let primary = AgentSpec::new(AgentId::Primary, primary_tools, 1);
    let explorer = AgentSpec::new(AgentId::Explorer, read_only_tool_names(), 0);
    AgentRegistry::new(primary, Some(explorer))
}

/// `AgentSpec` do **modo plano** (§2.13): read-only + `plan` + `subagent`.
///
/// Nunca inclui `write_file`/`edit_file`/`bash`/`trash_file`/`memory_write` —
/// o read-only é estrutural, não um pedido no prompt.
#[must_use]
pub fn plan_mode_spec() -> AgentSpec {
    let mut tools = read_only_tool_names();
    tools.push("plan".to_owned());
    tools.push(SUBAGENT_TOOL.to_owned());
    AgentSpec::new(AgentId::Primary, tools, 1)
}

#[cfg(test)]
mod tests {
    use super::{default_registry, read_only_tool_names};
    use ktu_core::guard::AgentId;

    #[test]
    fn explorer_is_read_only() {
        let registry = default_registry();
        assert!(registry.get(AgentId::Explorer).is_some());
        let Some(explorer) = registry.get(AgentId::Explorer) else {
            return;
        };
        for tool in ["read_file", "list_dir", "glob", "grep", "memory_ask"] {
            assert!(explorer.allows(tool), "explorer devia permitir {tool}");
        }
        for tool in [
            "write_file",
            "edit_file",
            "bash",
            "trash_file",
            "memory_write",
        ] {
            assert!(!explorer.allows(tool), "explorer nao devia permitir {tool}");
        }
        assert_eq!(explorer.max_depth(), 0);
    }

    #[test]
    fn primary_sees_write_tools_and_can_delegate() {
        let registry = default_registry();
        let primary = registry.primary();
        assert!(primary.allows("write_file"));
        assert!(primary.allows("bash"));
        assert!(primary.allows("subagent"));
        assert_eq!(primary.max_depth(), 1);
    }

    #[test]
    fn plan_mode_spec_is_read_only_with_plan() {
        let spec = super::plan_mode_spec();
        assert!(spec.allows("read_file"));
        assert!(spec.allows("plan"));
        assert!(spec.allows("subagent"));
        for tool in [
            "write_file",
            "edit_file",
            "bash",
            "trash_file",
            "memory_write",
        ] {
            assert!(!spec.allows(tool), "modo plano nao devia permitir {tool}");
        }
    }

    #[test]
    fn read_only_names_exclude_writes() {
        let names = read_only_tool_names();
        assert!(!names.iter().any(|name| name == "write_file"));
        assert!(names.iter().any(|name| name == "read_file"));
    }
}
