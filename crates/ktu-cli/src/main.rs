//! `ktu-cli` — o binário `ktu`: front-end de linha de comandos (modo embedded).
//!
//! `stdout` é **dado** (a resposta final); `stderr` é **log** estruturado. Nunca
//! se misturam (§1.3 do `epic.md`). O mapa de comandos vive em [`cli`].
#![forbid(unsafe_code)]

mod cli;
mod clock;
mod guardrails_cmd;
mod hub;
mod init;
pub(crate) mod logging;
mod memory_cmd;
mod mode_cmd;
mod plugins_cmd;
mod provider_defaults;
mod provider_fake;
mod providers;
mod run;
mod serve;
mod session_mode;
mod session_store;
mod subagents;
mod trash_cmd;
mod tui;
mod turn_runtime;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use ktu_core::plan::Approval;
use ktu_core::ports::Logger;

use crate::cli::{Cli, Command, GuardrailsAction, MemoryAction, ProvidersAction, TrashAction};

fn main() -> ExitCode {
    let logger = logging::init();
    let cli = Cli::parse();
    dispatch(cli.command, &logger)
}

/// Despacha o subcomando escolhido para a borda correspondente.
fn dispatch(command: Option<Command>, logger: &logging::TracingLogger) -> ExitCode {
    match command {
        Some(Command::Init) => init::run_command(),
        Some(command @ Command::Run { .. }) => dispatch_run(command, logger),
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
                providers::doctor(&base_url, gguf.as_deref(), choice)
            }
        },
        Some(Command::Trash { action }) => match action {
            TrashAction::List { root } => trash_cmd::list(&root_or_cwd(root)),
            TrashAction::Restore { id, root } => trash_cmd::restore(&root_or_cwd(root), &id),
        },
        Some(Command::Memory { action }) => match action {
            MemoryAction::Export { root } => memory_cmd::export(&root_or_cwd(root)),
            MemoryAction::List { root } => memory_cmd::list(&root_or_cwd(root)),
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
        Some(Command::Doctor { root, memory }) => {
            let root = root_or_cwd(root);
            if memory {
                memory_cmd::doctor(&root)
            } else {
                providers::doctor("http://127.0.0.1:8080", None, ktu_provider::Probe::Skip)
            }
        }
        Some(Command::Tui { server, session }) => match tui::run(&server, &session) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                run::report_error(&message);
                ExitCode::FAILURE
            }
        },
        Some(Command::Plugins { action }) => plugins_cmd::run(action),
        Some(Command::Guardrails { action }) => match action {
            GuardrailsAction::List { root } => guardrails_cmd::list(root),
        },
        None => ExitCode::SUCCESS,
    }
}

/// Resolve o *root* pedido, caindo para o directório actual.
pub(crate) fn root_or_cwd(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| env::current_dir().unwrap_or_default())
}

/// Prepara e corre um turno (`ktu run`) a partir dos argumentos já analisados.
fn dispatch_run(command: Command, logger: &logging::TracingLogger) -> ExitCode {
    let Command::Run {
        prompt,
        root,
        provider,
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
    let mode = if compact {
        session_mode::SessionMode::ResumeCompacted
    } else if resume {
        session_mode::SessionMode::Resume
    } else {
        session_mode::SessionMode::Fresh
    };
    let config = run::RunConfig {
        base_url: base_url.unwrap_or_else(|| provider_defaults::default_base(&provider).to_owned()),
        model: model.unwrap_or_else(|| provider_defaults::default_model(&provider).to_owned()),
        provider,
        api_key,
        effort,
        max_steps,
        session,
        mode,
        plan,
    };
    run_command(root, &prompt, &config, logger)
}

/// Corre um turno do agente (`ktu run`).
fn run_command(
    root: Option<PathBuf>,
    prompt: &[String],
    config: &run::RunConfig,
    logger: &logging::TracingLogger,
) -> ExitCode {
    run::run(&root_or_cwd(root), &prompt.join(" "), logger, config)
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
    let config = run::RunConfig {
        base_url: base_url.unwrap_or_else(|| provider_defaults::default_base(&provider).to_owned()),
        model: model.unwrap_or_else(|| provider_defaults::default_model(&provider).to_owned()),
        provider,
        api_key,
        effort,
        max_steps,
        session: "default".to_owned(),
        mode: session_mode::SessionMode::Resume,
        plan,
    };
    let logger: std::sync::Arc<dyn Logger> = std::sync::Arc::new(logging::TracingLogger);
    match serve::serve(root_or_cwd(root), &bind, config, logger) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            run::report_error(&message);
            ExitCode::FAILURE
        }
    }
}
