//! `ktu memory`: vistas e saúde da memória (§2.7) — `export`, `list`, `project`,
//! `drain` e `doctor`.

use std::path::Path;
use std::process::ExitCode;

use ktu_core::ports::Fs;
use ktu_core::store;
use ktu_memory::drain;
use ktu_memory::{JsonlMemory, export_markdown, health, project};
use ktu_tools::StdFs;

use crate::clock::SystemClock;
use crate::embeddings;
use crate::output::{self, Format};

/// Constrói o adaptador de memória para `root`.
fn memory<'a>(fs: &'a StdFs, clock: &'a SystemClock, root: &Path) -> JsonlMemory<'a> {
    JsonlMemory::new(fs, clock, root, root.join(".agents").join("memory"))
}

/// `ktu memory export`: imprime a vista Markdown **derivada** em `stdout`.
pub(crate) fn export(root: &Path, format: Format) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = memory(&fs, &clock, root);
    match memory.load_notes() {
        Ok(notes) => {
            let markdown = export_markdown(&notes);
            output::emit(
                format,
                "memory export",
                markdown.trim_end(),
                &serde_json::json!({ "markdown": markdown }),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(
                format,
                "memory export",
                "io",
                &format!("nao foi possivel ler a memoria: {error}"),
            );
            ExitCode::FAILURE
        }
    }
}

/// Uma linha do `list`: identificador, tipo e afirmação.
type Row = (String, String, String);

/// Linhas do `list`: `(id, tipo, afirmação)` por nota.
fn rows(root: &Path) -> Result<Vec<Row>, String> {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = memory(&fs, &clock, root);
    let notes = memory
        .load_notes()
        .map_err(|error| format!("nao foi possivel ler a memoria: {error}"))?;
    Ok(notes
        .iter()
        .map(|note| {
            let id = note.id().unwrap_or("").to_owned();
            let kind = note
                .frontmatter
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or("nota")
                .to_owned();
            let statement = note.frontmatter.statement().unwrap_or("").to_owned();
            (id, kind, statement)
        })
        .collect())
}

/// Texto do `list` (uma linha por nota) — reutilizado pelo REPL (`/memory`).
///
/// # Errors
/// Devolve mensagem legível se a memória não carregar.
pub(crate) fn list_text(root: &Path) -> Result<String, String> {
    let rows = rows(root)?;
    if rows.is_empty() {
        return Ok("sem notas".to_owned());
    }
    Ok(rows
        .iter()
        .map(|(id, kind, statement)| format!("{id}\t{kind}\t{statement}"))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// `ktu memory list`: uma linha por nota (`id`, tipo e afirmação).
pub(crate) fn list(root: &Path, format: Format) -> ExitCode {
    match rows(root) {
        Ok(rows) => {
            let text = if rows.is_empty() {
                "sem notas".to_owned()
            } else {
                rows.iter()
                    .map(|(id, kind, statement)| format!("{id}\t{kind}\t{statement}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let data: Vec<serde_json::Value> = rows
                .iter()
                .map(|(id, kind, statement)| {
                    serde_json::json!({ "id": id, "kind": kind, "statement": statement })
                })
                .collect();
            output::emit(
                format,
                "memory list",
                &text,
                &serde_json::json!({ "notes": data }),
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            output::emit_error(format, "memory list", "io", &message);
            ExitCode::FAILURE
        }
    }
}

/// `ktu memory project`: projecta logs de sessão em **notas derivadas** (`session-log.jsonl`).
///
/// A projecção é **read-only** sobre o log (que continua a ser do agente): um turno vira uma nota
/// `meta` com `provenance.activity=derive`, pronta a ser indexada pelo `knudge` (`kd ai session-log`).
/// É idempotente — re-projectar o mesmo log não reescreve notas já gravadas.
pub(crate) fn project(
    root: &Path,
    session: Option<&str>,
    out: Option<&Path>,
    format: Format,
) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let sessions_dir = root.join(".ktu").join("sessions");
    let out_path = out.map_or_else(
        || {
            root.join(".agents")
                .join("memory")
                .join("session-log.jsonl")
        },
        Path::to_path_buf,
    );
    if let Some(parent) = out_path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        output::emit_error(
            format,
            "memory project",
            "io",
            &format!("nao foi possivel criar {}: {error}", parent.display()),
        );
        return ExitCode::FAILURE;
    }
    let ids = match session_ids(fs, &sessions_dir, session) {
        Ok(ids) => ids,
        Err(message) => {
            output::emit_error(format, "memory project", "io", &message);
            return ExitCode::FAILURE;
        }
    };
    let mut rows: Vec<(String, usize)> = Vec::new();
    let mut total = 0usize;
    for id in ids {
        let log = match store::load_recovering(&fs, &sessions_dir, &id) {
            Ok(Some(log)) => log,
            Ok(None) if session.is_some() => {
                output::emit_error(
                    format,
                    "memory project",
                    "not_found",
                    &format!("sessao {id} inexistente"),
                );
                return ExitCode::FAILURE;
            }
            Ok(None) => {
                rows.push((id, 0));
                continue;
            }
            Err(error) => {
                output::emit_error(
                    format,
                    "memory project",
                    "io",
                    &format!("sessao {id}: {error}"),
                );
                return ExitCode::FAILURE;
            }
        };
        let drafts = project::project(&id, log.events());
        match project::write(&fs, &clock, &out_path, &drafts) {
            Ok(written) => {
                total = total.saturating_add(written);
                rows.push((id, written));
            }
            Err(error) => {
                output::emit_error(
                    format,
                    "memory project",
                    "io",
                    &format!("nao foi possivel gravar a projeccao: {error}"),
                );
                return ExitCode::FAILURE;
            }
        }
    }
    let text = rows
        .iter()
        .map(|(id, notes)| format!("{id}\t{notes}"))
        .collect::<Vec<_>>()
        .join("\n");
    let sessions: Vec<serde_json::Value> = rows
        .iter()
        .map(|(id, notes)| serde_json::json!({ "id": id, "notes": notes }))
        .collect();
    let data = serde_json::json!({
        "out": out_path.display().to_string(),
        "written": total,
        "sessions": sessions,
    });
    output::emit(format, "memory project", &text, &data);
    ExitCode::SUCCESS
}

/// Ids de sessão a projectar (só `*.jsonl`, ordenados); `only` restringe a um.
fn session_ids(fs: StdFs, dir: &Path, only: Option<&str>) -> Result<Vec<String>, String> {
    if let Some(id) = only {
        return Ok(vec![id.to_owned()]);
    }
    if !fs.is_dir(dir) {
        return Ok(Vec::new());
    }
    let entries = fs
        .read_dir(dir)
        .map_err(|error| format!("nao foi possivel listar {}: {error}", dir.display()))?;
    let mut ids: Vec<String> = entries
        .iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?;
            name.strip_suffix(".jsonl").map(str::to_owned)
        })
        .collect();
    ids.sort();
    Ok(ids)
}

/// Linha do canal semântico no `doctor` (incentiva o worker sem o exigir).
fn semantic_line(fs: StdFs, root: &Path, memory: &JsonlMemory<'_>) -> String {
    let Ok(config) = embeddings::load_config(&fs, root) else {
        return "memory.embeddings.channel: memory.toml invalido".to_owned();
    };
    if !config.embeddings.enabled() {
        return "memory.embeddings.channel: lexical (`ktu memory drain service --install` liga o canal)"
            .to_owned();
    }
    let notes = memory.load_notes().unwrap_or_default();
    match drain::status(&fs, memory.memory_dir(), &config.embeddings, &notes) {
        Ok(status) if status.index.is_present() => format!(
            "memory.embeddings.channel: semantic (indexed {}, pending {}, stale {})",
            status.indexed, status.pending, status.stale
        ),
        Ok(_) => {
            "memory.embeddings.channel: semantic (`ktu memory drain --digest` constroi o indice)"
                .to_owned()
        }
        Err(error) => format!("memory.embeddings.channel: indisponivel ({error})"),
    }
}

/// `ktu memory doctor`: âncoras quebradas e saúde do índice (read-only).
pub(crate) fn doctor(root: &Path, format: Format) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = memory(&fs, &clock, root);
    match health(&memory) {
        Ok(report) => {
            let mut lines = vec![
                format!("memory.notes: {}", report.notes),
                format!("memory.malformed: {}", report.malformed),
                format!("memory.bytes: {}", report.bytes),
                format!("memory.broken_anchors: {}", report.broken_anchors.len()),
            ];
            for (id, anchor) in &report.broken_anchors {
                lines.push(format!("  {id} -> {anchor}"));
            }
            let semantic = semantic_line(fs, root, &memory);
            lines.push(semantic);
            let anchors: Vec<serde_json::Value> = report
                .broken_anchors
                .iter()
                .map(|(id, anchor)| serde_json::json!({ "id": id, "anchor": anchor }))
                .collect();
            let data = serde_json::json!({
                "notes": report.notes,
                "malformed": report.malformed,
                "bytes": report.bytes,
                "broken_anchors": anchors,
                "healthy": report.is_healthy(),
            });
            output::emit(format, "memory doctor", &lines.join("\n"), &data);
            if report.is_healthy() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            output::emit_error(
                format,
                "memory doctor",
                "io",
                &format!("nao foi possivel diagnosticar a memoria: {error}"),
            );
            ExitCode::FAILURE
        }
    }
}
