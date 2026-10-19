//! O log da sessão — a **fonte da verdade** (I3).
//!
//! Tudo o que o modelo viu é reconstruível a partir do log: [`SessionLog::derive_messages`]
//! é a única fonte do pedido ao provider. Cada linha é um [`Event`] serializável
//! (a persistência em JSONL é feita pela borda, via a porta `Fs`).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::compact::summarize;
use crate::domain::{Message, StopReason, ToolCall, ToolCallId};
use crate::plan::Mode;

mod title;

pub use title::{MAX_CHARS, MAX_WORDS, title_from, title_of};

/// Um evento do log da sessão.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// Início de um turno (entrada do utilizador).
    TurnStart {
        /// Instrução do utilizador.
        prompt: String,
    },
    /// *Steering*: mensagem **admitida** (durável no log), ainda não entregue ao modelo.
    ///
    /// A admissão é o primeiro evento; a entrega ([`Event::Steer`]) referencia o **mesmo**
    /// `id` — uma mensagem admitida e não entregue sobrevive a um *crash* e é entregue no
    /// turno seguinte (M7).
    SteerQueued {
        /// Identificador dado pela **borda** (idempotência: o mesmo id entrega uma vez).
        id: String,
        /// Texto admitido.
        text: String,
    },
    /// *Steering*: mensagem **entregue** ao modelo (fronteira segura).
    Steer {
        /// Identificador da admissão correspondente.
        id: String,
        /// Texto entregue ao modelo.
        text: String,
    },
    /// Fim de um passo do modelo (texto, pensamento e/ou chamadas de tool).
    Assistant {
        /// Texto emitido (resposta final).
        content: String,
        /// Pensamento do modelo — campo próprio, nunca reenviado ao modelo.
        reasoning: String,
        /// Chamadas de tool emitidas no mesmo passo.
        tool_calls: Vec<ToolCall>,
    },
    /// Resultado de uma tool.
    ToolResult {
        /// Chamada a que este resultado responde.
        tool_call_id: ToolCallId,
        /// Nome da tool.
        name: String,
        /// Conteúdo do resultado.
        content: String,
        /// Se o resultado é um erro.
        is_error: bool,
    },
    /// Decisão de guardrail — a evidência da tese (§2.8).
    GuardDecision {
        /// Tool avaliada.
        tool: String,
        /// Se a decisão foi `Allow`.
        allowed: bool,
        /// Motivo tipado (texto estável).
        reason: String,
    },
    /// Fim do turno, com a razão (I5).
    TurnEnd {
        /// Razão do término.
        reason: StopReason,
    },
    /// Modo de sessão definido por um comando explícito (`SetMode`) — I15.
    ModeSet {
        /// Novo modo.
        mode: Mode,
    },
    /// *Dispatch* de um *hook* para os plugins — evidência da intercepção (§2.9).
    PluginDispatch {
        /// *Hook* despachado (nome estável).
        hook: String,
        /// Geração dos plugins (I12).
        generation: u32,
        /// Se o *dispatch* permitiu a acção.
        allowed: bool,
        /// Motivo da negação, se houver.
        reason: Option<String>,
    },
}

/// Mensagem de *steering*: `id` (dado pela borda) + texto.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteerMessage {
    /// Identificador da admissão (idempotência).
    pub id: String,
    /// Texto da mensagem.
    pub text: String,
}

/// Violação das invariantes do log detectada por [`SessionLog::verify`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogViolation {
    /// Um turno ficou aberto (I1).
    OpenTurn,
    /// Uma chamada de tool ficou sem resultado (I2).
    MissingToolResult(ToolCallId),
}

/// Log append-only de uma sessão.
#[derive(Debug, Default, Clone)]
pub struct SessionLog {
    events: Vec<Event>,
    compaction_from: Option<usize>,
}

impl SessionLog {
    /// Cria um log vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Devolve os eventos, por ordem.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Acrescenta um evento ao log (append-only).
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Marca o corte de compactação: os eventos antes de `from` passam a ser
    /// resumidos na projecção de *wire* (o log cru permanece intacto).
    pub fn set_compaction(&mut self, from: usize) {
        self.compaction_from = Some(from);
    }

    /// Índice do corte de compactação, se houver.
    #[must_use]
    pub fn compaction_from(&self) -> Option<usize> {
        self.compaction_from
    }

    /// `true` se a sessão já foi compactada.
    #[must_use]
    pub fn is_compacted(&self) -> bool {
        self.compaction_from.is_some()
    }

    /// Modo corrente da sessão (último `ModeSet`; `Build` por omissão).
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.events
            .iter()
            .rev()
            .find_map(|event| match event {
                Event::ModeSet { mode } => Some(*mode),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Define o modo da sessão (comando explícito, sempre logado — I15).
    pub fn set_mode(&mut self, mode: Mode) {
        self.push(Event::ModeSet { mode });
    }

    /// Reconstrói a projecção de *wire* a partir do log (I3).
    ///
    /// O system prompt é composto **à parte** pelo provider; aqui ficam só as
    /// mensagens do diálogo. Um passo = **uma** `Assistant` com N `tool_calls`,
    /// seguida de N `Tool`. Uma mensagem de *steering* entra **uma só vez**, mesmo
    /// que o `id` apareça em mais do que um `Steer` (idempotência — M7).
    #[must_use]
    pub fn derive_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        let from = self.compaction_from.unwrap_or(0).min(self.events.len());
        // Ids entregues **antes do corte**: o resumo já os menciona, a projecção não os repete.
        let summarized: BTreeSet<&str> = self
            .events
            .get(..from)
            .unwrap_or_default()
            .iter()
            .filter_map(|event| match event {
                Event::Steer { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        if from > 0 {
            let summary = summarize(self.events.get(..from).unwrap_or_default());
            if !summary.is_empty() {
                messages.push(Message::User { content: summary });
            }
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for event in self.events.get(from..).unwrap_or_default() {
            // Já mencionada pelo resumo, ou repetida: entra **uma só vez**.
            if let Event::Steer { id, .. } = event
                && (summarized.contains(id.as_str()) || !seen.insert(id.as_str()))
            {
                continue;
            }
            if let Some(message) = project(event) {
                messages.push(message);
            }
        }
        messages
    }

    /// Ids de *steering* já **entregues** (aparecem num `Event::Steer`).
    fn delivered_steers(&self) -> BTreeSet<&str> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Event::Steer { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Mensagens admitidas e **ainda não entregues**, por ordem de admissão e sem
    /// repetir o `id` (é o que o turno seguinte entrega — M7).
    #[must_use]
    pub fn pending_steers(&self) -> Vec<SteerMessage> {
        let delivered = self.delivered_steers();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut pending = Vec::new();
        for event in &self.events {
            if let Event::SteerQueued { id, text } = event
                && !delivered.contains(id.as_str())
                && seen.insert(id.as_str())
            {
                pending.push(SteerMessage {
                    id: id.clone(),
                    text: text.clone(),
                });
            }
        }
        pending
    }

    /// `true` se o `id` já foi admitido (entregue ou pendente) — a borda pode repetir
    /// o pedido sem duplicar a mensagem (M7).
    #[must_use]
    pub fn knows_steer(&self, id: &str) -> bool {
        self.events.iter().any(|event| match event {
            Event::SteerQueued { id: queued, .. } | Event::Steer { id: queued, .. } => queued == id,
            _ => false,
        })
    }

    /// Verifica as invariantes I1 (nenhum turno aberto) e I2 (todo `ToolCall`
    /// tem `ToolResult`).
    #[must_use]
    pub fn verify(&self) -> Vec<LogViolation> {
        let mut violations = Vec::new();
        let mut open_turn = false;
        let mut pending: Vec<ToolCallId> = Vec::new();
        for event in &self.events {
            match event {
                Event::TurnStart { .. } => open_turn = true,
                Event::TurnEnd { .. } => open_turn = false,
                Event::Assistant { tool_calls, .. } => {
                    for call in tool_calls {
                        pending.push(call.id.clone());
                    }
                }
                Event::ToolResult { tool_call_id, .. } => {
                    if let Some(position) = pending.iter().position(|id| id == tool_call_id) {
                        let _removed = pending.remove(position);
                    }
                }
                Event::GuardDecision { .. }
                | Event::ModeSet { .. }
                | Event::PluginDispatch { .. }
                | Event::SteerQueued { .. }
                | Event::Steer { .. } => {}
            }
        }
        if open_turn {
            violations.push(LogViolation::OpenTurn);
        }
        for id in pending {
            violations.push(LogViolation::MissingToolResult(id));
        }
        violations
    }
}

/// Projecta um evento na mensagem de *wire* correspondente (`None` se não emite).
fn project(event: &Event) -> Option<Message> {
    match event {
        Event::TurnStart { prompt } => Some(Message::User {
            content: prompt.clone(),
        }),
        Event::Steer { text, .. } => Some(Message::User {
            content: text.clone(),
        }),
        Event::Assistant {
            content,
            reasoning,
            tool_calls,
        } => Some(Message::Assistant {
            content: content.clone(),
            reasoning: reasoning.clone(),
            tool_calls: tool_calls.clone(),
        }),
        Event::ToolResult {
            tool_call_id,
            content,
            is_error,
            ..
        } => Some(Message::Tool {
            tool_call_id: tool_call_id.clone(),
            content: content.clone(),
            is_error: *is_error,
        }),
        Event::GuardDecision { .. }
        | Event::TurnEnd { .. }
        | Event::ModeSet { .. }
        | Event::PluginDispatch { .. }
        | Event::SteerQueued { .. } => None,
    }
}

#[cfg(test)]
mod tests;
