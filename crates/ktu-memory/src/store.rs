//! Adaptador de memória sobre `.agents/memory/notes.jsonl`.
//!
//! O JSONL é a **fonte da verdade** (append-only, dedup-on-read); o índice BM25 +
//! grafo é **derivado** e reconstruído a partir das notas em memória, usando o
//! motor puro do `knudge-core`. As escritas são duráveis (`temp → fsync → rename`).

use std::path::{Path, PathBuf};

use knudge_core::corpus::{Loaded, parse_jsonl};
use knudge_core::graph::Graph;
use knudge_core::retrieval::{Index, RecallQuery, recall};
use knudge_core::store::Note;
use knudge_core::write::{DedupDecision, DedupThresholds, propose};
use ktu_core::memory::{Hint, Memory, MemoryDraft, MemoryError, MemoryResult, MemoryWrite};
use ktu_core::memory_config::Embeddings;
use ktu_core::ports::{Clock, Embedder, Fs};

use crate::ACTIVITY_WRITE;
use crate::convert::{map_core, map_knudge, merge_into, to_knudge_draft};
use crate::embed::{self, SEMANTIC_TOP_K};

/// Memória persistente em JSONL, com o motor `knudge-core` como recuperação.
pub struct JsonlMemory<'a> {
    fs: &'a dyn Fs,
    clock: &'a dyn Clock,
    /// Raiz do projecto (para resolver âncoras no `doctor`).
    project_root: PathBuf,
    /// Directório da memória (`.agents/memory`).
    memory_dir: PathBuf,
    thresholds: DedupThresholds,
    /// Ajustes do canal semântico (H8; por omissão desligado).
    embeddings: Embeddings,
    /// Provedor de embeddings, quando há um (o índice derivado é opcional).
    embedder: Option<&'a dyn Embedder>,
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
            embeddings: Embeddings::default(),
            embedder: None,
        }
    }

    /// Liga o canal semântico ao recall (índice derivado + provedor).
    #[must_use]
    pub fn with_embeddings(mut self, embedder: &'a dyn Embedder, config: Embeddings) -> Self {
        self.embeddings = config;
        self.embedder = Some(embedder);
        self
    }

    /// Directório da memória (`.agents/memory`).
    #[must_use]
    pub fn memory_dir(&self) -> &Path {
        &self.memory_dir
    }

    /// Ajustes do canal semântico em uso.
    #[must_use]
    pub fn embeddings(&self) -> &Embeddings {
        &self.embeddings
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
    /// O parse é o do `knudge-core` ([`parse_jsonl`]) — o ktu **não** reimplementa o
    /// codec nem a tolerância; só a porta `Fs` é a do kernel.
    ///
    /// # Errors
    /// Propaga erros de I/O da porta.
    pub fn load_notes(&self) -> MemoryResult<Vec<Note>> {
        Ok(self.read_source()?.0)
    }

    /// Lê as notas e devolve também as linhas malformadas (para o `doctor`).
    ///
    /// # Errors
    /// Propaga erros de I/O da porta.
    pub fn load_report(&self) -> MemoryResult<(Vec<Note>, usize)> {
        let (notes, warnings) = self.read_source()?;
        Ok((notes, warnings.len()))
    }

    /// Lê `notes.jsonl` pelas regras do `knudge` (dedup-on-read + skip tolerante).
    fn read_source(&self) -> MemoryResult<Loaded> {
        let path = self.notes_path();
        if !self.fs.exists(&path) {
            return Ok((Vec::new(), Vec::new()));
        }
        let bytes = self.fs.read(&path).map_err(map_core)?;
        let text =
            String::from_utf8(bytes).map_err(|_| MemoryError::schema("notes.jsonl nao e UTF-8"))?;
        Ok(parse_jsonl(&text))
    }

    /// Memoriza um rascunho aplicando o protocolo idempotente de dedup.
    fn remember(&self, draft: &MemoryDraft) -> MemoryResult<MemoryWrite> {
        let now = self.clock.now().as_millis();
        let knudge_draft = to_knudge_draft(draft, ACTIVITY_WRITE);
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
        request.vector = self.semantic_channel(query);
        let output = recall(&index, &graph, &request).map_err(|error| map_knudge(&error))?;
        Ok(output
            .hits
            .into_iter()
            .map(|hit| Hint::new(hit.id, hit.statement, hit.score))
            .collect())
    }

    /// Canal semântico do recall: ids do índice derivado mais próximos da consulta.
    ///
    /// Degradação graciosa: sem provedor, sem índice ou com o provedor fora, devolve
    /// `None` e o recall fica lexical — **sem aviso**.
    fn semantic_channel(&self, query: &str) -> Option<Vec<String>> {
        let embedder = self.embedder?;
        if !self.embeddings.enabled() {
            return None;
        }
        let expected = embed::engine_meta(&self.embeddings).ok()?;
        let index = embed::load(self.fs, &self.memory_dir, &expected).ok()??;
        let ids = embed::semantic_ids(&index, embedder, query, SEMANTIC_TOP_K).ok()?;
        (!ids.is_empty()).then_some(ids)
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
        text.push_str(&note.to_jsonl().map_err(|error| map_knudge(&error))?);
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
