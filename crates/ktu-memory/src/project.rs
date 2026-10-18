//! Projecção do log de sessão em **notas derivadas** (H7).
//!
//! O log `.ktu/sessions/<id>.jsonl` é a fonte da verdade da sessão (I3) e **continua a ser do
//! agente**. A projecção é **read-only** sobre ele: um turno vira **uma** nota derivada
//! (`provenance.activity = derive`), com a afirmação a ser o pedido do utilizador e o corpo a
//! resposta final + as ferramentas usadas. O `knudge` só **indexa** o resultado — nunca aprende o
//! schema `Event` (o contrato é o **documento**).
//!
//! Deterministmo: a ordem dos turnos é a do log; ferramentas e âncoras são conjuntos ordenados;
//! o `statement` é prefixado com a sessão (turnos iguais em sessões diferentes não colidem).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use knudge_core::corpus::parse_jsonl;
use knudge_core::schema::Value;
use ktu_core::domain::ToolCall;
use ktu_core::memory::{MemoryDraft, MemoryError, MemoryKind, MemoryResult};
use ktu_core::ports::{Clock, Fs};
use ktu_core::session::Event;

use crate::ACTIVITY_DERIVE;
use crate::convert::{map_core, map_knudge, to_knudge_draft};

/// Etiqueta de todas as notas derivadas.
pub const TAG: &str = "session-log";
/// Máximo de âncoras por turno.
const MAX_ANCHORS: usize = 8;
/// Máximo de escalares no `statement` (contrato do `knudge`, D08).
const MAX_STATEMENT: usize = 120;

/// Projecta os eventos de uma sessão em notas derivadas — **uma por turno**.
///
/// Um turno é `TurnStart`…`TurnEnd`; um turno aberto (sem `TurnEnd`) não é projectado.
#[must_use]
pub fn project(session: &str, events: &[Event]) -> Vec<MemoryDraft> {
    let mut drafts = Vec::new();
    let mut open: Option<Turn> = None;
    for event in events {
        match event {
            Event::TurnStart { prompt } => open = Some(Turn::new(prompt)),
            Event::Assistant {
                content,
                tool_calls,
                ..
            } => {
                if let Some(turn) = open.as_mut() {
                    turn.observe(content, tool_calls);
                }
            }
            Event::TurnEnd { .. } => {
                if let Some(turn) = open.take() {
                    drafts.push(turn.into_draft(session));
                }
            }
            _ => {}
        }
    }
    drafts
}

/// Escreve as notas derivadas num JSONL, de forma durável (`temp → sync → rename`).
///
/// É **idempotente**: uma nota cujo `id` já exista com o mesmo `body_hash` não é reescrita.
/// Devolve quantas notas foram efectivamente acrescentadas.
///
/// # Errors
/// Propaga erros de I/O da porta e de schema do `knudge`.
pub fn write(
    fs: &dyn Fs,
    clock: &dyn Clock,
    path: &Path,
    drafts: &[MemoryDraft],
) -> MemoryResult<usize> {
    if drafts.is_empty() {
        return Ok(0);
    }
    let mut text = read_text(fs, path)?;
    let known = known_bodies(&text);
    let now = clock.now().as_millis();
    let mut written = 0usize;
    for draft in drafts {
        let note = to_knudge_draft(draft, ACTIVITY_DERIVE)
            .to_note(now)
            .map_err(|error| map_knudge(&error))?;
        let id = note.id().map_err(|error| map_knudge(&error))?.to_owned();
        let hash = note
            .frontmatter
            .get("body_hash")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        if known.get(&id) == Some(&hash) {
            continue;
        }
        text.push_str(&note.to_jsonl().map_err(|error| map_knudge(&error))?);
        text.push('\n');
        written = written.saturating_add(1);
    }
    if written == 0 {
        return Ok(0);
    }
    let temp = path.with_extension("jsonl.tmp");
    fs.write(&temp, text.as_bytes()).map_err(map_core)?;
    fs.sync(&temp).map_err(map_core)?;
    fs.rename(&temp, path).map_err(map_core)?;
    Ok(written)
}

/// Turno em construção (resposta, ferramentas e âncoras).
struct Turn {
    prompt: String,
    answer: String,
    tools: BTreeSet<String>,
    anchors: BTreeSet<String>,
}

impl Turn {
    fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_owned(),
            answer: String::new(),
            tools: BTreeSet::new(),
            anchors: BTreeSet::new(),
        }
    }

    /// Acumula um passo: a resposta final (último passo sem ferramentas) e as ferramentas usadas.
    fn observe(&mut self, content: &str, tool_calls: &[ToolCall]) {
        if tool_calls.is_empty() && !content.trim().is_empty() {
            content.trim().clone_into(&mut self.answer);
        }
        for call in tool_calls {
            let _ignored = self.tools.insert(call.name.clone());
            if let Some(path) = path_argument(&call.arguments) {
                let _ignored = self.anchors.insert(path);
            }
        }
    }

    fn into_draft(self, session: &str) -> MemoryDraft {
        let statement = truncate(
            &format!("[{session}] {}", collapse(&self.prompt)),
            MAX_STATEMENT,
        );
        let mut draft = MemoryDraft::new(MemoryKind::Meta, statement);
        draft.body = body_of(&self.answer, &self.tools);
        draft.tags = vec![TAG.to_owned()];
        draft.anchors = self.anchors.into_iter().take(MAX_ANCHORS).collect();
        draft
    }
}

/// Corpo da nota derivada: a resposta final e as ferramentas usadas.
fn body_of(answer: &str, tools: &BTreeSet<String>) -> String {
    let mut body = answer.to_owned();
    if !tools.is_empty() {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str("Ferramentas: ");
        body.push_str(&tools.iter().cloned().collect::<Vec<_>>().join(", "));
    }
    body
}

/// Colapsa whitespace: a afirmação é uma linha só.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Trunca a `text` a `max` escalares Unicode (contrato do `knudge`, D08).
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Caminho declarado nos argumentos JSON de uma tool (`path`/`file_path`), se houver.
fn path_argument(arguments: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(arguments).ok()?;
    let object = value.as_object()?;
    for key in ["path", "file_path"] {
        if let Some(text) = object.get(key).and_then(serde_json::Value::as_str)
            && !text.trim().is_empty()
        {
            return Some(text.trim().to_owned());
        }
    }
    None
}

/// Texto do ficheiro (vazio se ausente).
fn read_text(fs: &dyn Fs, path: &Path) -> MemoryResult<String> {
    if !fs.exists(path) {
        return Ok(String::new());
    }
    let bytes = fs.read(path).map_err(map_core)?;
    String::from_utf8(bytes).map_err(|_| MemoryError::schema("session-log nao e UTF-8"))
}

/// `id` → `body_hash` das notas já gravadas (dedup da projecção).
fn known_bodies(text: &str) -> BTreeMap<String, String> {
    let (notes, _warnings) = parse_jsonl(text);
    notes
        .into_iter()
        .filter_map(|note| {
            let id = note.id().ok()?.to_owned();
            let hash = note.frontmatter.get("body_hash")?.as_str()?.to_owned();
            Some((id, hash))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{TAG, project, write};
    use ktu_core::domain::{StopReason, ToolCall, ToolCallId};
    use ktu_core::memory::MemoryKind;
    use ktu_core::ports::{FixedClock, Fs, MemFs, Timestamp};
    use ktu_core::session::Event;
    use std::path::Path;

    fn tool_call(name: &str, arguments: &str) -> ToolCall {
        ToolCall {
            id: ToolCallId::new("c1"),
            name: name.to_owned(),
            arguments: arguments.to_owned(),
        }
    }

    fn turn(prompt: &str, answer: &str, calls: Vec<ToolCall>) -> Vec<Event> {
        vec![
            Event::TurnStart {
                prompt: prompt.to_owned(),
            },
            Event::Assistant {
                content: String::new(),
                reasoning: String::new(),
                tool_calls: calls,
            },
            Event::Assistant {
                content: answer.to_owned(),
                reasoning: "pensei".to_owned(),
                tool_calls: Vec::new(),
            },
            Event::TurnEnd {
                reason: StopReason::EndTurn,
            },
        ]
    }

    #[test]
    fn one_turn_becomes_one_derived_note() {
        let events = turn(
            "onde vive a memoria?",
            "num jsonl",
            vec![tool_call("read_file", "{\"path\":\"src/store.rs\"}")],
        );
        let drafts = project("s1", &events);
        assert_eq!(drafts.len(), 1);
        let Some(draft) = drafts.first() else {
            return;
        };
        assert_eq!(draft.kind, MemoryKind::Meta);
        assert_eq!(draft.statement, "[s1] onde vive a memoria?");
        assert!(draft.body.contains("num jsonl"));
        assert!(draft.body.contains("Ferramentas: read_file"));
        assert_eq!(draft.anchors, vec!["src/store.rs".to_owned()]);
        assert_eq!(draft.tags, vec![TAG.to_owned()]);
    }

    #[test]
    fn open_turn_is_not_projected_and_turns_are_ordered() {
        let mut events = turn("primeiro", "a", Vec::new());
        events.extend(turn("segundo", "b", Vec::new()));
        events.push(Event::TurnStart {
            prompt: "incompleto".to_owned(),
        });
        let drafts = project("s1", &events);
        assert_eq!(drafts.len(), 2);
        assert_eq!(
            drafts.first().map(|draft| draft.statement.as_str()),
            Some("[s1] primeiro")
        );
        assert_eq!(
            drafts.get(1).map(|draft| draft.statement.as_str()),
            Some("[s1] segundo")
        );
    }

    #[test]
    fn statement_is_collapsed_and_truncated_to_120_scalars() {
        let long = "a".repeat(400);
        let events = turn(&format!("linha\n  com   espacos {long}"), "", Vec::new());
        let drafts = project("s", &events);
        let Some(draft) = drafts.first() else {
            return;
        };
        assert!(draft.statement.chars().count() <= 120);
        assert!(draft.statement.contains("linha com espacos"));
    }

    #[test]
    fn projection_is_idempotent_on_disk() {
        let fs = MemFs::new();
        let clock = FixedClock::new(Timestamp::from_millis(1_700_000_000_000));
        let path = Path::new("/.agents/memory/session-log.jsonl");
        let drafts = project("s1", &turn("ola", "adeus", Vec::new()));
        assert_eq!(write(&fs, &clock, path, &drafts).ok(), Some(1));
        assert_eq!(write(&fs, &clock, path, &drafts).ok(), Some(0));
        let text = fs
            .read(path)
            .map(|bytes| String::from_utf8(bytes).unwrap_or_default())
            .unwrap_or_default();
        assert_eq!(text.lines().count(), 1);
    }
}
