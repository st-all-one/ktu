//! Adaptador de memória sobre `.agents/memory/notes.jsonl`.
//!
//! O JSONL é a **fonte da verdade** (append-only, dedup-on-read); o índice BM25 +
//! grafo é **derivado** e reconstruído a partir das notas em memória, usando o
//! motor puro do `knudge-core`. As escritas são duráveis (`temp → fsync → rename`).

use std::path::{Path, PathBuf};

use knudge_core::graph::Graph;
use knudge_core::retrieval::{Index, RecallQuery, recall};
use knudge_core::store::Note;
use knudge_core::write::{DedupDecision, DedupThresholds, propose};
use ktu_core::memory::{Hint, Memory, MemoryDraft, MemoryError, MemoryResult, MemoryWrite};
use ktu_core::ports::{Clock, Fs};

use crate::convert::{map_core, map_knudge, merge_into, to_knudge_draft};
use crate::record::{decode_note, encode_note};

/// Memória persistente em JSONL, com o motor `knudge-core` como recuperação.
pub struct JsonlMemory<'a> {
    fs: &'a dyn Fs,
    clock: &'a dyn Clock,
    /// Raiz do projecto (para resolver âncoras no `doctor`).
    project_root: PathBuf,
    /// Directório da memória (`.agents/memory`).
    memory_dir: PathBuf,
    thresholds: DedupThresholds,
}

impl<'a> JsonlMemory<'a> {
    /// Cria o adaptador.
    #[must_use]
    pub fn new(
        fs: &'a dyn Fs,
        clock: &'a dyn Clock,
        project_root: impl Into<PathBuf>,
        memory_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            fs,
            clock,
            project_root: project_root.into(),
            memory_dir: memory_dir.into(),
            thresholds: DedupThresholds::default(),
        }
    }

    /// Raiz do projecto.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Caminho de `notes.jsonl`.
    #[must_use]
    pub fn notes_path(&self) -> PathBuf {
        self.memory_dir.join("notes.jsonl")
    }

    /// Porta `Fs` usada (`doctor`/projecções de leitura).
    #[must_use]
    pub fn fs(&self) -> &dyn Fs {
        self.fs
    }

    /// Lê todas as notas, com **dedup-on-read** por `id` (a última linha vence) e
    /// saltando linhas malformadas (tolerância do contrato do `knudge`).
    ///
    /// # Errors
    /// Propaga erros de I/O da porta.
    pub fn load_notes(&self) -> MemoryResult<Vec<Note>> {
        let path = self.notes_path();
        if !self.fs.exists(&path) {
            return Ok(Vec::new());
        }
        let bytes = self.fs.read(&path).map_err(map_core)?;
        let text =
            String::from_utf8(bytes).map_err(|_| MemoryError::schema("notes.jsonl nao e UTF-8"))?;
        Ok(load_lines(&text))
    }

    /// Lê as notas e devolve também as linhas malformadas (para o `doctor`).
    ///
    /// # Errors
    /// Propaga erros de I/O da porta.
    pub fn load_report(&self) -> MemoryResult<(Vec<Note>, usize)> {
        let path = self.notes_path();
        if !self.fs.exists(&path) {
            return Ok((Vec::new(), 0));
        }
        let bytes = self.fs.read(&path).map_err(map_core)?;
        let text =
            String::from_utf8(bytes).map_err(|_| MemoryError::schema("notes.jsonl nao e UTF-8"))?;
        let mut notes: Vec<Note> = Vec::new();
        let mut malformed = 0usize;
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            match decode_note(line) {
                Ok(note) => upsert(&mut notes, note),
                Err(_) => malformed = malformed.saturating_add(1),
            }
        }
        notes.sort_by(|a, b| a.id().unwrap_or("").cmp(b.id().unwrap_or("")));
        Ok((notes, malformed))
    }

    /// Memoriza um rascunho aplicando o protocolo idempotente de dedup.
    fn remember(&self, draft: &MemoryDraft) -> MemoryResult<MemoryWrite> {
        let now = self.clock.now().as_millis();
        let knudge_draft = to_knudge_draft(draft);
        let note = knudge_draft
            .to_note(now)
            .map_err(|error| map_knudge(&error))?;
        let id = note.id().map_err(|error| map_knudge(&error))?.to_owned();
        let notes = self.load_notes()?;
        if let Some(existing) = notes
            .iter()
            .find(|item| item.id().ok() == Some(id.as_str()))
        {
            if existing.frontmatter.get("body_hash") == note.frontmatter.get("body_hash") {
                return Ok(MemoryWrite::Unchanged { id });
            }
            let merged = merge_into(existing, &note)?;
            self.append(&merged)?;
            return Ok(MemoryWrite::Unchanged { id });
        }
        if notes.is_empty() {
            self.append(&note)?;
            return Ok(MemoryWrite::Created { id });
        }
        let index = Index::build(&notes).map_err(|error| map_knudge(&error))?;
        let proposal =
            propose(&index, &knudge_draft, &self.thresholds).map_err(|error| map_knudge(&error))?;
        match proposal.decision {
            DedupDecision::Create => {
                self.append(&note)?;
                Ok(MemoryWrite::Created { id })
            }
            DedupDecision::Merge { candidate, score } => {
                let target = notes
                    .iter()
                    .find(|item| item.id().ok() == Some(candidate.as_str()));
                let Some(target) = target else {
                    self.append(&note)?;
                    return Ok(MemoryWrite::Created { id });
                };
                let merged = merge_into(target, &note)?;
                self.append(&merged)?;
                Ok(MemoryWrite::Merged {
                    id: candidate,
                    score,
                })
            }
            DedupDecision::Reject { candidate, score } => Ok(MemoryWrite::Rejected {
                id: candidate,
                score,
            }),
        }
    }

    /// Recupera ponteiros (id + statement + score) para `query`.
    fn search(&self, query: &str, limit: usize) -> MemoryResult<Vec<Hint>> {
        let notes = self.load_notes()?;
        if notes.is_empty() {
            return Ok(Vec::new());
        }
        let index = Index::build(&notes).map_err(|error| map_knudge(&error))?;
        let graph = Graph::from_notes_ref(&notes).map_err(|error| map_knudge(&error))?;
        let mut request = RecallQuery::new(query);
        request.limit = limit;
        let output = recall(&index, &graph, &request).map_err(|error| map_knudge(&error))?;
        Ok(output
            .hits
            .into_iter()
            .map(|hit| Hint::new(hit.id, hit.statement, hit.score))
            .collect())
    }

    /// Acrescenta uma nota ao JSONL de forma durável (`temp → fsync → rename`).
    fn append(&self, note: &Note) -> MemoryResult<()> {
        let path = self.notes_path();
        let mut text = if self.fs.exists(&path) {
            let bytes = self.fs.read(&path).map_err(map_core)?;
            String::from_utf8(bytes).map_err(|_| MemoryError::schema("notes.jsonl nao e UTF-8"))?
        } else {
            String::new()
        };
        text.push_str(&encode_note(note)?);
        text.push('\n');
        let temp = path.with_extension("jsonl.tmp");
        self.fs.write(&temp, text.as_bytes()).map_err(map_core)?;
        self.fs.sync(&temp).map_err(map_core)?;
        self.fs.rename(&temp, &path).map_err(map_core)?;
        Ok(())
    }
}

impl Memory for JsonlMemory<'_> {
    fn recall(&self, query: &str, limit: usize) -> MemoryResult<Vec<Hint>> {
        self.search(query, limit)
    }

    fn write(&self, draft: &MemoryDraft) -> MemoryResult<MemoryWrite> {
        self.remember(draft)
    }
}

/// Carrega notas de um texto JSONL com dedup-on-read (última linha vence).
fn load_lines(text: &str) -> Vec<Note> {
    let mut notes: Vec<Note> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if let Ok(note) = decode_note(line) {
            upsert(&mut notes, note);
        }
    }
    notes.sort_by(|a, b| a.id().unwrap_or("").cmp(b.id().unwrap_or("")));
    notes
}

/// Insere uma nota substituindo a de mesmo `id` (última versão vence).
fn upsert(notes: &mut Vec<Note>, note: Note) {
    if let Ok(id) = note.id()
        && let Some(slot) = notes.iter_mut().find(|item| item.id().ok() == Some(id))
    {
        *slot = note;
        return;
    }
    notes.push(note);
}
