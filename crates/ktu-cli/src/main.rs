//! `ktu-cli` — o binário `ktu`: front-end de linha de comandos (modo embedded).
//!
//! `stdout` é **dado** (a resposta final); `stderr` é **log** estruturado. Nunca
//! se misturam (§1.3 do `epic.md`). O mapa de comandos vive em [`cli`].
#![forbid(unsafe_code)]

mod actions;
mod agents_cmd;
mod batch;
mod cli;
mod clock;
mod command_run;
mod commands_cmd;
mod config_cmd;
mod embeddings;
mod guardrails_cmd;
mod hub;
mod init;
mod instructions;
mod lock;
pub(crate) mod logging;
mod memory_cmd;
mod memory_drain;
mod mode_cmd;
mod output;
mod params;
mod permission_cmd;
mod plugins_cmd;
mod prime;
mod prompt_cmd;
mod provider_defaults;
mod provider_script;
mod providers;
mod render;
mod repl;
mod run;
mod run_cmd;
mod script;
mod self_cmd;
mod serve;
mod session_cmd;
mod session_mode;
mod session_store;
mod slash;
mod subagents;
mod surface;
mod trash_cmd;
mod tui;
mod turn_runtime;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use ktu_core::git::GitMode;
use ktu_core::plan::Approval;
use ktu_core::ports::Logger;

use crate::actions::{
    AgentsAction, CommandsAction, DrainCommand, GuardrailsAction, MemoryAction, PermissionAction,
    ProvidersAction, SelfAction, SessionAction, TrashAction,
};
use crate::cli::{Cli, Command};
use crate::output::Format;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let level = if cli.quiet {
        "off"
    } else {
        cli.log_level.as_str()
    };
    let logger = logging::init(level);
    let format = if cli.json { Format::Json } else { Format::Text };
    dispatch(cli.command, format, &logger)
}

/// Despacha o subcomando escolhido para a borda correspondente.
fn dispatch(command: Option<Command>, format: Format, logger: &logging::TracingLogger) -> ExitCode {
    match command {
        Some(Command::Init {
            force,
            git_excluded,
            git_tracked: _,
            no_prompt,
        }) => {
            let git = if git_excluded {
                GitMode::Excluded
            } else {
                GitMode::Tracked
            };
            let overwrite = if force {
                init::Overwrite::Force
            } else {
                init::Overwrite::Keep
            };
            let options = init::Options {
                overwrite,
                git,
                prompt: !no_prompt,
            };
            init::run_command(&options, format)
        }
        Some(command @ Command::Run { .. }) => run_cmd::dispatch_run(command, format, logger),
        Some(command @ Command::Session { .. }) => dispatch_session(command, format),
        Some(command @ Command::Serve { .. }) => dispatch_serve(command, logger),
        Some(Command::Providers { action }) => match action {
            ProvidersAction::Doctor {
                base_url,
                gguf,
                probe,
            } => {
                let choice = if probe {
                    ktu_provider::Probe::Run
                } else {
                    ktu_provider::Probe::Skip
                };
                providers::doctor(&base_url, gguf.as_deref(), choice, format)
            }
        },
        Some(Command::Trash { action }) => match action {
            TrashAction::List { root } => trash_cmd::list(&root_or_cwd(root), format),
            TrashAction::Restore { id, root } => {
                trash_cmd::restore(&root_or_cwd(root), &id, format)
            }
        },
        Some(Command::Memory { action }) => match action {
            MemoryAction::Export { root } => memory_cmd::export(&root_or_cwd(root), format),
            MemoryAction::List { root } => memory_cmd::list(&root_or_cwd(root), format),
            MemoryAction::Project { root, session, out } => memory_cmd::project(
                &root_or_cwd(root),
                session.as_deref(),
                out.as_deref(),
                format,
            ),
            MemoryAction::Drain {
                root,
                flags,
                command,
            } => {
                let root = root_or_cwd(root);
                match command {
                    Some(DrainCommand::Service(args)) => {
                        memory_drain::service(&root, &args, format)
                    }
                    None => memory_drain::queue(&root, flags.mode(), format),
                }
            }
        },
        Some(Command::Mode {
            target,
            approve,
            root,
            session,
        }) => {
            let approval = if approve {
                Approval::Approved
            } else {
                Approval::Pending
            };
            mode_cmd::set(&root_or_cwd(root), &session, &target, approval)
        }
        Some(Command::Prompt { root }) => prompt_cmd::show(root, format),
        Some(Command::Doctor { root, memory }) => {
            let root = root_or_cwd(root);
            if memory {
                memory_cmd::doctor(&root, format)
            } else {
                providers::doctor(
                    "http://127.0.0.1:8080",
                    None,
                    ktu_provider::Probe::Skip,
                    format,
                )
            }
        }
        Some(Command::Tui { server, session }) => match tui::run(&server, &session) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                run::report_error(&message);
                ExitCode::FAILURE
            }
        },
        Some(Command::Plugins { action }) => plugins_cmd::run(action, format),
        Some(Command::Guardrails { action }) => match action {
            GuardrailsAction::List { root } => guardrails_cmd::list(root, format),
        },
        Some(Command::Commands { action }) => match action {
            CommandsAction::Ls { root } => commands_cmd::list(root, format),
        },
        Some(Command::Agents { action }) => match action {
            AgentsAction::Ls { root } => agents_cmd::list(root, format),
        },
        Some(Command::Permission { action }) => match action {
            PermissionAction::Ls { root } => permission_cmd::list(&root_or_cwd(root), format),
            PermissionAction::Set {
                tool,
                permission,
                root,
            } => permission_cmd::set(&root_or_cwd(root), &tool, &permission, format),
            PermissionAction::Unset { tool, root } => {
                permission_cmd::unset(&root_or_cwd(root), &tool, format)
            }
        },
        Some(Command::Config {
            local,
            global,
            action,
        }) => {
            let scope = if local {
                config_cmd::Scope::Local
            } else if global {
                config_cmd::Scope::Global
            } else {
                config_cmd::Scope::Effective
            };
            config_cmd::run(scope, action, format)
        }
        Some(Command::Prime) => prime::run(format),
        Some(Command::SelfCmd { action }) => match action {
            SelfAction::Version => self_cmd::version(format),
            SelfAction::Upgrade => self_cmd::upgrade(),
            SelfAction::Completions { shell } => self_cmd::completions(&shell),
            SelfAction::Surface => surface::run(format),
        },
        None => ExitCode::SUCCESS,
    }
}

/// Resolve o *root* pedido, caindo para o directório actual.
pub(crate) fn root_or_cwd(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| env::current_dir().unwrap_or_default())
}

/// Flags de provider comuns aos front-ends sem `--params` (`serve`, `session`).
struct ProviderFlags {
    provider: Option<String>,
    script: Option<PathBuf>,
    base_url: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    effort: Option<String>,
    max_steps: Option<u32>,
}

/// Constrói a config de turno a partir das flags e dos ficheiros de config.
fn front_config(
    root: &Path,
    flags: ProviderFlags,
    session: session_mode::SessionTarget,
    mode: session_mode::SessionMode,
) -> Result<run::RunConfig, String> {
    let mut settings = config_cmd::load(root)?;
    let provider = flags
        .provider
        .or_else(|| settings.config.provider.clone())
        .unwrap_or_else(|| "llama".to_owned());
    let effort = flags
        .effort
        .or_else(|| settings.config.effort.clone())
        .unwrap_or_else(|| "xhigh".to_owned());
    let max_steps = flags.max_steps.or(settings.config.max_steps).unwrap_or(24);
    let api_key = flags
        .api_key
        .or_else(|| settings.auth.api_key(&provider).map(str::to_owned));
    Ok(run::RunConfig {
        base_url: flags
            .base_url
            .or_else(|| settings.config.base_url.clone())
            .unwrap_or_else(|| provider_defaults::default_base(&provider).to_owned()),
        model: flags
            .model
            .or_else(|| settings.config.model.clone())
            .unwrap_or_else(|| provider_defaults::default_model(&provider).to_owned()),
        provider,
        script: flags.script,
        api_key,
        effort,
        max_steps,
        deferred_tools: settings.config.deferred_tools.take().unwrap_or_default(),
        session,
        mode,
        plan: false,
    })
}

/// Abre o REPL `ktu session` (front primário) ou gere as sessões persistidas.
fn dispatch_session(command: Command, format: Format) -> ExitCode {
    let Command::Session {
        id,
        root,
        provider,
        base_url,
        model,
        api_key,
        effort,
        max_steps,
        plan,
        action,
    } = command
    else {
        return ExitCode::FAILURE;
    };
    if let Some(action) = action {
        return match action {
            SessionAction::Ls { root } => session_cmd::list(&root_or_cwd(root), format),
            SessionAction::Rm { id, root } => session_cmd::remove(&root_or_cwd(root), &id, format),
            SessionAction::Fork { from, to, root } => {
                session_cmd::fork(&root_or_cwd(root), &from, &to, format)
            }
        };
    }
    let root = root_or_cwd(root);
    let id = id.unwrap_or_else(|| "default".to_owned());
    let flags = ProviderFlags {
        provider,
        script: None,
        base_url,
        model,
        api_key,
        effort,
        max_steps,
    };
    let config = match front_config(
        &root,
        flags,
        session_mode::SessionTarget::Persistent(id.clone()),
        session_mode::SessionMode::Resume,
    ) {
        Ok(mut config) => {
            config.plan = plan;
            config
        }
        Err(message) => {
            output::emit_error(format, "session", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(logging::TracingLogger);
    match repl::run(&root, &id, &config, logger) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            run::report_error(&message);
            ExitCode::FAILURE
        }
    }
}

/// Prepara e sobe o servidor da porta única (`ktu serve`).
fn dispatch_serve(command: Command, _logger: &logging::TracingLogger) -> ExitCode {
    let Command::Serve {
        root,
        bind,
        provider,
        base_url,
        model,
        api_key,
        effort,
        max_steps,
        plan,
    } = command
    else {
        return ExitCode::FAILURE;
    };
    let root = root_or_cwd(root);
    let flags = ProviderFlags {
        provider,
        script: None,
        base_url,
        model,
        api_key,
        effort,
        max_steps,
    };
    let config = match front_config(
        &root,
        flags,
        session_mode::SessionTarget::Persistent("default".to_owned()),
        session_mode::SessionMode::Resume,
    ) {
        Ok(mut config) => {
            config.plan = plan;
            config
        }
        Err(message) => {
            run::report_error(&message);
            return ExitCode::FAILURE;
        }
    };
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(logging::TracingLogger);
    match serve::serve(root, &bind, config, logger) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            run::report_error(&message);
            ExitCode::FAILURE
        }
    }
}
