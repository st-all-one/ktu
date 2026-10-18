//! `ktu session [id]` — REPL multi-turno: **cliente in-process** da porta `Host`.
//!
//! Sem lógica própria (M4): cada turno é [`Host::turn`] e cada comando `/` é um
//! comando da porta. A mesma sessão, por `ktu run` ou pelo REPL, produz o mesmo log.
//! `stdout` = respostas; `stderr` = transcrição e *chrome* (prompt, ajuda).

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::sync::Arc;

use ktu_core::command::{self, Command};
use ktu_core::layout::COMMANDS_DIR;
use ktu_core::plan::{Approval, Mode};
use ktu_core::ports::Logger;
use ktu_core::session::Event;
use ktu_port::host::Host;
use ktu_port::port::EventStream;
use ktu_port::protocol::ServerMsg;
use ktu_tools::StdFs;
use tokio::runtime::Builder;
use tokio::sync::mpsc;

use crate::command_run;
use crate::hub::SessionHub;
use crate::render::{Style, describe_styled};
use crate::run::{RunConfig, report_error};
use crate::slash::{Slash, help, parse_slash};
use crate::turn_runtime::ensure_layout;

/// Canal de linhas de `stdin` (lido numa thread dedicada, para que o turno possa
/// continuar a aceitar *steering* enquanto corre).
type Input = mpsc::UnboundedReceiver<String>;

/// Spawna a thread que lê `stdin` e alimenta o canal (EOF ⇒ canal fechado).
fn spawn_input() -> Input {
    let (sender, receiver) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}

/// Abre o REPL na sessão `id`.
///
/// # Errors
/// Devolve mensagem legível se o runtime, o layout ou a sessão falharem.
pub(crate) fn run(
    root: &Path,
    id: &str,
    config: &RunConfig,
    logger: Arc<dyn Logger>,
) -> Result<(), String> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("nao foi possivel iniciar o runtime: {error}"))?;
    runtime.block_on(session(root, id, config, logger))
}

/// Corpo assíncrono do REPL (o runtime é construído por [`run`]).
async fn session(
    root: &Path,
    id: &str,
    config: &RunConfig,
    logger: Arc<dyn Logger>,
) -> Result<(), String> {
    ensure_layout(root)?;
    let interactive = io::stdin().is_terminal() && io::stderr().is_terminal();
    let style = if interactive {
        Style::Color
    } else {
        Style::Plain
    };
    let hub = Arc::new(SessionHub::new(root.to_path_buf(), config.clone(), logger));
    let info = hub
        .create_session(id)
        .await
        .map_err(|error| error.to_string())?;
    let mut events = hub.subscribe(id);
    let mut mode = hub.state(id).await.map_err(|error| error.to_string())?.mode;
    let (commands, problems) = command::load(&StdFs, &root.join(COMMANDS_DIR));
    let registry = command_run::registry(root)?;
    for problem in &problems {
        log(&format!("aviso: comando ignorado: {problem}"));
    }

    let state = if info.resumed { "retomada" } else { "nova" };
    log(&format!(
        "ktu session {id} [{mode:?}] — {state}; /help ajuda, /quit sai"
    ));
    let mut input = spawn_input();
    let mut pending = String::new();
    loop {
        if interactive {
            prompt(&format!("ktu {id} [{mode:?}]> "));
        }
        let Some(line) = input.recv().await else {
            break;
        };
        let trimmed = line.trim_end();
        if let Some(head) = trimmed.strip_suffix('\\') {
            pending.push_str(head);
            pending.push('\n');
            continue;
        }
        let text = if pending.is_empty() {
            line.trim().to_owned()
        } else {
            let mut joined = std::mem::take(&mut pending);
            joined.push_str(&line);
            joined
        };
        if text.is_empty() {
            continue;
        }
        match parse_slash(&text) {
            Some(Slash::Quit) => break,
            Some(Slash::Custom { name, args }) => match command::find(&commands, &name) {
                Some(found) => {
                    if !found.settings().is_empty() {
                        log("(settings ignorados no REPL; usa `ktu run --command`)");
                    }
                    match command_run::render(root, found, &args, &registry) {
                        Err(message) => report_error(&message),
                        Ok(prompt) => {
                            record_history(root, id, &prompt);
                            if let Err(message) =
                                turn(&hub, id, &prompt, &mut events, &mut input, style).await
                            {
                                report_error(&message);
                            }
                        }
                    }
                }
                None => report_error(&format!("comando desconhecido: /{name} (veja /help)")),
            },
            Some(slash) => {
                if let Err(message) = apply(&hub, id, slash, &mut mode, &commands).await {
                    report_error(&message);
                }
            }
            None => {
                record_history(root, id, &text);
                if let Err(message) = turn(&hub, id, &text, &mut events, &mut input, style).await {
                    report_error(&message);
                }
            }
        }
    }
    log("ate a proxima.");
    Ok(())
}

/// Envia o prompt e drena os eventos, entregando *steering* a meio.
///
/// O input é lido **sem bloquear** entre eventos (`try_recv`): a mensagem é
/// entregue na próxima fronteira segura do loop, sem `select!` nem matar o turno.
#[allow(
    clippy::too_many_arguments,
    reason = "passagem directa do estado do REPL; sem ramificacoes"
)]
async fn turn(
    hub: &SessionHub,
    id: &str,
    prompt: &str,
    events: &mut EventStream,
    input: &mut Input,
    style: Style,
) -> Result<(), String> {
    hub.turn(id, prompt)
        .await
        .map_err(|error| error.to_string())?;
    let mut answer: Option<String> = None;
    loop {
        drain_input(hub, id, input).await;
        let Some(message) = events.recv().await else {
            break;
        };
        match message {
            ServerMsg::Log { event } => {
                if let Event::Assistant {
                    content,
                    tool_calls,
                    ..
                } = &event
                    && tool_calls.is_empty()
                {
                    answer = Some(content.clone());
                }
                log(&describe_styled(&event, style));
            }
            ServerMsg::TurnFinished { .. } => break,
            ServerMsg::Error { message } => report_error(&message),
            _ => {}
        }
    }
    if let Some(text) = answer {
        out(&text);
    }
    Ok(())
}

/// Entrega o input acumulado durante o turno: `/cancel` cancela, o resto é *steering*.
async fn drain_input(hub: &SessionHub, id: &str, input: &mut Input) {
    while let Ok(line) = input.try_recv() {
        steer(hub, id, &line).await;
    }
}

/// Trata uma linha escrita **durante** o turno.
async fn steer(hub: &SessionHub, id: &str, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    match parse_slash(trimmed) {
        Some(Slash::Cancel) => {
            if let Err(error) = hub.cancel(id).await {
                report_error(&error.to_string());
            } else {
                log("(cancelado)");
            }
        }
        Some(_) => log("(comandos / so entre turnos; linha ignorada)"),
        None => {
            if let Err(error) = hub.steer(id, trimmed).await {
                report_error(&error.to_string());
            } else {
                log(&format!("(steer) {trimmed}"));
            }
        }
    }
}

/// Aplica um comando `/` embutido (cada braço é um comando da porta).
async fn apply(
    hub: &SessionHub,
    id: &str,
    slash: Slash,
    mode: &mut Mode,
    commands: &[Command],
) -> Result<(), String> {
    match slash {
        Slash::Quit | Slash::Custom { .. } => {}
        Slash::Help => log(&help(commands)),
        Slash::Plan => {
            hub.set_mode(id, Mode::Plan, Approval::Pending)
                .await
                .map_err(|error| error.to_string())?;
            *mode = Mode::Plan;
            log("modo: plan");
        }
        Slash::Build => {
            hub.set_mode(id, Mode::Build, Approval::Approved)
                .await
                .map_err(|error| error.to_string())?;
            *mode = Mode::Build;
            log("modo: build");
        }
        Slash::Mode => log(&format!("modo: {mode:?}")),
        Slash::Cancel => {
            hub.cancel(id).await.map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

/// Acrescenta o prompt ao histórico da sessão (`.ktu/sessions/<id>.history`).
fn record_history(root: &Path, id: &str, prompt: &str) {
    let dir = root.join(".ktu").join("sessions");
    let _ignored = std::fs::create_dir_all(&dir);
    let line = serde_json::json!({ "prompt": prompt }).to_string();
    let _ignored = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("{id}.history")))
        .and_then(|mut file| writeln!(file, "{line}"));
}

/// Escreve uma linha de dados (resposta) em `stdout`.
fn out(text: &str) {
    let mut stdout = io::stdout().lock();
    let _ignored = writeln!(stdout, "{text}");
}

/// Escreve uma linha de transcrição em `stderr`.
fn log(text: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "{text}");
}

/// Escreve o *prompt* (sem nova linha) em `stderr` e força o *flush*.
fn prompt(text: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = write!(stderr, "{text}");
    let _ignored = stderr.flush();
}
