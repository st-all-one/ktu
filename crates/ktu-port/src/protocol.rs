//! Protocolo de fio versionado (NDJSON/SSE sobre *stream*).
//!
//! O protocolo é **aditivo**: novos campos e variantes podem ser acrescentados
//! sem quebrar front-ends; renomear ou remover um id é uma quebra de contrato.
//! Os eventos do log são a **única** fonte da verdade (I3): o front-end é um
//! espectador, nunca a fonte.

use ktu_core::domain::StopReason;
use ktu_core::plan::{Approval, Mode};
use ktu_core::session::Event;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Versão do protocolo de fio, exposta no prefixo `/v1`.
pub const PROTOCOL_VERSION: u16 = 1;

/// Pedido de um front-end (cliente → servidor).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Cria uma sessão nova ou retoma a existente.
    CreateSession {
        /// Identificador da sessão.
        session_id: String,
    },
    /// Envia um prompt (aceite de imediato; corre em *background*).
    Turn {
        /// Identificador da sessão.
        session_id: String,
        /// Instrução do utilizador.
        prompt: String,
    },
    /// Cancela o turno em curso.
    Cancel {
        /// Identificador da sessão.
        session_id: String,
    },
    /// Muda o modo da sessão (`SetMode`, sempre logado — I15).
    SetMode {
        /// Identificador da sessão.
        session_id: String,
        /// Modo alvo.
        mode: Mode,
        /// Aprovação explícita (liberta `plan→build`).
        approval: Approval,
    },
    /// Pede o estado corrente.
    State {
        /// Identificador da sessão.
        session_id: String,
    },
    /// Sonda de saúde do servidor.
    Health,
}

/// Mensagem de um servidor (servidor → front-end).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    /// Sessão pronta (nova ou retomada).
    SessionReady {
        /// Identificador da sessão.
        session_id: String,
        /// `true` se a sessão foi retomada de disco.
        resumed: bool,
    },
    /// Turno aceite.
    TurnAccepted {
        /// Id monotónico do turno dentro da sessão.
        turn_id: u64,
    },
    /// Evento do log — a única fonte da verdade (I3).
    Log {
        /// Evento append-only do log.
        event: Event,
    },
    /// Turno terminado (I5).
    TurnFinished {
        /// Razão do término.
        reason: StopReason,
    },
    /// Estado corrente da sessão.
    State {
        /// Estado observável.
        state: SessionState,
    },
    /// Saúde do servidor.
    Health {
        /// Relatório de saúde.
        health: Health,
    },
    /// Erro de protocolo ou de execução (nunca faz *panic*).
    Error {
        /// Mensagem legível.
        message: String,
    },
}

/// Resultado de preparar uma sessão.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Identificador da sessão.
    pub session_id: String,
    /// `true` se a sessão foi retomada de disco.
    pub resumed: bool,
}

/// Estado observável de uma sessão (inclui o log, para o front-end arrancar
/// sem voltar a pedir os eventos um a um).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionState {
    /// Identificador da sessão.
    pub session_id: String,
    /// Modo corrente (`build`/`plan`).
    pub mode: Mode,
    /// `true` se há um turno em curso.
    pub pending: bool,
    /// Corte de compactação do contexto (`None` = contexto completo): índice do primeiro
    /// evento que a projecção mantém. Vive no *checkpoint*, não no log (que fica intacto).
    pub keep_from: Option<usize>,
    /// Log append-only completo (a verdade da sessão — I3).
    pub log: Vec<Event>,
}

/// Saúde do servidor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    /// Nome do provider activo.
    pub provider: String,
    /// Nome/alias do modelo.
    pub model: String,
    /// Tools oferecidas ao modelo.
    pub tools: Vec<String>,
    /// `true` se a memória persistente está ligada.
    pub memory: bool,
}

/// Erro da porta (a borda degrada, nunca aborta).
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    /// Falha de transporte (socket, canal fechado).
    #[error("transporte: {0}")]
    Transport(String),
    /// Mensagem inválida ou inesperada.
    #[error("protocolo: {0}")]
    Protocol(String),
    /// Sessão inexistente.
    #[error("sessao inexistente: {0}")]
    NoSession(String),
}

/// Codifica um valor como uma linha NDJSON (terminada em `\n`).
///
/// # Errors
/// Devolve [`PortError::Protocol`] se o valor não for serializável para JSON.
pub fn encode_line<T: Serialize>(value: &T) -> Result<String, PortError> {
    let mut line =
        serde_json::to_string(value).map_err(|error| PortError::Protocol(error.to_string()))?;
    line.push('\n');
    Ok(line)
}

/// Descoda uma linha NDJSON (espaços em volta são ignorados).
///
/// # Errors
/// Devolve [`PortError::Protocol`] se a linha não for JSON válido do tipo `T`.
pub fn decode_line<T: DeserializeOwned>(line: &str) -> Result<T, PortError> {
    serde_json::from_str(line).map_err(|error| PortError::Protocol(error.to_string()))
}

#[cfg(test)]
mod tests {
    use ktu_core::domain::StopReason;
    use ktu_core::plan::{Approval, Mode};
    use ktu_core::session::Event;

    use super::{ClientMsg, ServerMsg, decode_line, encode_line};

    #[test]
    fn client_messages_round_trip_as_ndjson() -> Result<(), Box<dyn std::error::Error>> {
        let messages = [
            ClientMsg::CreateSession {
                session_id: "s1".to_owned(),
            },
            ClientMsg::Turn {
                session_id: "s1".to_owned(),
                prompt: "ola".to_owned(),
            },
            ClientMsg::Cancel {
                session_id: "s1".to_owned(),
            },
            ClientMsg::SetMode {
                session_id: "s1".to_owned(),
                mode: Mode::Plan,
                approval: Approval::Approved,
            },
            ClientMsg::State {
                session_id: "s1".to_owned(),
            },
            ClientMsg::Health,
        ];
        for message in messages {
            let line = encode_line(&message)?;
            assert!(line.ends_with('\n'));
            let decoded: ClientMsg = decode_line(line.trim_end())?;
            assert_eq!(decoded, message);
        }
        Ok(())
    }

    #[test]
    fn server_messages_round_trip_as_ndjson() -> Result<(), Box<dyn std::error::Error>> {
        let messages = [
            ServerMsg::SessionReady {
                session_id: "s1".to_owned(),
                resumed: false,
            },
            ServerMsg::TurnAccepted { turn_id: 3 },
            ServerMsg::Log {
                event: Event::TurnStart {
                    prompt: "ola".to_owned(),
                },
            },
            ServerMsg::TurnFinished {
                reason: StopReason::EndTurn,
            },
            ServerMsg::Error {
                message: "falhou".to_owned(),
            },
        ];
        for message in messages {
            let line = encode_line(&message)?;
            let decoded: ServerMsg = decode_line(line.trim_end())?;
            assert_eq!(decoded, message);
        }
        Ok(())
    }

    #[test]
    fn invalid_json_is_a_protocol_error() {
        let error = decode_line::<ClientMsg>("nao e json");
        assert!(matches!(error, Err(super::PortError::Protocol(_))));
    }
}
