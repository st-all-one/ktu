//! O dono **único** do tecto de contexto (I14).
//!
//! Nenhum outro módulo mantém um tecto próprio. Atingir o tecto é recusa, nunca
//! “cortar evidência”: a última mensagem do utilizador é a unidade que a
//! montagem **nunca** deixa cair (§2.10).

use crate::domain::Message;

/// O único dono do tecto de contexto.
#[derive(Debug, Clone, Copy)]
pub struct BudgetGate {
    max_tokens: usize,
}

impl BudgetGate {
    /// Cria um tecto de contexto em *tokens*.
    #[must_use]
    pub const fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    /// Estimativa conservadora de tokens (≈ 4 bytes/token).
    #[must_use]
    pub fn estimate(system: &str, messages: &[Message]) -> usize {
        let mut bytes = system.len();
        for message in messages {
            bytes = bytes.saturating_add(message_bytes(message));
        }
        bytes.div_ceil(4)
    }

    /// Indica se o pedido cabe no tecto.
    #[must_use]
    pub fn fits(self, system: &str, messages: &[Message]) -> bool {
        Self::estimate(system, messages) <= self.max_tokens
    }
}

/// Índice da última mensagem do utilizador — a unidade **nunca** descartada.
#[must_use]
pub fn pin_last_user(messages: &[Message]) -> Option<usize> {
    messages
        .iter()
        .rposition(|message| matches!(message, Message::User { .. }))
}

/// Bytes estimados de uma mensagem.
fn message_bytes(message: &Message) -> usize {
    match message {
        Message::Assistant {
            content,
            tool_calls,
        } => tool_calls.iter().fold(content.len(), |bytes, call| {
            bytes
                .saturating_add(call.name.len())
                .saturating_add(call.arguments.len())
        }),
        Message::System { content } | Message::User { content } | Message::Tool { content, .. } => {
            content.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BudgetGate, pin_last_user};
    use crate::domain::Message;

    #[test]
    fn estimate_and_fit() {
        let messages = vec![Message::User {
            content: "a".repeat(40),
        }];
        assert_eq!(BudgetGate::estimate("", &messages), 10);
        assert!(!BudgetGate::new(9).fits("", &messages));
        assert!(BudgetGate::new(10).fits("", &messages));
        assert!(BudgetGate::new(1000).fits("", &messages));
    }

    #[test]
    fn pin_returns_the_last_user_index() {
        let messages = vec![
            Message::System {
                content: "s".to_owned(),
            },
            Message::User {
                content: "um".to_owned(),
            },
            Message::Assistant {
                content: String::new(),
                tool_calls: Vec::new(),
            },
            Message::User {
                content: "dois".to_owned(),
            },
        ];
        assert_eq!(pin_last_user(&messages), Some(3));
    }
}
