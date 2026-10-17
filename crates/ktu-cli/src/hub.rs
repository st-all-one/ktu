//! Serviço de sessões atrás da porta (`Host`): **um actor por sessão**.
//!
//! Assim o kernel nunca partilha estado com o front-end (I8). O actor corre os
//! turnos e publica os eventos do log; o `cancel` é de fora-de-banda (não espera
//! o actor).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use ktu_core::cancel::CancelToken;
use ktu_core::plan::{Approval, Mode};
use ktu_core::ports::Logger;
use ktu_core::session::{Event, SessionLog};
use ktu_core::store;
use ktu_port::host::Host;
use ktu_port::port::EventStream;
use ktu_port::protocol::{Health, PortError, ServerMsg, SessionInfo, SessionState};
use ktu_tools::{StdFs, default_registry};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::run::RunConfig;

mod actor;
#[cfg(test)]
mod tests;

use actor::{Cmd, run_actor};

/// Capacidade do canal de eventos em difusão (por sessão).
const EVENT_BUFFER: usize = 256;

/// Instantâneo do estado observável de uma sessão (leitura sem bloquear o actor).
struct Snapshot {
    mode: Mode,
    pending: bool,
    log: Vec<Event>,
}

/// Referência de comando e observação de uma sessão.
#[derive(Clone)]
pub(crate) struct Handle {
    tx: mpsc::UnboundedSender<Cmd>,
    events: broadcast::Sender<ServerMsg>,
    cancel: Arc<Mutex<Option<CancelToken>>>,
    snapshot: Arc<Mutex<Snapshot>>,
}

impl Handle {
    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn is_pending(&self) -> bool {
        Self::lock(&self.snapshot).pending
    }

    fn mark_pending(&self) {
        Self::lock(&self.snapshot).pending = true;
    }

    fn clear_pending(&self) {
        Self::lock(&self.snapshot).pending = false;
    }

    fn store_snapshot(&self, mode: Mode, log: &SessionLog) {
        let mut guard = Self::lock(&self.snapshot);
        guard.mode = mode;
        guard.pending = false;
        guard.log = log.events().to_vec();
    }

    fn install_cancel(&self, token: Option<CancelToken>) {
        *Self::lock(&self.cancel) = token;
    }

    fn request_cancel(&self) {
        if let Some(token) = Self::lock(&self.cancel).as_ref() {
            token.cancel();
        }
    }

    fn command(&self) -> mpsc::UnboundedSender<Cmd> {
        self.tx.clone()
    }

    pub(crate) fn send_event(&self, message: ServerMsg) {
        let _ignored = self.events.send(message);
    }

    fn subscribe(&self) -> broadcast::Receiver<ServerMsg> {
        self.events.subscribe()
    }
}

/// Dependências partilhadas do serviço.
pub(crate) struct Inner {
    pub(crate) root: PathBuf,
    pub(crate) config: RunConfig,
    pub(crate) logger: Arc<dyn Logger>,
    sessions: Mutex<BTreeMap<String, Handle>>,
}

/// Serviço de sessões atrás da porta única.
pub(crate) struct SessionHub {
    inner: Arc<Inner>,
}

impl SessionHub {
    /// Cria o serviço com a raiz e a configuração do provider.
    pub(crate) fn new(root: PathBuf, config: RunConfig, logger: Arc<dyn Logger>) -> Self {
        Self {
            inner: Arc::new(Inner {
                root,
                config,
                logger,
                sessions: Mutex::new(BTreeMap::new()),
            }),
        }
    }

    /// Resolve o `Handle` de uma sessão, se existir.
    fn handle(&self, session_id: &str) -> Result<Handle, PortError> {
        let sessions = Handle::lock(&self.inner.sessions);
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| PortError::NoSession(session_id.to_owned()))
    }

    fn sessions_dir(&self) -> PathBuf {
        self.inner.root.join(".ktu").join("sessions")
    }

    /// Carrega (ou inicia) o log da sessão e arranca o actor.
    fn start_session(&self, session_id: &str) -> Result<SessionInfo, PortError> {
        let mut sessions = Handle::lock(&self.inner.sessions);
        if sessions.contains_key(session_id) {
            return Ok(SessionInfo {
                session_id: session_id.to_owned(),
                resumed: true,
            });
        }
        let (tx, rx) = mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        let handle = Handle {
            tx,
            events,
            cancel: Arc::new(Mutex::new(None)),
            snapshot: Arc::new(Mutex::new(Snapshot {
                mode: Mode::Build,
                pending: false,
                log: Vec::new(),
            })),
        };
        sessions.insert(session_id.to_owned(), handle.clone());
        drop(sessions);

        let fs = StdFs;
        let dir = self.sessions_dir();
        let mut log = match store::load_recovering(&fs, &dir, session_id) {
            Ok(Some(log)) => log,
            Ok(None) => SessionLog::new(),
            Err(error) => {
                let _removed = Handle::lock(&self.inner.sessions).remove(session_id);
                return Err(PortError::Protocol(format!(
                    "nao foi possivel retomar {session_id}: {error}"
                )));
            }
        };
        let resumed = !log.events().is_empty();
        if self.inner.config.plan && log.mode() != Mode::Plan {
            log.set_mode(Mode::Plan);
        }
        handle.store_snapshot(log.mode(), &log);

        let inner = Arc::clone(&self.inner);
        let sid = session_id.to_owned();
        tokio::spawn(run_actor(inner, sid, log, rx, handle));
        Ok(SessionInfo {
            session_id: session_id.to_owned(),
            resumed,
        })
    }
}

#[async_trait::async_trait]
impl Host for SessionHub {
    async fn health(&self) -> Health {
        let registry = default_registry();
        Health {
            provider: self.inner.config.provider.clone(),
            model: self.inner.config.model.clone(),
            tools: registry.primary().tool_names(),
            memory: true,
        }
    }

    async fn create_session(&self, session_id: &str) -> Result<SessionInfo, PortError> {
        self.start_session(session_id)
    }

    async fn turn(&self, session_id: &str, prompt: &str) -> Result<u64, PortError> {
        let handle = self.handle(session_id)?;
        let (reply, receiver) = oneshot::channel();
        handle
            .command()
            .send(Cmd::Turn {
                prompt: prompt.to_owned(),
                reply,
            })
            .map_err(|_| PortError::NoSession(session_id.to_owned()))?;
        receiver
            .await
            .map_err(|_| PortError::Transport("actor terminou".to_owned()))?
    }

    async fn cancel(&self, session_id: &str) -> Result<(), PortError> {
        self.handle(session_id)?.request_cancel();
        Ok(())
    }

    async fn set_mode(
        &self,
        session_id: &str,
        mode: Mode,
        approval: Approval,
    ) -> Result<(), PortError> {
        let handle = self.handle(session_id)?;
        let (reply, receiver) = oneshot::channel();
        handle
            .command()
            .send(Cmd::SetMode {
                mode,
                approval,
                reply,
            })
            .map_err(|_| PortError::NoSession(session_id.to_owned()))?;
        receiver
            .await
            .map_err(|_| PortError::Transport("actor terminou".to_owned()))?
    }

    async fn state(&self, session_id: &str) -> Result<SessionState, PortError> {
        let handle = self.handle(session_id)?;
        let (reply, receiver) = oneshot::channel();
        handle
            .command()
            .send(Cmd::State { reply })
            .map_err(|_| PortError::NoSession(session_id.to_owned()))?;
        receiver
            .await
            .map_err(|_| PortError::Transport("actor terminou".to_owned()))?
    }

    fn subscribe(&self, session_id: &str) -> EventStream {
        let (tx, rx) = mpsc::unbounded_channel();
        let Ok(handle) = self.handle(session_id) else {
            let _ignored = tx.send(ServerMsg::Error {
                message: format!("sessao inexistente: {session_id}"),
            });
            return rx;
        };
        let mut events = handle.subscribe();
        tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(message) => {
                        if tx.send(message).is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        rx
    }
}
