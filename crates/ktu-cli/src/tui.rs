//! `ktu tui` — TUI mínima (ratatui) como **cliente** da porta única.
//!
//! Não toca no kernel: liga-se a um `ktu serve` por [`HttpPort`] e mostra os
//! eventos do log em tempo real, enviando prompts pela mesma porta (§2.3/F3).

use std::time::Duration;

use ktu_core::session::Event;
use ktu_port::HttpPort;
use ktu_port::port::Port;
use ktu_port::protocol::ServerMsg;
use ratatui::crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};
use tokio::runtime::Builder;
use tokio::time::sleep;

/// Corre a TUI ligada a `server` (URL base) na sessão `session`.
///
/// # Errors
/// Devolve mensagem legível se a porta ou o terminal falharem.
pub(crate) fn run(server: &str, session: &str) -> Result<(), String> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("nao foi possivel iniciar o runtime: {error}"))?;
    runtime.block_on(app(server, session))
}

/// Restaura o terminal quando a app termina (mesmo com erro).
struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

async fn app(server: &str, session: &str) -> Result<(), String> {
    let port = HttpPort::new(server);
    port.create_session(session)
        .await
        .map_err(|error| error.to_string())?;
    let state = port
        .state(session)
        .await
        .map_err(|error| error.to_string())?;
    let mut log: Vec<String> = state.log.iter().map(describe).collect();
    let mut input = String::new();
    let mut events = port.subscribe(session);

    let mut terminal = ratatui::init();
    let _restore = RestoreTerminal;
    let mut running = true;
    while running {
        terminal
            .draw(|frame| draw(frame, &log, &input))
            .map_err(|error| error.to_string())?;
        while event::poll(Duration::from_millis(0)).map_err(|error| error.to_string())? {
            let event::Event::Key(key) = event::read().map_err(|error| error.to_string())? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') if input.is_empty() => running = false,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    running = false;
                }
                KeyCode::Char(character) => input.push(character),
                KeyCode::Backspace => {
                    let _popped = input.pop();
                }
                KeyCode::Enter => {
                    let prompt = std::mem::take(&mut input);
                    if !prompt.is_empty()
                        && let Err(error) = port.turn(session, &prompt).await
                    {
                        log.push(format!("erro: {error}"));
                    }
                }
                _ => {}
            }
        }
        while let Ok(message) = events.try_recv() {
            match message {
                ServerMsg::Log { event } => log.push(describe(&event)),
                ServerMsg::TurnFinished { reason } => log.push(format!("— fim: {reason:?}")),
                ServerMsg::Error { message } => log.push(format!("erro: {message}")),
                _ => {}
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    Ok(())
}

/// Desenha a vista: eventos em cima, linha de prompt em baixo.
fn draw(frame: &mut ratatui::Frame<'_>, log: &[String], input: &str) {
    let area = frame.area();
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(area);
    let list_area = chunks.first().copied().unwrap_or(area);
    let input_area = chunks.get(1).copied().unwrap_or(area);
    let visible = usize::from(list_area.height.saturating_sub(2));
    let lines: Vec<Line<'_>> = log
        .iter()
        .rev()
        .take(visible)
        .rev()
        .map(|line| Line::from(line.as_str()))
        .collect();
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title("eventos"))
            .wrap(Wrap { trim: false }),
        list_area,
    );
    frame.render_widget(
        Paragraph::new(input)
            .block(Block::bordered().title("prompt — Enter envia · q sai"))
            .style(Style::default().fg(Color::Yellow)),
        input_area,
    );
}

/// Texto estável de um evento do log.
fn describe(event: &Event) -> String {
    match event {
        Event::TurnStart { prompt } => format!("» {prompt}"),
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

/// Trunca preservando fronteiras de caractere.
fn truncate(text: &str, max: usize) -> String {
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

    use super::{describe, truncate};

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
}
