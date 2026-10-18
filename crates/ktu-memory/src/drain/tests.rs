//! Testes do modulo (extraidos para manter o ficheiro de producao abaixo do teto).

use knudge_core::schema::NoteType;
use knudge_core::store::Note;
use knudge_core::write::Draft;
use ktu_core::memory_config::{EmbeddingProvider, Embeddings};
use ktu_core::ports::{FakeEmbedder, MemFs, UnavailableEmbedder};

use super::{Channel, IndexState, Rebuild, digest, status};

/// Nota de teste com `body_hash` (o contrato do motor).
fn note(statement: &str, body: &str) -> Result<Note, String> {
    let mut draft = Draft::new(NoteType::Fact, statement);
    draft.body = body.to_owned();
    draft.to_note(1_700_000_000_000).map_err(|e| e.to_string())
}

fn config(provider: EmbeddingProvider) -> Embeddings {
    Embeddings {
        provider,
        dimensions: 8,
        ..Embeddings::default()
    }
}

fn dir() -> &'static std::path::Path {
    std::path::Path::new("/mem")
}

#[test]
fn provider_off_reports_lexical_and_no_queue() -> Result<(), String> {
    let fs = MemFs::new();
    let notes = vec![note("a", "")?];
    let report =
        status(&fs, dir(), &config(EmbeddingProvider::None), &notes).map_err(|e| e.to_string())?;
    assert_eq!(report.channel, Channel::Lexical);
    assert!(!report.channel.is_semantic());
    assert_eq!(report.index, IndexState::Absent);
    assert_eq!(report.pending, 0, "sem provedor nao ha fila semantica");
    let outcome = digest(
        &fs,
        dir(),
        &config(EmbeddingProvider::None),
        &notes,
        &FakeEmbedder::with_dimensions(8),
        Rebuild::Keep,
    )
    .map_err(|e| e.to_string())?;
    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.failure, None);
    Ok(())
}

#[test]
fn digest_indexes_everything_and_is_idempotent() -> Result<(), String> {
    let fs = MemFs::new();
    let config = config(EmbeddingProvider::Http);
    let embedder = FakeEmbedder::with_dimensions(8);
    let notes = vec![note("a", "corpo a")?, note("b", "")?];

    let before = status(&fs, dir(), &config, &notes).map_err(|e| e.to_string())?;
    assert_eq!(before.pending, 2);
    assert!(!before.index.is_present());

    let first =
        digest(&fs, dir(), &config, &notes, &embedder, Rebuild::Keep).map_err(|e| e.to_string())?;
    assert_eq!(first.indexed, 2);
    assert_eq!(first.pending, 0);
    assert_eq!(first.failure, None);

    let after = status(&fs, dir(), &config, &notes).map_err(|e| e.to_string())?;
    assert!(after.index.is_present());
    assert_eq!(after.channel, Channel::Semantic);
    assert_eq!(after.indexed, 2);
    assert_eq!(after.pending, 0);

    let second =
        digest(&fs, dir(), &config, &notes, &embedder, Rebuild::Keep).map_err(|e| e.to_string())?;
    assert_eq!(second.indexed, 0, "nada mudou: o dreno e idempotente");
    assert_eq!(second.batches, 0);
    Ok(())
}

#[test]
fn changed_body_is_stale_and_force_reindexes() -> Result<(), String> {
    let fs = MemFs::new();
    let config = config(EmbeddingProvider::Http);
    let embedder = FakeEmbedder::with_dimensions(8);
    let original = note("a", "corpo a")?;
    let notes = vec![original.clone()];
    let _first =
        digest(&fs, dir(), &config, &notes, &embedder, Rebuild::Keep).map_err(|e| e.to_string())?;

    let mut changed = original;
    changed.body.push_str(" + novo");
    changed.refresh_body_hash().map_err(|e| e.to_string())?;
    let changed = vec![changed];
    let report = status(&fs, dir(), &config, &changed).map_err(|e| e.to_string())?;
    assert_eq!(report.stale, 1);
    assert_eq!(report.pending, 1);

    let forced = digest(&fs, dir(), &config, &changed, &embedder, Rebuild::Wipe)
        .map_err(|e| e.to_string())?;
    assert_eq!(forced.indexed, 1);
    assert_eq!(
        forced.stale, 0,
        "--force re-embeda tudo: nao ha vetor antigo a comparar"
    );
    assert!(Rebuild::Wipe.wipes());
    assert!(!Rebuild::Keep.wipes());
    Ok(())
}

#[test]
fn batches_are_bounded_by_the_configuration() -> Result<(), String> {
    let fs = MemFs::new();
    let mut config = config(EmbeddingProvider::Http);
    config.batch = 1;
    let embedder = FakeEmbedder::with_dimensions(8);
    let notes = vec![note("a", "")?, note("b", "")?, note("c", "")?];
    let outcome =
        digest(&fs, dir(), &config, &notes, &embedder, Rebuild::Keep).map_err(|e| e.to_string())?;
    assert_eq!(outcome.indexed, 3);
    assert_eq!(outcome.batches, 3, "um lote por nota");
    Ok(())
}

#[test]
fn provider_failure_keeps_the_queue_pending() -> Result<(), String> {
    let fs = MemFs::new();
    let config = config(EmbeddingProvider::Http);
    let notes = vec![note("a", "")?];
    let outcome = digest(
        &fs,
        dir(),
        &config,
        &notes,
        &UnavailableEmbedder::new(),
        Rebuild::Keep,
    )
    .map_err(|e| e.to_string())?;
    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.pending, 1, "a nota continua pendente");
    assert!(outcome.failure.is_some(), "a falha e reportada");
    let report = status(&fs, dir(), &config, &notes).map_err(|e| e.to_string())?;
    assert_eq!(report.pending, 1);
    assert_eq!(report.index, IndexState::Absent);
    Ok(())
}
