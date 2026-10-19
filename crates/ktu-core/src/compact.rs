//! Compactação **aditiva** do contexto (§2.7 / F4).
//!
//! O log cru (`events`) permanece intacto no disco (fonte da verdade); a
//! compactação apenas marca um **ponto de corte**: os eventos anteriores são
//! substituídos, na projecção de *wire*, por um resumo determinístico. É
//! reversível (basta limpar o corte) e reproduzível byte-a-byte.

use crate::session::Event;

/// Tamanho máximo de cada pedido no resumo (caracteres).
const MAX_PROMPT_CHARS: usize = 80;

/// Resumo determinístico dos eventos descartados pelo corte.
#[must_use]
pub fn summarize(events: &[Event]) -> String {
    let mut prompts = Vec::new();
    let mut tools = 0usize;
    for event in events {
        match event {
            Event::TurnStart { prompt } => prompts.push(truncate(prompt)),
            Event::Steer { text, .. } => prompts.push(truncate(text)),
            Event::Assistant { tool_calls, .. } => {
                tools = tools.saturating_add(tool_calls.len());
            }
            _ => {}
        }
    }
    if prompts.is_empty() && tools == 0 {
        return String::new();
    }
    let mut out = String::from("[contexto compactado: ");
    out.push_str(&prompts.len().to_string());
    out.push_str(" turno(s) anteriores resumidos; ");
    out.push_str(&tools.to_string());
    out.push_str(" chamada(s) de tool. Pedidos: ");
    out.push_str(&prompts.join(" | "));
    out.push(']');
    out
}

/// Índice a partir do qual ficam os últimos `keep_turns` turnos completos.
///
/// `None` quando não há turnos a compactar (o log cabe).
#[must_use]
pub fn keep_from_for_turns(events: &[Event], keep_turns: usize) -> Option<usize> {
    let starts: Vec<usize> = events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| matches!(event, Event::TurnStart { .. }).then_some(index))
        .collect();
    if starts.len() <= keep_turns {
        return None;
    }
    let position = starts.len().saturating_sub(keep_turns);
    starts.get(position).copied()
}

fn truncate(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= MAX_PROMPT_CHARS {
        return trimmed.to_owned();
    }
    let mut out: String = trimmed.chars().take(MAX_PROMPT_CHARS).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use crate::domain::StopReason;
    use crate::session::Event;

    use super::{keep_from_for_turns, summarize};

    fn turn(prompt: &str) -> Vec<Event> {
        vec![
            Event::TurnStart {
                prompt: prompt.to_owned(),
            },
            Event::Assistant {
                content: "ok".to_owned(),
                reasoning: String::new(),
                tool_calls: Vec::new(),
            },
            Event::TurnEnd {
                reason: StopReason::EndTurn,
            },
        ]
    }

    #[test]
    fn keeps_the_last_turns_and_summarizes_the_rest() {
        let mut events = Vec::new();
        for prompt in ["um", "dois", "tres"] {
            events.extend(turn(prompt));
        }
        let from = keep_from_for_turns(&events, 1);
        assert_eq!(from, Some(6));
        let summary = summarize(events.get(..6).unwrap_or_default());
        assert!(summary.contains("2 turno(s)"));
        assert!(summary.contains("um"));
        assert!(summary.contains("dois"));
    }

    #[test]
    fn nothing_to_compact_when_it_fits() {
        let events = turn("so um");
        assert_eq!(keep_from_for_turns(&events, 4), None);
        assert_eq!(summarize(&[]), "");
    }
}
