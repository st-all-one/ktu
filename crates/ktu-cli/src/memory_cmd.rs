//! `ktu memory` e `ktu doctor --memory`: vistas e saúde da memória (§2.7).

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_memory::{JsonlMemory, export_markdown, health};
use ktu_tools::StdFs;

use crate::clock::SystemClock;

/// Constrói o adaptador de memória para `root`.
fn memory<'a>(fs: &'a StdFs, clock: &'a SystemClock, root: &Path) -> JsonlMemory<'a> {
    JsonlMemory::new(fs, clock, root, root.join(".agents").join("memory"))
}

/// `ktu memory export`: imprime a vista Markdown **derivada** em `stdout`.
pub(crate) fn export(root: &Path) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = memory(&fs, &clock, root);
    match memory.load_notes() {
        Ok(notes) => {
            let mut stdout = io::stdout().lock();
            let _ignored = write!(stdout, "{}", export_markdown(&notes));
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&format!("nao foi possivel ler a memoria: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// `ktu memory list`: uma linha por nota (`id`, tipo e afirmação).
pub(crate) fn list(root: &Path) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = memory(&fs, &clock, root);
    match memory.load_notes() {
        Ok(notes) => {
            let mut stdout = io::stdout().lock();
            for note in &notes {
                let id = note.id().unwrap_or("");
                let kind = note
                    .frontmatter
                    .get("type")
                    .and_then(|value| value.as_str())
                    .unwrap_or("nota");
                let statement = note.frontmatter.statement().unwrap_or("");
                let _ignored = writeln!(stdout, "{id}\t{kind}\t{statement}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&format!("nao foi possivel ler a memoria: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// `ktu doctor --memory`: âncoras quebradas e saúde do índice.
pub(crate) fn doctor(root: &Path) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = memory(&fs, &clock, root);
    match health(&memory) {
        Ok(report) => {
            let mut stdout = io::stdout().lock();
            let _ignored = writeln!(stdout, "memory.notes: {}", report.notes);
            let _ignored = writeln!(stdout, "memory.malformed: {}", report.malformed);
            let _ignored = writeln!(stdout, "memory.bytes: {}", report.bytes);
            let _ignored = writeln!(
                stdout,
                "memory.broken_anchors: {}",
                report.broken_anchors.len()
            );
            for (id, anchor) in &report.broken_anchors {
                let _ignored = writeln!(stdout, "  {id} -> {anchor}");
            }
            if report.is_healthy() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            report_error(&format!("nao foi possivel diagnosticar a memoria: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// Escreve em `stderr` uma mensagem de erro da borda.
fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
