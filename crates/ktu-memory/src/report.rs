//! Vista derivada (`export --md`) e saúde da memória (`doctor --memory`).
//!
//! Nenhum destes é **fonte**: são projecções de leitura do `notes.jsonl`.

use std::path::Path;

use knudge_core::store::Note;
use ktu_core::memory::MemoryResult;

use crate::store::JsonlMemory;

/// Saúde da memória: contagens, linhas malformadas e âncoras quebradas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryHealth {
    /// Notas vivas (com dedup-on-read).
    pub notes: usize,
    /// Linhas malformadas saltadas na leitura.
    pub malformed: usize,
    /// Âncoras que apontam para caminhos inexistentes `(id, âncora)`.
    pub broken_anchors: Vec<(String, String)>,
    /// Tamanho de `notes.jsonl` em bytes.
    pub bytes: u64,
}

impl MemoryHealth {
    /// `true` se não há linhas malformadas nem âncoras quebradas.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.malformed == 0 && self.broken_anchors.is_empty()
    }
}

/// Calcula a saúde a partir das notas e do `notes.jsonl`.
///
/// # Errors
/// Propaga erros de I/O da porta.
pub fn health(memory: &JsonlMemory<'_>) -> MemoryResult<MemoryHealth> {
    let (notes, malformed) = memory.load_report()?;
    let mut broken_anchors = Vec::new();
    for note in &notes {
        let id = note.id().unwrap_or("").to_owned();
        for anchor in note.frontmatter.string_list("anchors").unwrap_or_default() {
            if is_broken(memory.project_root(), memory, anchor) {
                broken_anchors.push((id.clone(), anchor.to_owned()));
            }
        }
    }
    let bytes = file_bytes(memory, &memory.notes_path());
    Ok(MemoryHealth {
        notes: notes.len(),
        malformed,
        broken_anchors,
        bytes,
    })
}

/// Renderiza as notas como Markdown (vista humana **derivada**).
#[must_use]
pub fn export_markdown(notes: &[Note]) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("# Memória do projecto (vista derivada)\n\n");
    let _ignored = writeln!(out, "{} note(s).\n", notes.len());
    for note in notes {
        render_note(note, &mut out);
    }
    out
}

fn render_note(note: &Note, out: &mut String) {
    use std::fmt::Write as _;
    let statement = note
        .frontmatter
        .statement()
        .unwrap_or("(sem afirmação)")
        .to_owned();
    let kind = note
        .frontmatter
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or("nota")
        .to_owned();
    let id = note.id().unwrap_or("").to_owned();
    let _ignored = writeln!(out, "## {statement}\n");
    let _ignored = writeln!(out, "_tipo: {kind} · id: {id}_\n");
    if !note.body.is_empty() {
        out.push_str(&note.body);
        out.push_str("\n\n");
    }
}

/// `true` se a âncora aponta para um caminho inexistente (globs não contam).
fn is_broken(root: &Path, memory: &JsonlMemory<'_>, anchor: &str) -> bool {
    if anchor.is_empty() || anchor.contains('*') || anchor.contains('?') {
        return false;
    }
    let path = root.join(anchor.trim_start_matches("./"));
    !memory.fs().exists(&path)
}

fn file_bytes(memory: &JsonlMemory<'_>, path: &Path) -> u64 {
    if !memory.fs().exists(path) {
        return 0;
    }
    memory
        .fs()
        .read(path)
        .map_or(0, |bytes| u64::try_from(bytes.len()).unwrap_or(u64::MAX))
}
