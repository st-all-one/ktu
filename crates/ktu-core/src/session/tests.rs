//! Testes do log da sessão.

use super::{Event, LogViolation, SessionLog};
use crate::domain::{Message, StopReason, ToolCall, ToolCallId};

fn call(id: &str, name: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new(id),
        name: name.to_owned(),
        arguments: "{}".to_owned(),
    }
}

#[test]
fn derive_messages_projects_one_assistant_with_n_tools() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "le dois ficheiros".to_owned(),
    });
    log.push(Event::Assistant {
        content: String::new(),
        reasoning: String::new(),
        tool_calls: vec![call("a", "read_file"), call("b", "read_file")],
    });
    log.push(Event::ToolResult {
        tool_call_id: ToolCallId::new("a"),
        name: "read_file".to_owned(),
        content: "A".to_owned(),
        is_error: false,
    });
    log.push(Event::ToolResult {
        tool_call_id: ToolCallId::new("b"),
        name: "read_file".to_owned(),
        content: "B".to_owned(),
        is_error: false,
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });

    let messages = log.derive_messages();
    assert_eq!(messages.len(), 4);
    assert!(matches!(messages.first(), Some(Message::User { .. })));
    let assistant_index = messages
        .iter()
        .position(|message| matches!(message, Message::Assistant { .. }));
    assert_eq!(assistant_index, Some(1));
}

#[test]
fn verify_accepts_a_closed_consistent_turn() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "ola".to_owned(),
    });
    log.push(Event::Assistant {
        content: "adeus".to_owned(),
        reasoning: "pensei".to_owned(),
        tool_calls: Vec::new(),
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });
    assert_eq!(log.verify(), Vec::new());
}

#[test]
fn verify_flags_open_turn_and_missing_result() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "ola".to_owned(),
    });
    log.push(Event::Assistant {
        content: String::new(),
        reasoning: String::new(),
        tool_calls: vec![call("x", "read_file")],
    });
    let violations = log.verify();
    assert!(violations.contains(&LogViolation::OpenTurn));
    assert!(violations.contains(&LogViolation::MissingToolResult(ToolCallId::new("x"))));
}

#[test]
fn wire_projection_is_deterministic_and_prefix_stable() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "primeira".to_owned(),
    });
    log.push(Event::Assistant {
        content: "feito".to_owned(),
        reasoning: "penso".to_owned(),
        tool_calls: Vec::new(),
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });
    // O prefixo da projeção é reproduzível byte-a-byte (cache de prefixo).
    let before = log.derive_messages();
    assert_eq!(before, log.derive_messages());
    // Acrescentar um turno não altera as mensagens anteriores.
    log.push(Event::TurnStart {
        prompt: "segunda".to_owned(),
    });
    let after = log.derive_messages();
    assert_eq!(after.get(..before.len()), Some(before.as_slice()));
    assert!(after.len() > before.len());
    // O pensamento nunca é reenviado ao modelo.
    assert_eq!(before.len(), 2);
}

/// Uma mensagem admitida e não entregue fica **pendente** — é o que o turno seguinte entrega.
#[test]
fn a_queued_steer_is_pending_until_delivered() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "faz".to_owned(),
    });
    log.push(Event::SteerQueued {
        id: "s1".to_owned(),
        text: "antes isto".to_owned(),
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });
    assert_eq!(log.pending_steers().len(), 1);
    assert!(log.knows_steer("s1"));
    // A admissão **não** entra no contexto: só a entrega projeta (model-visible ⟺ logged).
    let messages = log.derive_messages();
    assert!(
        !messages
            .iter()
            .any(|message| matches!(message, Message::User { content } if content == "antes isto"))
    );

    log.push(Event::Steer {
        id: "s1".to_owned(),
        text: "antes isto".to_owned(),
    });
    assert_eq!(log.pending_steers(), Vec::new());
    let messages = log.derive_messages();
    assert_eq!(
        messages
            .iter()
            .filter(
                |message| matches!(message, Message::User { content } if content == "antes isto")
            )
            .count(),
        1
    );
}

/// O mesmo `id` entregue duas vezes (retry do front, retomada) entra **uma só vez** no contexto.
#[test]
fn a_repeated_steer_id_is_projected_once() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "faz".to_owned(),
    });
    for _ in 0..2 {
        log.push(Event::Steer {
            id: "s1".to_owned(),
            text: "antes isto".to_owned(),
        });
    }
    log.push(Event::Steer {
        id: "s2".to_owned(),
        text: "antes isto".to_owned(),
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });
    let count = log
        .derive_messages()
        .iter()
        .filter(|message| matches!(message, Message::User { content } if content == "antes isto"))
        .count();
    assert_eq!(count, 2, "mesmo id uma vez; ids distintos duas");
    assert_eq!(log.verify(), Vec::new());
}

/// A admissão repetida (mesmo id) não fica pendente duas vezes.
#[test]
fn a_repeated_queued_steer_id_is_pending_once() {
    let mut log = SessionLog::new();
    for _ in 0..3 {
        log.push(Event::SteerQueued {
            id: "s1".to_owned(),
            text: "antes isto".to_owned(),
        });
    }
    assert_eq!(log.pending_steers().len(), 1);
}
