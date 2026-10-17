//! Actor de **uma** sessão: corre os turnos em sequência e publica os eventos do
//! log. O actor é dono do `SessionLog`; ninguém mais lhe toca (I3/I8).

use std::sync::Arc;

use ktu_core::cancel::CancelToken;
use ktu_core::domain::StopReason;
use ktu_core::plan::{Approval, Mode};
use ktu_core::session::SessionLog;
use ktu_core::store;
use ktu_port::protocol::{PortError, ServerMsg, SessionState};
use ktu_tools::StdFs;
use tokio::sync::{mpsc, oneshot};

use super::{Handle, Inner};
use crate::turn_runtime::execute_turn;

/// Comando dirigido a uma sessão.
pub(crate) enum Cmd {
    /// Corre um turno com o prompt dado.
    Turn {
        /// Instrução do utilizador.
        prompt: String,
        /// Devolve o id do turno (ou o erro de admissão).
        reply: oneshot::Sender<Result<u64, PortError>>,
    },
    /// Muda o modo da sessão (I15).
    SetMode {
        /// Modo alvo.
        mode: Mode,
        /// Aprovação da transição `plan→build`.
        approval: Approval,
        /// Confirmação.
        reply: oneshot::Sender<Result<(), PortError>>,
    },
    /// Devolve o estado corrente (com o log).
    State {
        /// Estado observável.
        reply: oneshot::Sender<Result<SessionState, PortError>>,
    },
}

/// Corre o actor até o canal de comandos fechar.
pub(crate) async fn run_actor(
    inner: Arc<Inner>,
    session_id: String,
    mut log: SessionLog,
    mut rx: mpsc::UnboundedReceiver<Cmd>,
    handle: Handle,
) {
    let mut mode = log.mode();
    let mut turns: u64 = 0;
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Cmd::Turn { prompt, reply } => {
                if handle.is_pending() {
                    let _ignored = reply.send(Err(busy_error()));
                    continue;
                }
                turns = turns.saturating_add(1);
                let _ignored = reply.send(Ok(turns));
                run_turn(&inner, &session_id, &mut log, &prompt, &handle, turns).await;
                mode = log.mode();
                handle.store_snapshot(mode, &log);
            }
            Cmd::SetMode {
                mode: target,
                approval,
                reply,
            } => {
                let result = set_mode(
                    &inner,
                    &session_id,
                    &mut log,
                    mode,
                    target,
                    approval,
                    &handle,
                );
                if let Ok(next) = result {
                    mode = next;
                    handle.store_snapshot(mode, &log);
                }
                let _ignored = reply.send(result.map(|_| ()));
            }
            Cmd::State { reply } => {
                let state = SessionState {
                    session_id: session_id.clone(),
                    mode,
                    pending: handle.is_pending(),
                    log: log.events().to_vec(),
                };
                let _ignored = reply.send(Ok(state));
            }
        }
    }
}

/// Aplica uma transição de modo, regista-a e persiste.
#[allow(
    clippy::too_many_arguments,
    reason = "passagem directa do comando para a transicao; sem ramificacoes"
)]
fn set_mode(
    inner: &Arc<Inner>,
    session_id: &str,
    log: &mut SessionLog,
    current: Mode,
    target: Mode,
    approval: Approval,
    handle: &Handle,
) -> Result<Mode, PortError> {
    if handle.is_pending() {
        return Err(busy_error());
    }
    let next = Mode::transition(current, target, approval).map_err(PortError::Protocol)?;
    log.set_mode(next);
    if let Some(event) = log.events().last() {
        handle.send_event(ServerMsg::Log {
            event: event.clone(),
        });
    }
    persist(inner, session_id, log, handle);
    Ok(next)
}

/// Corre um turno, publica os eventos novos e persiste o log.
#[allow(
    clippy::too_many_arguments,
    reason = "composicao explicita do turno; sem ramificacoes"
)]
async fn run_turn(
    inner: &Arc<Inner>,
    session_id: &str,
    log: &mut SessionLog,
    prompt: &str,
    handle: &Handle,
    turn_id: u64,
) {
    handle.mark_pending();
    handle.send_event(ServerMsg::TurnAccepted { turn_id });
    let token = CancelToken::new();
    handle.install_cancel(Some(token.clone()));
    let before = log.events().len();
    let result = execute_turn(
        &inner.root,
        log,
        prompt,
        &inner.config,
        inner.logger.as_ref(),
        &token,
    )
    .await;
    handle.install_cancel(None);
    handle.clear_pending();
    for event in log.events().get(before..).unwrap_or(&[]) {
        handle.send_event(ServerMsg::Log {
            event: event.clone(),
        });
    }
    let reason = match result {
        Ok(reason) => reason,
        Err(message) => {
            handle.send_event(ServerMsg::Error { message });
            StopReason::Error
        }
    };
    handle.send_event(ServerMsg::TurnFinished { reason });
    persist(inner, session_id, log, handle);
}

/// Grava o log de forma durável; uma falha degrada para o stream de eventos.
fn persist(inner: &Arc<Inner>, session_id: &str, log: &SessionLog, handle: &Handle) {
    let fs = StdFs;
    let dir = inner.root.join(".ktu").join("sessions");
    if let Err(error) = store::save(&fs, &dir, session_id, log) {
        handle.send_event(ServerMsg::Error {
            message: format!("falha ao gravar a sessao: {error}"),
        });
    }
}

/// Erro devolvido quando já há um turno em curso.
fn busy_error() -> PortError {
    PortError::Protocol("ja ha um turno em curso".to_owned())
}
