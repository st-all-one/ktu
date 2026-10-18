//! `ktu run` — preparação e despacho do turno *embedded*.
//!
//! Separado de `main.rs` (que fica com o mapa de despacho) para manter os
//! ficheiros pequenos.

use std::path::Path;
use std::process::ExitCode;

use ktu_core::command::{self, Command as Recipe};
use ktu_core::layout::COMMANDS_DIR;
use ktu_tools::StdFs;

use crate::cli::Command;
use crate::output::{self, Format};
use crate::{
    batch, command_run, config_cmd, logging, params, provider_defaults, root_or_cwd, run,
    session_mode,
};

/// Resolve o destino da sessão: `--session` explícito persiste; `--resume`/`--compact`
/// implicam a sessão `default`; caso contrário é **efémera** (uso #2, pipelines).
fn resolve_session(
    flag: Option<String>,
    param: Option<String>,
    mode: session_mode::SessionMode,
) -> session_mode::SessionTarget {
    if let Some(id) = flag.or(param) {
        return session_mode::SessionTarget::Persistent(id);
    }
    if mode.resumes() {
        return session_mode::SessionTarget::Persistent("default".to_owned());
    }
    session_mode::SessionTarget::Ephemeral
}

/// Prepara e corre um turno (`ktu run`): funde flags, `--params` e defaults.
pub(crate) fn dispatch_run(
    command: Command,
    format: Format,
    logger: &logging::TracingLogger,
) -> ExitCode {
    let Command::Run {
        prompt,
        command,
        params,
        batch,
        sequential,
        concurrency,
        root,
        provider,
        script,
        base_url,
        model,
        api_key,
        effort,
        max_steps,
        session,
        resume,
        compact,
        plan,
    } = command
    else {
        return ExitCode::FAILURE;
    };
    let root = root_or_cwd(root);
    // Comando do projecto (se pedido): resolve antes da config, para que as suas
    // `settings` entrem na precedência **flag > comando > params > config**.
    let recipe = match command.as_deref().map(|name| resolve_command(&root, name)) {
        None => None,
        Some(Ok(found)) => Some(found),
        Some(Err(message)) => {
            output::emit_error(format, "run", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    let mut recipe_settings = recipe
        .as_ref()
        .map(|found| found.settings().clone())
        .unwrap_or_default();
    let mut settings = match config_cmd::load(&root) {
        Ok(settings) => settings,
        Err(message) => {
            output::emit_error(format, "run", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    let params = match params.as_deref().map(params::parse) {
        None => params::RunParams::default(),
        Some(Ok(parsed)) => parsed,
        Some(Err(message)) => {
            output::emit_error(format, "run", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    let provider = params::pick(
        provider.or_else(|| recipe_settings.provider.take()),
        params.provider,
        settings
            .config
            .provider
            .clone()
            .unwrap_or_else(|| "llama".to_owned()),
    );
    let effort = params::pick(
        effort,
        params.effort,
        settings
            .config
            .effort
            .clone()
            .unwrap_or_else(|| "xhigh".to_owned()),
    );
    let max_steps = params::pick(
        max_steps.or(recipe_settings.max_steps),
        params.max_steps,
        settings.config.max_steps.unwrap_or(24),
    );
    let resume = resume || params.resume.unwrap_or(false);
    let compact = compact || params.compact.unwrap_or(false);
    let plan = plan || params.plan.unwrap_or(false);
    let mode = if compact {
        session_mode::SessionMode::ResumeCompacted
    } else if resume {
        session_mode::SessionMode::Resume
    } else {
        session_mode::SessionMode::Fresh
    };
    let session = resolve_session(session, params.session, mode);
    let api_key = api_key
        .or(params.api_key)
        .or_else(|| settings.auth.api_key(&provider).map(str::to_owned));
    let config = run::RunConfig {
        base_url: base_url
            .or_else(|| recipe_settings.base_url.take())
            .or(params.base_url)
            .or_else(|| settings.config.base_url.clone())
            .unwrap_or_else(|| provider_defaults::default_base(&provider).to_owned()),
        model: model
            .or_else(|| recipe_settings.model.take())
            .or(params.model)
            .or_else(|| settings.config.model.clone())
            .unwrap_or_else(|| provider_defaults::default_model(&provider).to_owned()),
        provider,
        script,
        api_key,
        effort,
        max_steps,
        deferred_tools: settings.config.deferred_tools.take().unwrap_or_default(),
        session,
        mode,
        plan,
    };
    if let Some(source) = batch {
        let options = batch::BatchOptions {
            source: &source,
            sequential,
            concurrency,
        };
        return batch::run(&root, logger, &config, &options);
    }
    let words: Vec<String> = if prompt.is_empty() {
        params.prompt.map(|text| vec![text]).unwrap_or_default()
    } else {
        prompt
    };
    if let Some(found) = &recipe {
        return match render_command(&root, found, &words.join(" ")) {
            Ok(text) => run::run(&root, &text, logger, &config),
            Err(message) => {
                output::emit_error(format, "run", "invalid", &message);
                ExitCode::FAILURE
            }
        };
    }
    match run::resolve_prompt(&words) {
        Ok(text) => run::run(&root, &text, logger, &config),
        Err(message) => {
            output::emit_error(format, "run", "invalid", &message);
            ExitCode::FAILURE
        }
    }
}

/// Resolve um comando do projecto pelo nome (erro accionável com a lista).
fn resolve_command(root: &Path, name: &str) -> Result<Recipe, String> {
    let fs = StdFs;
    let (commands, _problems) = command::load(&fs, &root.join(COMMANDS_DIR));
    if let Some(found) = command::find(&commands, name) {
        return Ok(found.clone());
    }
    let names: Vec<&str> = commands.iter().map(Recipe::name).collect();
    if names.is_empty() {
        Err(format!(
            "comando desconhecido: {name} (nao ha comandos em .agents/commands/)"
        ))
    } else {
        Err(format!(
            "comando desconhecido: {name}; disponiveis: {}",
            names.join(", ")
        ))
    }
}

/// Renderiza o comando (argumentos + *shell* guardado).
fn render_command(root: &Path, command: &Recipe, args: &str) -> Result<String, String> {
    let registry = command_run::registry(root)?;
    command_run::render(root, command, args, &registry)
}
