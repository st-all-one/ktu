//! Runtime de **um turno**: compõe as portas e corre o loop.
//!
//! É partilhado pelo modo *embedded* (`ktu run`) e pelo servidor (`ktu serve`),
//! para não haver duas composições de DI (§2.3).

use std::path::Path;

use ktu_core::agent::{Capabilities, Deps, LoopConfig, Ports, Telemetry, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::StopReason;
use ktu_core::events;
use ktu_core::guard::StructuralGuard;
use ktu_core::guardrails::{self, ChainGuard, DeclarativeGuard};
use ktu_core::plan::{FsPlanHost, Mode};
use ktu_core::ports::{Fs, Level, LogRecord, Logger, NoopMetrics};
use ktu_core::prompt::{BASE, compose};
use ktu_core::session::SessionLog;
use ktu_core::tools::{ScopedTools, ToolExecutor};
use ktu_memory::JsonlMemory;
use ktu_plugin::PluginHost;
use ktu_tools::{StdFs, ToolThread, Toolbox, default_registry, plan_mode_spec};

use crate::clock::SystemClock;
use crate::run::{RunConfig, build_provider};
use crate::subagents::CliSubagents;

/// Tecto de contexto (caracteres) do turno.
const CONTEXT_BUDGET_CHARS: usize = 120_000;

/// Garante os directórios de runtime do projecto (`.ktu/`, `.agents/memory`).
///
/// # Errors
/// Devolve mensagem legível se algum directório não puder ser criado.
pub(crate) fn ensure_layout(root: &Path) -> Result<(), String> {
    let dirs = [
        root.join(".ktu").join("sessions"),
        root.join(".ktu").join("trash"),
        root.join(".agents").join("memory"),
        root.join(".ktu").join("plan"),
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)
            .map_err(|error| format!("nao foi possivel criar {}: {error}", dir.display()))?;
    }
    Ok(())
}

/// Compõe o system prompt: base + `AGENTS.md` do projecto (condensado).
pub(crate) fn compose_system(fs: &dyn Fs, root: &Path) -> String {
    let agents_md = fs
        .read(&root.join("AGENTS.md"))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());
    compose(BASE, agents_md.as_deref())
}

/// Compõe as portas e corre **um** turno, escrevendo no `log`.
///
/// O `cancel` é partilhado com a borda (a UI cancela sem esperar pelo turno).
///
/// # Errors
/// Devolve mensagem legível se o provider ou a thread de tools não arrancarem.
#[allow(
    clippy::too_many_arguments,
    reason = "composicao explicita de DI; sem ramificacoes, um so caminho"
)]
pub(crate) async fn execute_turn(
    root: &Path,
    log: &mut SessionLog,
    prompt: &str,
    config: &RunConfig,
    logger: &dyn Logger,
    cancel: &CancelToken,
) -> Result<StopReason, String> {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = JsonlMemory::new(&fs, &clock, root, root.join(".agents").join("memory"));
    let provider = build_provider(config)?;
    let registry = default_registry();
    let plan_spec = plan_mode_spec();
    let plan_mode = log.mode() == Mode::Plan;
    let sessions_dir = root.join(".ktu").join("sessions");
    let tool_thread = ToolThread::spawn({
        let root = root.to_path_buf();
        let session = config.session.clone();
        move |ctx| {
            let fs = StdFs;
            let clock = SystemClock::new();
            let memory = JsonlMemory::new(&fs, &clock, &root, root.join(".agents").join("memory"));
            let mut toolbox = Toolbox::new(&root, &fs, &clock, Some(&memory));
            if plan_mode {
                toolbox = toolbox.with_plan(FsPlanHost::new(
                    &fs,
                    root.join(".ktu").join("plan"),
                    session,
                ));
            }
            ctx.serve(&toolbox);
        }
    })
    .map_err(|error| format!("nao foi possivel iniciar a thread de tools: {error}"))?;
    let scoped = ScopedTools::new(&tool_thread, plan_spec.tool_names());
    let tool_ref: &dyn ToolExecutor = if plan_mode { &scoped } else { &tool_thread };
    let structural = if plan_mode {
        StructuralGuard::for_spec(root, &plan_spec)
    } else {
        StructuralGuard::for_spec(root, registry.primary())
    };
    // Guardrails declarativos (I16): um ficheiro inválido impede o arranque.
    let rules = guardrails::load(&fs, &root.join(".agents").join("guardrails"))
        .map_err(|error| format!("guardrails invalidos: {error}"))?;
    let declarative = DeclarativeGuard::new(rules);
    let guard = ChainGuard::new(&structural, &declarative);
    let budget = BudgetGate::new(CONTEXT_BUDGET_CHARS);
    let metrics = NoopMetrics;
    // Plugins externos (§2.9): um manifesto inválido desactiva o plugin, não a sessão.
    let plugins = PluginHost::load(root.join(".ktu").join("plugins"));
    for error in plugins.errors() {
        logger.log(&LogRecord {
            level: Level::Warn,
            event: events::PLUGIN_LIFECYCLE,
            message: "plugin desativado",
            fields: &[("erro", error.as_str())],
        });
    }
    let system = compose_system(&fs, root);
    let loop_config = LoopConfig {
        max_steps: config.max_steps,
        ..LoopConfig::default()
    };
    let subagents = CliSubagents::new(
        provider.as_ref(),
        tool_ref,
        &registry,
        root,
        &fs,
        sessions_dir,
        &budget,
        logger,
        &metrics,
        &clock,
        cancel,
        system.clone(),
        loop_config,
        config.session.clone(),
    );
    let deps = Deps {
        ports: Ports {
            provider: provider.as_ref(),
            tools: tool_ref,
            guard: &guard,
            budget: &budget,
            cancel,
        },
        telemetry: Telemetry {
            logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: Some(&memory),
            subagents: Some(&subagents),
            plugins: Some(&plugins),
        },
    };
    Ok(run_turn(log, &deps, &system, prompt, &loop_config).await)
}
