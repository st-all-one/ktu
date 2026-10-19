//! Runtime de **um turno**: compõe as portas reais e delega na fachada.
//!
//! É partilhado pelo modo *embedded* (`ktu run`) e pelo servidor (`ktu serve`),
//! para não haver duas composições de DI (§2.3). A composição **pura** (guardrails,
//! permissões, system prompt, orçamento, modo) vive na fachada
//! [`ktu_core::facade::Ktu`] — aqui só se constroem os **adaptadores** reais.

use std::path::Path;
use std::sync::Arc;

use ktu_core::Error;
use ktu_core::agent::{AgentRegistry, LoopConfig, Steering};
use ktu_core::agent_file;
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::StopReason;
use ktu_core::events;
use ktu_core::facade::{Ktu, system_prompt_in};
use ktu_core::layout::AGENTS_FILES_DIR;
use ktu_core::plan::{FsPlanHost, Mode};
use ktu_core::ports::{Level, LogRecord, Logger, NoopMetrics};
use ktu_core::session::SessionLog;
use ktu_memory::JsonlMemory;
use ktu_plugin::PluginHost;
use ktu_tools::{FileMutationQueue, StdFs, ToolThread, Toolbox, default_registry, plan_mode_spec};

use crate::clock::SystemClock;
use crate::config_cmd::global_dir_path;
use crate::embeddings;
use crate::instructions::LocalInstructions;
use crate::plugins_cmd;
use crate::run::{RunConfig, build_provider};
use crate::subagents::CliSubagents;

/// Garante os directórios de runtime do projecto (`.ktu/`, `.agents/memory`).
///
/// # Errors
/// Devolve mensagem legível se algum directório não puder ser criado.
pub(crate) fn ensure_layout(root: &Path) -> Result<(), String> {
    for dir in [
        root.join(".ktu").join("trash"),
        root.join(".agents").join("memory"),
        root.join(".agents").join("memory").join(".idx"),
        root.join(".ktu").join("plan"),
    ] {
        ensure_dir(&dir)?;
    }
    Ok(())
}

/// Garante o directório de sessões (só quando a sessão **persiste**).
pub(crate) fn ensure_sessions(root: &Path) -> Result<(), String> {
    ensure_dir(&root.join(".ktu").join("sessions"))
}

/// Cria um directório (e os pais), com erro legível.
fn ensure_dir(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|error| format!("nao foi possivel criar {}: {error}", dir.display()))
}

/// Mensagem do arranque da fachada: o texto de `Invalid` já é legível e nomeia o ficheiro.
fn build_message(error: Error) -> String {
    match error {
        Error::Invalid { message } => message,
        other => other.to_string(),
    }
}

/// Constrói o registo de agentes: default + declarativos do projecto (fail-closed).
pub(crate) fn load_registry(fs: StdFs, root: &Path) -> Result<AgentRegistry, String> {
    let registry = default_registry();
    let files = agent_file::load(&fs, &root.join(AGENTS_FILES_DIR))
        .map_err(|error| format!("agentes invalidos: {error}"))?;
    let catalog = registry.primary().tool_names();
    agent_file::apply(&files, registry, &catalog)
        .map_err(|error| format!("agentes invalidos: {error}"))
}

/// Compõe as portas e corre **um** turno, escrevendo no `log`.
///
/// O `cancel` é partilhado com a borda (a UI cancela sem esperar pelo turno).
///
/// # Errors
/// Devolve mensagem legível se o provider, a thread de tools ou a fachada não
/// arrancarem (guardrails/permissões inválidos impedem o arranque — I16).
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
    steering: Option<&dyn Steering>,
    mutations: &Arc<FileMutationQueue>,
) -> Result<StopReason, String> {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory_config = embeddings::load_config(&fs, root)?;
    let embedder = embeddings::build(&memory_config.embeddings)
        .map_err(|error| format!("memory.toml: {error}"))?;
    let mut memory = JsonlMemory::new(&fs, &clock, root, root.join(".agents").join("memory"));
    if let Some(embedder) = &embedder {
        memory = memory.with_embeddings(embedder, memory_config.embeddings.clone());
    }
    let provider = build_provider(config)?;
    let registry = load_registry(fs, root)?;
    let plan_spec = plan_mode_spec();
    let plan_mode = log.mode() == Mode::Plan;
    let sessions_dir = root.join(".ktu").join("sessions");
    let deferred = config.deferred_tools.clone();
    let tool_thread = ToolThread::spawn({
        let root = root.to_path_buf();
        let session = config.session.label().to_owned();
        let memory_config = memory_config.clone();
        // A fila é partilhada pelo **processo** (Arc clonado): a T2 vive num thread
        // próprio, pelo que a fila tem de ser possuída por quem a usa.
        let mutations = Arc::clone(mutations);
        move |ctx| {
            let fs = StdFs;
            let clock = SystemClock::new();
            let embedder = embeddings::build(&memory_config.embeddings).ok().flatten();
            let mut memory =
                JsonlMemory::new(&fs, &clock, &root, root.join(".agents").join("memory"));
            if let Some(embedder) = &embedder {
                memory = memory.with_embeddings(embedder, memory_config.embeddings.clone());
            }
            let mut toolbox = Toolbox::new(&root, &fs, &clock, Some(&memory), &mutations)
                .with_deferred(deferred.clone());
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
    let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
    let metrics = NoopMetrics;
    let user_dir = global_dir_path();
    // Plugins externos (§2.9): um manifesto inválido desactiva o plugin, não a sessão.
    // Dois níveis (M8): o global do utilizador + o do projecto, aditivos.
    let plugins = PluginHost::layered(plugins_cmd::levels(root));
    for error in plugins.errors() {
        logger.log(&LogRecord {
            level: Level::Warn,
            event: events::PLUGIN_LIFECYCLE,
            message: "plugin desativado",
            fields: &[("erro", error.as_str())],
        });
    }
    let system = system_prompt_in(&fs, root, user_dir.as_deref());
    let instructions = LocalInstructions::new(root);
    let subagents = CliSubagents::new(
        provider.as_ref(),
        &tool_thread,
        &registry,
        root,
        &fs,
        sessions_dir,
        &budget,
        logger,
        &metrics,
        &clock,
        cancel,
        system,
        LoopConfig {
            max_steps: config.max_steps,
            ..LoopConfig::default()
        },
        config.session.label().to_owned(),
        &instructions,
    );
    let mut builder = Ktu::builder()
        .root(root)
        .fs(&fs)
        .clock(&clock)
        .logger(logger)
        .metrics(&metrics)
        .provider(provider.as_ref())
        .tools(&tool_thread)
        .budget(&budget)
        .cancel(cancel)
        .registry(&registry)
        .plan_spec(&plan_spec)
        .memory(&memory)
        .hooks(&plugins)
        .subagents(&subagents)
        .instructions(&instructions)
        .max_steps(config.max_steps);
    if let Some(dir) = user_dir.as_deref() {
        builder = builder.user_dir(dir);
    }
    if let Some(steering) = steering {
        builder = builder.steering(steering);
    }
    let ktu = builder.build().map_err(build_message)?;
    let outcome = ktu
        .turn(log, prompt)
        .await
        .map_err(|error| error.to_string())?;
    Ok(outcome.reason)
}
