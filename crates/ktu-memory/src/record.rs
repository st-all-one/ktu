//! Codec de uma nota no `notes.jsonl`: `{"frontmatter": {…}, "body": "…"}`.
//!
//! Usa o **codec JSON canónico do `knudge-core`** (chaves ordenadas, `null`
//! rejeitado) e o `Frontmatter`/`Value` do mesmo motor — o contrato de bytes é o
//! do `knudge`, sem reimplementação. Uma linha JSON por nota.

use knudge_core::jsonl::json::{decode, encode};
use knudge_core::schema::{Frontmatter, Value};
use knudge_core::store::Note;
use ktu_core::memory::{MemoryError, MemoryResult};

/// Serializa uma nota numa linha JSONL.
///
/// # Errors
/// Devolve [`MemoryError::Schema`] se o `frontmatter` não for codificável.
pub(crate) fn encode_note(note: &Note) -> MemoryResult<String> {
    let envelope = Value::map([
        ("frontmatter".to_owned(), note.frontmatter.to_value()),
        ("body".to_owned(), Value::Str(note.body.clone())),
    ]);
    encode(&envelope).map_err(|error| schema_from(&error))
}

/// Reconstrói uma nota de uma linha JSONL (tolerante: valida o schema).
///
/// # Errors
/// Devolve [`MemoryError::Schema`] se a linha não for um registo de nota válido.
pub(crate) fn decode_note(line: &str) -> MemoryResult<Note> {
    let value = decode(line).map_err(|error| schema_from(&error))?;
    let Value::Map(map) = &value else {
        return Err(MemoryError::schema("linha de memoria nao e um objeto JSON"));
    };
    let frontmatter = map
        .get("frontmatter")
        .ok_or_else(|| MemoryError::schema("registo sem `frontmatter`"))?;
    let (frontmatter, _warnings) =
        Frontmatter::from_value(frontmatter).map_err(|error| schema_from(&error))?;
    let body = map
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let note = Note::new(frontmatter, body);
    note.frontmatter
        .validate()
        .map_err(|error| schema_from(&error))?;
    Ok(note)
}

/// Converte um erro do `knudge-core` em erro de schema da memória.
fn schema_from(error: &knudge_core::Error) -> MemoryError {
    MemoryError::schema(error.to_string())
}

#[cfg(test)]
mod tests {
    use knudge_core::schema::{Frontmatter, NoteType};
    use knudge_core::store::Note;
    use knudge_core::write::Draft;

    use super::{decode_note, encode_note};

    fn note(statement: &str, body: &str) -> Note {
        let draft = Draft::new(NoteType::Fact, statement).with_body(body);
        draft
            .to_note(1_700_000_000_000)
            .unwrap_or_else(|_| Note::new(Frontmatter::default(), String::new()))
    }

    #[test]
    fn round_trips_note_through_one_jsonl_line() {
        let original = note("a memoria vive em notes.jsonl", "corpo");
        let line = encode_note(&original).unwrap_or_default();
        assert!(!line.contains('\n'));
        let restored = decode_note(&line).unwrap_or_else(|_| note("x", ""));
        assert_eq!(restored.frontmatter, original.frontmatter);
        assert_eq!(restored.body, original.body);
    }

    #[test]
    fn rejects_a_non_object_line() {
        assert!(decode_note("42").is_err());
        assert!(decode_note("{}").is_err());
    }
}
