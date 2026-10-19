//! `ktu session ls|rm|fork` — gestão das sessões persistidas (`.ktu/sessions/`).
//!
//! São operações de **ficheiro** (gestão), não de turno: a lógica de conversa vive
//! na porta `Host` (M4). `resume` é abrir o REPL (`ktu session <id>`).

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use ktu_core::checkpoint;
use ktu_core::session::{Event, SessionLog, title_of};
use ktu_core::store;
use ktu_tools::StdFs;

use crate::output::{self, Format};

/// Linha do `ls` (uma sessão persistida).
struct Row {
    id: String,
    title: Option<String>,
    turns: usize,
    mode: String,
    bytes: u64,
}

/// Directório das sessões de um *root*.
fn sessions_dir(root: &Path) -> std::path::PathBuf {
    root.join(".ktu").join("sessions")
}

/// Conta os turnos concluídos no log (`TurnEnd`).
fn count_turns(log: &SessionLog) -> usize {
    log.events()
        .iter()
        .filter(|event| matches!(event, Event::TurnEnd { .. }))
        .count()
}

/// Lê o directório de sessões, devolvendo uma linha por `.jsonl` (ordenado por id).
fn scan(root: &Path) -> Vec<Row> {
    let dir = sessions_dir(root);
    let fs = StdFs;
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let log = store::load_recovering(&fs, &dir, id).ok().flatten();
        let turns = log.as_ref().map_or(0, count_turns);
        let mode = log
            .as_ref()
            .map_or_else(|| "?".to_owned(), |log| format!("{:?}", log.mode()));
        // Título: o do checkpoint (índice) ou, em falta, derivado do log.
        let title = checkpoint::load(&fs, &dir, id)
            .ok()
            .flatten()
            .and_then(|checkpoint| checkpoint.title)
            .or_else(|| log.as_ref().and_then(title_of));
        let bytes = entry.metadata().map_or(0, |meta| meta.len());
        rows.push(Row {
            id: id.to_owned(),
            title,
            turns,
            mode,
            bytes,
        });
    }
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    rows
}

/// Identificadores das sessões persistidas (ordenados) — reutilizado pelo REPL (`/resume`).
pub(crate) fn ids(root: &Path) -> Vec<String> {
    scan(root).into_iter().map(|row| row.id).collect()
}

/// Copia o log de uma sessão para um novo id (gestão de ficheiros, sem porta).
///
/// # Errors
/// Devolve mensagem legível se a origem não existir, o destino já existir ou a cópia falhar.
pub(crate) fn copy(root: &Path, from: &str, to: &str) -> Result<(), String> {
    let dir = sessions_dir(root);
    let source = dir.join(format!("{from}.jsonl"));
    let target = dir.join(format!("{to}.jsonl"));
    if !source.is_file() {
        return Err(format!("sessao inexistente: {from}"));
    }
    if target.exists() {
        return Err(format!("destino ja existe: {to}"));
    }
    fs::copy(&source, &target)
        .map(|_copied| ())
        .map_err(|error| error.to_string())
}

/// Lista as sessões persistidas (`ktu session ls`).
pub(crate) fn list(root: &Path, format: Format) -> ExitCode {
    let rows = scan(root);
    let text = if rows.is_empty() {
        "sem sessoes".to_owned()
    } else {
        rows.iter()
            .map(|row| {
                format!(
                    "{}\t{}\t{}\t{}\t{}",
                    row.id,
                    row.turns,
                    row.mode,
                    row.bytes,
                    row.title.as_deref().unwrap_or("-")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "turns": row.turns,
                "mode": row.mode,
                "bytes": row.bytes,
                "title": row.title,
            })
        })
        .collect();
    output::emit(
        format,
        "session ls",
        &text,
        &serde_json::json!({ "sessions": data }),
    );
    ExitCode::SUCCESS
}

/// Remove uma sessão (log, checkpoint e histórico).
pub(crate) fn remove(root: &Path, id: &str, format: Format) -> ExitCode {
    let dir = sessions_dir(root);
    let mut removed = 0usize;
    for suffix in ["jsonl", "checkpoint.json", "history"] {
        if fs::remove_file(dir.join(format!("{id}.{suffix}"))).is_ok() {
            removed = removed.saturating_add(1);
        }
    }
    if removed == 0 {
        output::emit_error(
            format,
            "session rm",
            "not_found",
            &format!("sessao inexistente: {id}"),
        );
        return ExitCode::FAILURE;
    }
    output::emit(
        format,
        "session rm",
        &format!("removida: {id}"),
        &serde_json::json!({ "id": id }),
    );
    ExitCode::SUCCESS
}

/// Copia uma sessão para um novo id (`ktu session fork <from> <to>`).
pub(crate) fn fork(root: &Path, from: &str, to: &str, format: Format) -> ExitCode {
    match copy(root, from, to) {
        Ok(()) => {
            output::emit(
                format,
                "session fork",
                &format!("{from} -> {to}"),
                &serde_json::json!({ "from": from, "to": to }),
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            output::emit_error(format, "session fork", "io", &message);
            ExitCode::FAILURE
        }
    }
}
