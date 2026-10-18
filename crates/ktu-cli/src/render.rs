//! Renderização textual de eventos do log (partilhada por `tui` e `session`).
//!
//! O texto é **estável** (nunca inclui ids efémeros). A cor degrada para texto
//! simples quando a saída não é um terminal (*pipe-friendly*).

use ktu_core::session::Event;

/// Estilo de saída: com cor (terminal) ou simples (pipe/ficheiro).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Style {
    /// Sem cor — destino não interactivo.
    Plain,
    /// Com cor ANSI — terminal.
    Color,
}

impl Style {
    /// Envolve `text` num código ANSI quando colorido.
    fn paint(self, code: &str, text: &str) -> String {
        match self {
            Self::Plain => text.to_owned(),
            Self::Color => format!("\u{1b}[{code}m{text}\u{1b}[0m"),
        }
    }
}

/// Texto estável de um evento do log.
pub(crate) fn describe(event: &Event) -> String {
    match event {
        Event::TurnStart { prompt } => format!("» {prompt}"),
        Event::Steer { text } => format!("» (steer) {text}"),
        Event::Assistant {
            content,
            tool_calls,
            ..
        } => {
            if tool_calls.is_empty() {
                format!("agente: {content}")
            } else {
                format!("agente: {} chamada(s) de tool", tool_calls.len())
            }
        }
        Event::ToolResult {
            name,
            content,
            is_error,
            ..
        } => {
            let mark = if *is_error { "x" } else { "ok" };
            format!("  [{mark}] {name}: {}", truncate(content, 160))
        }
        Event::GuardDecision {
            tool,
            allowed,
            reason,
        } => {
            let mark = if *allowed { "allow" } else { "deny" };
            format!("  guard {mark} {tool} ({reason})")
        }
        Event::TurnEnd { reason } => format!("— fim: {reason:?}"),
        Event::ModeSet { mode } => format!("modo: {mode:?}"),
        Event::PluginDispatch {
            hook,
            generation,
            allowed,
            reason,
        } => {
            let mark = if *allowed { "pass" } else { "deny" };
            let detail = reason.as_deref().unwrap_or("");
            format!("  hook {mark} {hook} gen={generation} {detail}")
        }
    }
}

/// Texto estável de um evento, com a cor do papel (erro/negação a vermelho).
pub(crate) fn describe_styled(event: &Event, style: Style) -> String {
    let text = describe(event);
    match event {
        Event::ToolResult { is_error: true, .. } => style.paint("31", &text),
        Event::GuardDecision { allowed: false, .. }
        | Event::PluginDispatch { allowed: false, .. } => style.paint("33", &text),
        Event::TurnEnd { .. } => style.paint("90", &text),
        _ => text,
    }
}

/// Trunca preservando fronteiras de caractere.
pub(crate) fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut truncated: String = text.chars().take(max).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use ktu_core::domain::StopReason;
    use ktu_core::session::Event;

    use super::{Style, describe, describe_styled, truncate};

    #[test]
    fn describes_events_compactly() {
        let start = describe(&Event::TurnStart {
            prompt: "ola".to_owned(),
        });
        assert_eq!(start, "» ola");
        let end = describe(&Event::TurnEnd {
            reason: StopReason::EndTurn,
        });
        assert!(end.starts_with("— fim:"));
    }

    #[test]
    fn truncates_with_ellipsis() {
        assert_eq!(truncate("abcdef", 3), "abc…");
        assert_eq!(truncate("abc", 3), "abc");
    }

    #[test]
    fn color_is_added_only_in_color_style() {
        let event = Event::TurnEnd {
            reason: StopReason::EndTurn,
        };
        assert!(!describe_styled(&event, Style::Plain).contains('\u{1b}'));
        assert!(describe_styled(&event, Style::Color).contains('\u{1b}'));
    }
}
