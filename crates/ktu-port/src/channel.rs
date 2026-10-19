//! Transporte in-process (*embedded*): o front-end fala **directamente** com o
//! serviço, sem socket nem serialização. É o mesmo protocolo que o `Http`, por
//! isso nenhum front-end tem lógica própria.

use ktu_core::plan::{Approval, Mode};

use crate::host::SharedHost;
use crate::port::{EventStream, Port};
use crate::protocol::{Health, PortError, SessionInfo, SessionState};

/// Cliente `Channel` — liga um front-end ao serviço no mesmo processo.
pub struct Channel {
    host: SharedHost,
}

impl Channel {
    /// Liga ao serviço dado.
    #[must_use]
    pub fn new(host: SharedHost) -> Self {
        Self { host }
    }
}

#[async_trait::async_trait]
impl Port for Channel {
    async fn health(&self) -> Result<Health, PortError> {
        Ok(self.host.health().await)
    }

    async fn create_session(&self, session_id: &str) -> Result<SessionInfo, PortError> {
        self.host.create_session(session_id).await
    }

    async fn turn(&self, session_id: &str, prompt: &str) -> Result<u64, PortError> {
        self.host.turn(session_id, prompt).await
    }

    async fn cancel(&self, session_id: &str) -> Result<(), PortError> {
        self.host.cancel(session_id).await
    }

    async fn steer(&self, session_id: &str, id: &str, text: &str) -> Result<(), PortError> {
        self.host.steer(session_id, id, text).await
    }

    async fn set_mode(
        &self,
        session_id: &str,
        mode: Mode,
        approval: Approval,
    ) -> Result<(), PortError> {
        self.host.set_mode(session_id, mode, approval).await
    }

    async fn state(&self, session_id: &str) -> Result<SessionState, PortError> {
        self.host.state(session_id).await
    }

    fn subscribe(&self, session_id: &str) -> EventStream {
        self.host.subscribe(session_id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, PoisonError};

    use ktu_core::plan::{Approval, Mode};
    use tokio::sync::mpsc;

    use super::Channel;
    use crate::host::Host;
    use crate::port::{EventStream, Port};
    use crate::protocol::{Health, PortError, ServerMsg, SessionInfo, SessionState};

    /// Serviço de teste: guarda o último modo e reencaminha eventos.
    #[derive(Default)]
    struct FakeHost {
        sender: Mutex<Option<mpsc::UnboundedSender<ServerMsg>>>,
        mode: Mutex<Mode>,
    }

    #[async_trait::async_trait]
    impl Host for FakeHost {
        async fn health(&self) -> Health {
            Health {
                provider: "mock".to_owned(),
                model: "eco".to_owned(),
                tools: vec!["read_file".to_owned()],
                memory: false,
            }
        }

        async fn create_session(&self, session_id: &str) -> Result<SessionInfo, PortError> {
            Ok(SessionInfo {
                session_id: session_id.to_owned(),
                resumed: false,
            })
        }

        async fn turn(&self, _session_id: &str, _prompt: &str) -> Result<u64, PortError> {
            Ok(1)
        }

        async fn cancel(&self, _session_id: &str) -> Result<(), PortError> {
            Ok(())
        }

        async fn steer(&self, _session_id: &str, _id: &str, _text: &str) -> Result<(), PortError> {
            Ok(())
        }

        async fn set_mode(
            &self,
            _session_id: &str,
            mode: Mode,
            _approval: Approval,
        ) -> Result<(), PortError> {
            *self.mode.lock().unwrap_or_else(PoisonError::into_inner) = mode;
            Ok(())
        }

        async fn state(&self, session_id: &str) -> Result<SessionState, PortError> {
            Ok(SessionState {
                keep_from: None,
                session_id: session_id.to_owned(),
                mode: *self.mode.lock().unwrap_or_else(PoisonError::into_inner),
                pending: false,
                log: Vec::new(),
            })
        }

        fn subscribe(&self, _session_id: &str) -> EventStream {
            let (sender, receiver) = mpsc::unbounded_channel();
            *self.sender.lock().unwrap_or_else(PoisonError::into_inner) = Some(sender);
            receiver
        }
    }

    #[tokio::test]
    async fn channel_forwards_calls_and_events() -> Result<(), Box<dyn std::error::Error>> {
        let host = std::sync::Arc::new(FakeHost::default());
        let port = Channel::new(host.clone());

        let info = port.create_session("s1").await?;
        assert_eq!(info.session_id, "s1");
        assert_eq!(port.health().await?.provider, "mock");

        let mut events = port.subscribe("s1");
        let maybe_sender = host
            .sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let Some(sender) = maybe_sender else {
            return Err("subscricao sem remetente".into());
        };
        if sender.send(ServerMsg::TurnAccepted { turn_id: 7 }).is_err() {
            return Err("canal de eventos fechado".into());
        }
        let received = events.recv().await.ok_or("sem evento")?;
        assert_eq!(received, ServerMsg::TurnAccepted { turn_id: 7 });

        port.set_mode("s1", Mode::Plan, Approval::Pending).await?;
        assert_eq!(port.state("s1").await?.mode, Mode::Plan);
        port.steer("s1", "s1", "faz antes isto").await?;
        Ok(())
    }
}
