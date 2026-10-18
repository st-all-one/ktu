//! Implementação da porta `SubagentHost` na borda (§2.6).
//!
//! A sessão-filha corre **na thread do kernel** como parte do turno do pai: usa o
//! mesmo provider/portas, mas com o `AgentSpec` do `explorer` (só leitura) e um
//! log próprio, persistido em `.ktu/sessions/<id>.jsonl`. Sem memória automática
//! (a filha não escreve) e sem `subagent` (profundidade máxima 1).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use ktu_core::agent::{AgentRegistry, Capabilities, Deps, LoopConfig, Ports, Telemetry, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::guard::{AgentId, StructuralGuard};
use ktu_core::ports::{Clock, Fs, Instructions, Logger, Metrics};
use ktu_core::provider::Provider;
use ktu_core::session::{Event, SessionLog};
use ktu_core::store;
use ktu_core::subagent::{SubagentHost, SubagentReport, SubagentRequest};
use ktu_core::tool_search;
use ktu_core::tools::{ScopedTools, ToolExecutor};

/// Host de subagentes do front-end embedded.
pub(crate) struct CliSubagents<'a> {
    provider: &'a dyn Provider,
    tools: &'a dyn ToolExecutor,
    registry: &'a AgentRegistry,
    root: &'a Path,
    fs: &'a dyn Fs,
    sessions_dir: PathBuf,
    budget: &'a BudgetGate,
    logger: &'a dyn Logger,
    metrics: &'a dyn Metrics,
    clock: &'a dyn Clock,
    cancel: &'a CancelToken,
    system: String,
    loop_config: LoopConfig,
    parent: String,
    instructions: Option<&'a dyn Instructions>,
}

impl<'a> CliSubagents<'a> {
    /// Cria o host ligado às portas do turno do pai.
    #[must_use]
    #[allow(clippy::too_many_arguments, reason = "composição explícita de DI")]
    pub(crate) fn new(
        provider: &'a dyn Provider,
        tools: &'a dyn ToolExecutor,
        registry: &'a AgentRegistry,
        root: &'a Path,
        fs: &'a dyn Fs,
        sessions_dir: PathBuf,
        budget: &'a BudgetGate,
        logger: &'a dyn Logger,
        metrics: &'a dyn Metrics,
        clock: &'a dyn Clock,
        cancel: &'a CancelToken,
        system: String,
        loop_config: LoopConfig,
        parent: String,
        instructions: &'a dyn Instructions,
    ) -> Self {
        Self {
            provider,
            tools,
            registry,
            root,
            fs,
            sessions_dir,
            budget,
            logger,
            metrics,
            clock,
            cancel,
            system,
            loop_config,
            parent,
            instructions: Some(instructions),
        }
    }
}

#[async_trait]
impl SubagentHost for CliSubagents<'_> {
    fn agents(&self) -> Vec<String> {
        self.registry.subagent_names()
    }

    async fn run(&self, request: SubagentRequest) -> Result<SubagentReport, String> {
        let spec = self.registry.get_by_name(&request.agent).ok_or_else(|| {
            let names = self.registry.subagent_names();
            if names.is_empty() {
                format!("subagente desconhecido: {}", request.agent)
            } else {
                format!(
                    "subagente desconhecido: {}; disponiveis: {}",
                    request.agent,
                    names.join(", ")
                )
            }
        })?;
        if *spec.id() == AgentId::Primary {
            return Err("delegacao para o agente primario recusada".to_owned());
        }
        let session_id = request
            .session_id
            .clone()
            .unwrap_or_else(|| child_session_id(&self.parent, &request.prompt));
        let mut log = match &request.session_id {
            Some(id) => store::load_recovering(self.fs, &self.sessions_dir, id)
                .map_err(|error| error.to_string())?
                .unwrap_or_default(),
            None => SessionLog::new(),
        };
        let mut names = spec.tool_names();
        if !self.tools.deferred().is_empty() {
            names.push(tool_search::TOOL.to_owned());
        }
        let guard = StructuralGuard::new(self.root, names.clone(), spec.max_depth());
        let scoped = ScopedTools::new(self.tools, names);
        let deps = Deps {
            ports: Ports {
                provider: self.provider,
                tools: &scoped,
                guard: &guard,
                budget: self.budget,
                cancel: self.cancel,
            },
            telemetry: Telemetry {
                logger: self.logger,
                metrics: self.metrics,
                clock: self.clock,
            },
            capabilities: Capabilities {
                memory: None,
                subagents: None,
                plugins: None,
                steering: None,
                instructions: self.instructions,
            },
        };
        let prompt = compose_prompt(&request);
        let _reason = run_turn(&mut log, &deps, &self.system, &prompt, &self.loop_config).await;
        std::fs::create_dir_all(&self.sessions_dir).map_err(|error| {
            format!(
                "nao foi possivel criar {}: {error}",
                self.sessions_dir.display()
            )
        })?;
        store::save(self.fs, &self.sessions_dir, &session_id, &log)
            .map_err(|error| format!("nao foi possivel gravar a sessao-filha: {error}"))?;
        write_parent_meta(self.fs, &self.sessions_dir, &session_id, &self.parent);
        Ok(SubagentReport {
            session_id,
            text: final_answer(&log).unwrap_or_default(),
        })
    }
}

/// Compõe o prompt da filha a partir da descrição e da instrução.
fn compose_prompt(request: &SubagentRequest) -> String {
    if request.description.trim().is_empty() {
        return request.prompt.clone();
    }
    format!("{}\n\n{}", request.description, request.prompt)
}

/// Identificador determinístico da sessão-filha (pai + hash do prompt).
fn child_session_id(parent: &str, prompt: &str) -> String {
    format!("{parent}-sub-{:016x}", fnv1a(prompt.as_bytes()))
}

/// FNV-1a de 64 bits — determinístico, sem aleatoriedade.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Reextrai a resposta final (texto, sem `tool_calls`) do turno corrente da filha.
fn final_answer(log: &SessionLog) -> Option<String> {
    let events = log.events();
    let start = events
        .iter()
        .rposition(|event| matches!(event, Event::TurnStart { .. }))
        .map_or(0, |index| index.saturating_add(1));
    events
        .get(start..)
        .unwrap_or(&[])
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::Assistant {
                content,
                tool_calls,
                ..
            } if tool_calls.is_empty() && !content.trim().is_empty() => Some(content.clone()),
            _ => None,
        })
}

/// Regista o pai da sessão-filha num metadado lateral (best-effort).
fn write_parent_meta(fs: &dyn Fs, dir: &Path, session_id: &str, parent: &str) {
    let path = dir.join(format!("{session_id}.meta.json"));
    let meta = format!("{{\"parent\":\"{parent}\",\"session\":\"{session_id}\"}}\n");
    let _ignored = fs.write(&path, meta.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::{child_session_id, compose_prompt, fnv1a};
    use ktu_core::subagent::SubagentRequest;

    #[test]
    fn child_id_is_deterministic_and_distinct() {
        assert_eq!(
            child_session_id("ktu", "mapeia"),
            child_session_id("ktu", "mapeia")
        );
        assert_ne!(child_session_id("ktu", "a"), child_session_id("ktu", "b"));
        assert!(child_session_id("ktu", "x").starts_with("ktu-sub-"));
    }

    #[test]
    fn fnv_is_stable() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_ne!(fnv1a(b"a"), fnv1a(b"b"));
    }

    #[test]
    fn composes_description_and_prompt() {
        let request = SubagentRequest {
            agent: "explorer".to_owned(),
            description: "mapear".to_owned(),
            prompt: "lista ficheiros".to_owned(),
            session_id: None,
        };
        assert_eq!(compose_prompt(&request), "mapear\n\nlista ficheiros");
    }
}
