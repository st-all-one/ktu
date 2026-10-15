//! *Fallback* de parsing para quando o template não separa o pensamento nem
//! emite tool-calls nativas.
//!
//! O ktu prefere tool-calls nativas; mas alguns GGUF devolvem o formato em
//! texto (`<tool_call>{...}</tool_call>`) e o pensamento inline (` thinking`).

use ktu_core::domain::{ToolCall, ToolCallId};
use serde_json::Value;

const THINK_OPEN: &str = concat!("<", "think", ">");
const THINK_CLOSE: &str = concat!("<", "/think", ">");
const CALL_OPEN: &str = "<tool_call>";
const CALL_CLOSE: &str = "</tool_call>";

/// Indica se o texto contém uma tag de pensamento inline.
#[must_use]
pub fn has_thinking(text: &str) -> bool {
    text.contains(THINK_OPEN)
}

/// Separa o pensamento inline (` thinking...<｜end▁of▁thinking｜>`) do corpo.
///
/// Devolve `(reasoning, content)`; sem tag de pensamento, `reasoning` é vazio.
#[must_use]
pub fn split_thinking(text: &str) -> (String, String) {
    let Some(start) = text.find(THINK_OPEN) else {
        return (String::new(), text.to_owned());
    };
    let after_open = text
        .get(start.saturating_add(THINK_OPEN.len())..)
        .unwrap_or("");
    let before = text.get(..start).unwrap_or("");
    let Some(end) = after_open.find(THINK_CLOSE) else {
        return (after_open.to_owned(), before.to_owned());
    };
    let reasoning = after_open.get(..end).unwrap_or("");
    let tail = after_open
        .get(end.saturating_add(THINK_CLOSE.len())..)
        .unwrap_or("");
    (reasoning.to_owned(), format!("{before}{tail}"))
}

/// Extrai chamadas de tool no formato de texto `<tool_call>{...}</tool_call>`.
#[must_use]
pub fn parse_text_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(CALL_OPEN) {
        let after_open = rest
            .get(start.saturating_add(CALL_OPEN.len())..)
            .unwrap_or("");
        let Some(end) = after_open.find(CALL_CLOSE) else {
            break;
        };
        let raw = after_open.get(..end).unwrap_or("");
        if let Some(call) = parse_one(raw, calls.len()) {
            calls.push(call);
        }
        rest = after_open
            .get(end.saturating_add(CALL_CLOSE.len())..)
            .unwrap_or("");
    }
    calls
}

/// Remove os blocos `<tool_call>...</tool_call>` do texto.
#[must_use]
pub fn strip_text_tool_calls(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(CALL_OPEN) {
        out.push_str(rest.get(..start).unwrap_or(""));
        let after_open = rest
            .get(start.saturating_add(CALL_OPEN.len())..)
            .unwrap_or("");
        let Some(end) = after_open.find(CALL_CLOSE) else {
            break;
        };
        rest = after_open
            .get(end.saturating_add(CALL_CLOSE.len())..)
            .unwrap_or("");
    }
    out.push_str(rest);
    out
}

fn parse_one(raw: &str, ordinal: usize) -> Option<ToolCall> {
    let value: Value = serde_json::from_str(raw.trim()).ok()?;
    let name = value.get("name").and_then(Value::as_str)?.to_owned();
    let arguments = match value.get("arguments") {
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
        None => "{}".to_owned(),
    };
    let id = value
        .get("id")
        .and_then(Value::as_str)
        .map_or_else(|| format!("text_{ordinal}"), str::to_owned);
    Some(ToolCall {
        id: ToolCallId::new(id),
        name,
        arguments,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_text_tool_calls, split_thinking, strip_text_tool_calls};

    #[test]
    fn splits_inline_thinking() {
        let (reasoning, content) = split_thinking(concat!(
            "<",
            "think",
            ">",
            "penso muito",
            "<",
            "/think",
            ">",
            " e respondo"
        ));
        assert_eq!(reasoning, "penso muito");
        assert_eq!(content, " e respondo");
    }

    #[test]
    fn no_thinking_leaves_content_intact() {
        let (reasoning, content) = split_thinking("so resposta");
        assert_eq!(reasoning, "");
        assert_eq!(content, "so resposta");
    }

    #[test]
    fn parses_and_strips_text_tool_calls() {
        let text =
            r#"vejo <tool_call>{"name":"read_file","arguments":{"path":"a"}}</tool_call> fim"#;
        let calls = parse_text_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls.first().map(|c| c.name.as_str()), Some("read_file"));
        assert_eq!(
            calls.first().map(|c| c.arguments.as_str()),
            Some(r#"{"path":"a"}"#)
        );
        assert_eq!(strip_text_tool_calls(text), "vejo  fim");
    }
}
