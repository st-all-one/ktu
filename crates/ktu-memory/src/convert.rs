//! Conversão e fusão entre os tipos do ktu e o motor `knudge-core`.
//!
//! Fica separado do adaptador (`store.rs`) para manter cada ficheiro pequeno e o
//! adaptador focado no protocolo (recall/escrita/dedup).

use knudge_core::schema::{Frontmatter, NoteType, Value};
use knudge_core::store::Note;
use knudge_core::write::Draft;
use ktu_core::memory::{MemoryDraft, MemoryError, MemoryKind, MemoryResult};

/// Funde `incoming` em `target` (tags, âncoras e corpo), com revisão+1.
pub(crate) fn merge_into(target: &Note, incoming: &Note) -> MemoryResult<Note> {
    let mut merged = target.clone();
    let mut tags = tag_list(&merged);
    for tag in tag_list(incoming) {
        if !tags.iter().any(|existing| existing == &tag) {
            tags.push(tag);
        }
    }
    set_list(&mut merged.frontmatter, "tags", &tags)?;
    let mut anchors = anchor_list(&merged);
    for anchor in anchor_list(incoming) {
        if !anchors.iter().any(|existing| existing == &anchor) {
            anchors.push(anchor);
        }
    }
    set_list(&mut merged.frontmatter, "anchors", &anchors)?;
    if !incoming.body.is_empty() && !merged.body.contains(incoming.body.as_str()) {
        if !merged.body.is_empty() {
            merged.body.push_str("\n\n");
        }
        merged.body.push_str(&incoming.body);
    }
    let revision = merged.revision().saturating_add(1);
    merged
        .set_revision(revision)
        .map_err(|error| map_knudge(&error))?;
    merged
        .refresh_body_hash()
        .map_err(|error| map_knudge(&error))?;
    Ok(merged)
}

fn tag_list(note: &Note) -> Vec<String> {
    note.frontmatter
        .string_list("tags")
        .unwrap_or_default()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn anchor_list(note: &Note) -> Vec<String> {
    note.frontmatter
        .string_list("anchors")
        .unwrap_or_default()
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn set_list(frontmatter: &mut Frontmatter, key: &str, items: &[String]) -> MemoryResult<()> {
    if items.is_empty() {
        let _removed = frontmatter.remove(key);
        return Ok(());
    }
    let value = Value::List(items.iter().map(|item| Value::Str(item.clone())).collect());
    frontmatter
        .set(key, value)
        .map_err(|error| map_knudge(&error))
}

/// Traduz a espécie do ktu para o tipo fechado do `knudge`.
pub(crate) fn to_knudge_draft(draft: &MemoryDraft) -> Draft {
    let note_type = match draft.kind {
        MemoryKind::Fact => NoteType::Fact,
        MemoryKind::Decision => NoteType::Decision,
        MemoryKind::Error => NoteType::Error,
        MemoryKind::Risk => NoteType::Risk,
        MemoryKind::Question => NoteType::Question,
        MemoryKind::Def => NoteType::Def,
        MemoryKind::Snippet => NoteType::Snippet,
        MemoryKind::Link => NoteType::Link,
        MemoryKind::Meta => NoteType::Meta,
    };
    let mut knudge_draft = Draft::new(note_type, draft.statement.clone());
    knudge_draft.body.clone_from(&draft.body);
    knudge_draft.tags.clone_from(&draft.tags);
    knudge_draft.anchors.clone_from(&draft.anchors);
    knudge_draft
}

pub(crate) fn map_knudge(error: &knudge_core::Error) -> MemoryError {
    MemoryError::schema(error.to_string())
}

pub(crate) fn map_core(error: ktu_core::Error) -> MemoryError {
    match error {
        ktu_core::Error::Io { path, source } => MemoryError::Io { path, source },
        other => MemoryError::schema(other.to_string()),
    }
}
