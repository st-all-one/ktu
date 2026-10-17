//! Smoke para `cargo miri`: o núcleo é `#![forbid(unsafe_code)]`, mas o teste
//! exerce as operações puras mais usadas (log, projecção e verificação) sob o
//! interpretador da Miri, à procura de UB/aliasing invisível.
//!
//! Determinístico e sem I/O (não usa tokio nem o sistema de ficheiros).

use ktu_core::domain::{StopReason, ToolCall, ToolCallId};
use ktu_core::session::{Event, SessionLog};

#[test]
fn core_log_round_trip_is_defined() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "pergunta".to_owned(),
    });
    let call = ToolCall {
        id: ToolCallId::new("c1"),
        name: "read_file".to_owned(),
        arguments: r#"{"path":"a.txt"}"#.to_owned(),
    };
    log.push(Event::Assistant {
        content: String::new(),
        reasoning: "penso".to_owned(),
        tool_calls: vec![call.clone()],
    });
    log.push(Event::GuardDecision {
        tool: call.name.clone(),
        allowed: true,
        reason: "allow".to_owned(),
    });
    log.push(Event::ToolResult {
        tool_call_id: call.id,
        name: call.name,
        content: "conteudo".to_owned(),
        is_error: false,
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });

    assert_eq!(log.verify(), Vec::new());
    let messages = log.derive_messages();
    assert_eq!(messages.len(), 3);
}
