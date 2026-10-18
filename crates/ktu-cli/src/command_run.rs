//! Expansão do **shell embutido** de um comando do projecto (`!comando`).
//!
//! Um comando só expande *shell* quando declara `shell: true` (nunca por
//! surpresa) e **cada** segmento passa primeiro por `Guard::evaluate` — o
//! encadeamento **da fachada** (estrutural → guardrails → permissões). Negado ou
//! a exigir aprovação ⇒ o comando **não** corre (fail-closed), com remédio
//! accionável. A execução usa a **mesma** `BashTool` do turno (sem segundo motor).

use std::path::Path;

use ktu_core::agent::{AgentRegistry, AgentSpec};
use ktu_core::command::Command;
use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::facade::evaluate_fact;

use crate::config_cmd::global_dir_path;
use ktu_core::guard::{AgentId, Decision, Fact};
use ktu_core::permission::Permission;
use ktu_core::tool_search;
use ktu_core::tools::ToolExecutor;
use ktu_tools::{BashTool, StdFs};
use tokio::runtime::Builder;

use crate::turn_runtime::load_registry;

/// Renderiza um comando para um turno: argumentos, *shell* guardado e o prompt.
///
/// # Errors
/// Devolve mensagem legível se o registo de agentes ou os guardrails forem
/// inválidos, ou se um segmento de *shell* for negado.
pub(crate) fn render(
    root: &Path,
    command: &Command,
    args: &str,
    registry: &AgentRegistry,
) -> Result<String, String> {
    let mut prompt = command.render(args);
    let segments = command.shell_segments();
    if segments.is_empty() {
        return Ok(prompt);
    }
    let fs = StdFs;
    let user_dir = global_dir_path();
    let spec = shell_spec(registry);
    let bash = BashTool::new(root);
    for segment in segments {
        let fact = Fact {
            agent: AgentId::Primary,
            tool: "bash".to_owned(),
            requested_path: None,
            remaining_steps: u32::MAX,
            depth: 0,
        };
        let decision = evaluate_fact(&spec, root, user_dir.as_deref(), &fs, &fact)
            .map_err(|error| error.to_string())?;
        match decision {
            Decision::Allow => {}
            Decision::Deny(reason) => {
                return Err(format!(
                    "comando `{}`: shell negado ({}): {segment}",
                    command.name(),
                    reason.as_str()
                ));
            }
            Decision::Escalate => {
                return Err(format!(
                    "comando `{}`: shell exige aprovacao — aprova `bash` com \
                     `ktu permission set bash {}` ou corre a tool no turno",
                    command.name(),
                    Permission::Always.as_str()
                ));
            }
        }
        let call = ToolCall {
            id: ToolCallId::new("recipe"),
            name: "bash".to_owned(),
            arguments: serde_json::json!({ "command": segment }).to_string(),
        };
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("nao foi possivel iniciar o runtime: {error}"))?;
        let outcome = runtime.block_on(bash.execute(&call));
        let output = outcome.content.trim();
        let replacement = if outcome.is_error {
            format!("[shell falhou: {segment}]\n{output}")
        } else {
            format!("[shell: {segment}]\n{output}")
        };
        prompt = prompt.replace(&format!("!{segment}"), &replacement);
    }
    Ok(prompt)
}

/// Especificação do agente para o *shell* de um comando: o primário efectivo (com
/// `bash`), mais `tool_search` quando existem tools diferidas (como no turno).
fn shell_spec(registry: &AgentRegistry) -> AgentSpec {
    let primary = registry.primary();
    let mut tools = primary.tool_names();
    tools.push(tool_search::TOOL.to_owned());
    AgentSpec::new(AgentId::Primary, tools, primary.max_depth())
}

/// Registo efectivo do projecto (default + agentes declarativos).
///
/// # Errors
/// Devolve mensagem legível se os agentes declarativos forem inválidos.
pub(crate) fn registry(root: &Path) -> Result<AgentRegistry, String> {
    load_registry(StdFs, root)
}
