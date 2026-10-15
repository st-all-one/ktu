//! Spike de validação (`task_00uz2my4`): o motor puro do `knudge-core` indexa e
//! recupera notas materializadas **em memória** — sem `Store`/`Fs`, o que permite
//! manter o JSONL como container (fonte da verdade).
//!
//! Prova três coisas: (1) uma nota vira `Note` canónica (id endereçado por
//! conteúdo); (2) `Index::build` + `recall` devolvem *hints* (id + statement +
//! score); (3) o codec JSON do `knudge-core` faz ida-e-volta do frontmatter.

use knudge_core::graph::Graph;
use knudge_core::jsonl::json::{decode, encode};
use knudge_core::retrieval::{Index, RecallQuery, recall};
use knudge_core::schema::{Frontmatter, NoteType, Value};
use knudge_core::store::Note;
use knudge_core::write::Draft;

fn note(statement: &str, body: &str) -> Note {
    Draft::new(NoteType::Fact, statement)
        .with_body(body)
        .to_note(1_700_000_000_000)
        .unwrap_or_else(|_| Note::new(Frontmatter::new(), String::new()))
}

#[test]
fn knudge_engine_indexes_in_memory_notes() {
    let notes = vec![
        note(
            "O guard estrutural confina paths ao root resolvido",
            "Usa canonicalize e rejeita escapes por symlink.",
        ),
        note(
            "A memória vive em notes.jsonl append-only",
            "O índice é derivado e descartável.",
        ),
        note(
            "O provider fala SSE com o llama-server",
            "Streaming de deltas de tokens.",
        ),
    ];
    let index = Index::build(&notes);
    assert!(index.is_ok(), "Index::build falhou");
    let Ok(index) = index else { return };
    let graph = Graph::from_notes_ref(&notes);
    assert!(graph.is_ok(), "Graph::from_notes_ref falhou");
    let Ok(graph) = graph else { return };

    let output = recall(&index, &graph, &RecallQuery::new("guard confina path root"));
    assert!(output.is_ok(), "recall falhou");
    let Ok(output) = output else { return };
    assert!(!output.hits.is_empty(), "recall não devolveu hits");
    let Some(top) = output.hits.first() else {
        return;
    };
    assert!(top.statement.contains("guard"));
    assert!(top.score > 0.0);
}

#[test]
fn canonical_json_codec_round_trips_frontmatter() {
    let original = note("A memória vive em notes.jsonl", "corpo");
    let encoded = encode(&original.frontmatter.to_value());
    assert!(encoded.is_ok(), "encode do frontmatter falhou");
    let Ok(encoded) = encoded else { return };
    let decoded = decode(&encoded);
    assert!(decoded.is_ok(), "decode do frontmatter falhou");
    let Ok(decoded) = decoded else { return };
    let restored = Frontmatter::from_value(&decoded);
    assert!(restored.is_ok(), "from_value falhou");
    let Ok((restored, _warnings)) = restored else {
        return;
    };
    assert_eq!(restored.to_value(), original.frontmatter.to_value());

    // O corpo viaja como string JSON ao lado do frontmatter.
    let frontmatter_value = original.frontmatter.to_value();
    let envelope = Value::map([
        ("frontmatter".to_string(), frontmatter_value),
        ("body".to_string(), Value::Str(original.body)),
    ]);
    let line = encode(&envelope);
    assert!(line.is_ok(), "encode do envelope falhou");
    let Ok(line) = line else { return };
    let back = decode(&line);
    assert!(back.is_ok(), "decode do envelope falhou");
    let Ok(back) = back else { return };
    assert!(matches!(back, Value::Map(_)));
}
