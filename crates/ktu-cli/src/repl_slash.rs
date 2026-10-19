//! Comandos `/` embutidos do REPL — cada um **delega** num verbo existente (M4).
//!
//! O REPL não tem lógica própria: `/plan`, `/build`, `/mode` e `/cancel` são comandos da
//! porta `Host`; `/compact` pede a compactação ao actor (que grava log + checkpoint);
//! `/fork`, `/memory` e `/resume` reutilizam a **mesma** função que `ktu session fork`,
//! `ktu memory list` e `ktu session` — nunca uma segunda implementação.

use std::path::Path;

use ktu_core::command::Command;
use ktu_core::plan::{Approval, Mode};
use ktu_port::host::Host;
use ktu_port::port::EventStream;

use crate::hub::SessionHub;
use crate::memory_cmd;
use crate::repl::log;
use crate::session_cmd;
use crate::slash::{Slash, help};

/// Mudança de sessão pedida por `/resume` (o REPL é dono do estado do loop).
pub(crate) struct Switch {
    /// Identificador da sessão nova.
    pub(crate) id: String,
    /// Fluxo de eventos da sessão nova.
    pub(crate) events: EventStream,
    /// Modo da sessão nova.
    pub(crate) mode: Mode,
}

/// Aplica um comando `/` embutido; `Some` quando o REPL tem de **mudar de sessão**.
///
/// # Errors
/// Devolve mensagem legível se o comando da porta falhar (sessão inexistente, turno em
/// curso, ficheiro de sessão já existente).
#[allow(
    clippy::too_many_arguments,
    reason = "passagem directa do estado do REPL; sem ramificacoes"
)]
pub(crate) async fn apply(
    hub: &SessionHub,
    root: &Path,
    id: &str,
    slash: Slash,
    mode: &mut Mode,
    commands: &[Command],
) -> Result<Option<Switch>, String> {
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
        Slash::Compact => {
            hub.compact(id).await.map_err(|error| error.to_string())?;
            log("contexto compactado (o log cru fica intacto)");
        }
        Slash::Fork { to } => {
            if to.is_empty() {
                return Err("/fork exige o id de destino: /fork <novo>".to_owned());
            }
            session_cmd::copy(root, id, &to)?;
            log(&format!(
                "sessao copiada: {id} -> {to} (abra com /resume {to})"
            ));
        }
        Slash::Memory => {
            let text = memory_cmd::list_text(root)?;
            for line in text.lines() {
                log(line);
            }
        }
        Slash::Resume { id: target } => {
            let Some(target) = target else {
                let ids = session_cmd::ids(root);
                let text = if ids.is_empty() {
                    "nenhuma".to_owned()
                } else {
                    ids.join(", ")
                };
                log(&format!("sessoes: {text}"));
                return Ok(None);
            };
            hub.create_session(&target)
                .await
                .map_err(|error| error.to_string())?;
            let events = hub.subscribe(&target);
            let mode = hub
                .state(&target)
                .await
                .map_err(|error| error.to_string())?
                .mode;
            log(&format!("sessao: {target} [{mode:?}]"));
            return Ok(Some(Switch {
                id: target,
                events,
                mode,
            }));
        }
    }
    Ok(None)
}
