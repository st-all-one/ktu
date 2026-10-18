//! A superfície que o **servidor** implementa (`Host`): uma só interface para
//! todos os transportes. A implementação (o runtime do kernel) vive fora do
//! `ktu-port`, que não contém lógica de agente.

use std::sync::Arc;

use ktu_core::plan::{Approval, Mode};

use crate::port::EventStream;
use crate::protocol::{Health, PortError, SessionInfo, SessionState};

/// Serviço de sessões atrás da porta.
#[async_trait::async_trait]
pub trait Host: Send + Sync {
    /// Saúde do servidor.
    async fn health(&self) -> Health;

    /// Cria/retoma a sessão.
    ///
    /// # Errors
    /// Devolve [`PortError`] se a sessão não puder ser preparada.
    async fn create_session(&self, session_id: &str) -> Result<SessionInfo, PortError>;

    /// Aceita um prompt e devolve o id do turno.
    ///
    /// # Errors
    /// Devolve [`PortError::NoSession`] se a sessão não existir.
    async fn turn(&self, session_id: &str, prompt: &str) -> Result<u64, PortError>;

    /// Cancela o turno em curso.
    ///
    /// # Errors
    /// Devolve [`PortError::NoSession`] se a sessão não existir.
    async fn cancel(&self, session_id: &str) -> Result<(), PortError>;

    /// Entrega uma mensagem de *steering* ao turno em curso (fronteira segura).
    ///
    /// # Errors
    /// Devolve [`PortError::NoSession`] se a sessão não existir.
    async fn steer(&self, session_id: &str, text: &str) -> Result<(), PortError>;

    /// Muda o modo da sessão.
    ///
    /// # Errors
    /// Devolve [`PortError`] se a transição for recusada ou a sessão não existir.
    async fn set_mode(
        &self,
        session_id: &str,
        mode: Mode,
        approval: Approval,
    ) -> Result<(), PortError>;

    /// Estado corrente da sessão.
    ///
    /// # Errors
    /// Devolve [`PortError::NoSession`] se a sessão não existir.
    async fn state(&self, session_id: &str) -> Result<SessionState, PortError>;

    /// Subscreve os eventos da sessão.
    fn subscribe(&self, session_id: &str) -> EventStream;
}

/// Referência partilhada para o serviço (usada pelos transportes).
pub type SharedHost = Arc<dyn Host>;
