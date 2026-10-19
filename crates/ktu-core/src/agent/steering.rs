//! *Steering*: **admissão** durável e **entrega** numa fronteira segura (M7).
//!
//! Uma mensagem admitida entra no log (`Event::SteerQueued`) **antes** de ser entregue
//! (`Event::Steer`); o `id` vem da borda, pelo que repetir o pedido não duplica a
//! mensagem. Uma admissão que fique pendente (turno acabou, *crash*) é entregue na
//! primeira fronteira do turno seguinte — é o mesmo caminho para os dois casos.

use super::deps::Deps;
use super::logging::log_event;
use crate::events;
use crate::ports::Level;
use crate::session::{Event, SessionLog};

/// Entrega o *steering* admitido e admite o que chegou agora, por esta ordem.
pub(super) fn inject(log: &mut SessionLog, deps: &Deps<'_>) {
    for message in log.pending_steers() {
        log_event(
            deps,
            Level::Info,
            events::STEER,
            "mensagem de steering entregue",
            &[("steer_id", message.id.as_str())],
        );
        log.push(Event::Steer {
            id: message.id,
            text: message.text,
        });
    }
    let Some(steering) = deps.capabilities.steering else {
        return;
    };
    for message in steering.take() {
        if log.knows_steer(&message.id) {
            continue;
        }
        log_event(
            deps,
            Level::Info,
            events::STEER,
            "mensagem de steering admitida",
            &[("steer_id", message.id.as_str())],
        );
        log.push(Event::SteerQueued {
            id: message.id,
            text: message.text,
        });
    }
}
