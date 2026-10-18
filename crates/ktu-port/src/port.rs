//! A superfície que **qualquer** front-end usa (`Port`), independente do
//! transporte: `Channel` (in-process) ou `Http` (rede). Nenhum front-end toca
//! no estado do kernel; só fala por aqui (§2.3 do `plan.md`).

use ktu_core::plan::{Approval, Mode};
use tokio::sync::mpsc;

use crate::protocol::{Health, PortError, ServerMsg, SessionInfo, SessionState};

/// Fluxo de eventos de uma sessão, pela ordem de emissão.
pub type EventStream = mpsc::UnboundedReceiver<ServerMsg>;

/// Cliente da porta única: a fronteira do front-end com o kernel.
#[async_trait::async_trait]
pub trait Port: Send + Sync {
    /// Sonda de saúde do servidor.
    ///
    /// # Errors
    /// Devolve [`PortError`] se o transporte falhar.
    async fn health(&self) -> Result<Health, PortError>;

    /// Cria uma sessão nova ou retoma uma existente.
    ///
    /// # Errors
    /// Devolve [`PortError`] se o transporte falhar.
    async fn create_session(&self, session_id: &str) -> Result<SessionInfo, PortError>;

    /// Envia um prompt e devolve o id do turno aceite.
    ///
    /// # Errors
    /// Devolve [`PortError`] se a sessão não existir ou o transporte falhar.
    async fn turn(&self, session_id: &str, prompt: &str) -> Result<u64, PortError>;

    /// Cancela o turno em curso.
    ///
    /// # Errors
    /// Devolve [`PortError`] se a sessão não existir ou o transporte falhar.
    async fn cancel(&self, session_id: &str) -> Result<(), PortError>;

    /// Entrega uma mensagem de *steering* ao turno em curso (fronteira segura).
    ///
    /// # Errors
    /// Devolve [`PortError`] se a sessão não existir ou o transporte falhar.
    async fn steer(&self, session_id: &str, text: &str) -> Result<(), PortError>;

    /// Muda o modo da sessão (`approval` liberta `plan→build` — I15).
    ///
    /// # Errors
    /// Devolve [`PortError`] se a transição for recusada ou o transporte falhar.
    async fn set_mode(
        &self,
        session_id: &str,
        mode: Mode,
        approval: Approval,
    ) -> Result<(), PortError>;

    /// Estado corrente da sessão.
    ///
    /// # Errors
    /// Devolve [`PortError`] se a sessão não existir ou o transporte falhar.
    async fn state(&self, session_id: &str) -> Result<SessionState, PortError>;

    /// Subscreve os eventos de uma sessão (o front-end é um espectador).
    fn subscribe(&self, session_id: &str) -> EventStream;
}
