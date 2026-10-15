//! *Trash*: apagar é **mover** para `.ktu/trash/` com índice recuperável.
//!
//! A tool `trash_file` nunca destrói: renomeia o ficheiro para o *trash* e regista
//! a operação em `.ktu/trash/index.jsonl`. `list`/`restore` leem esse índice.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::ports::{Clock, Fs};
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use ktu_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::path::resolve_existing;

/// Entrada do índice do *trash*.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashEntry {
    /// Identificador (único) da entrada.
    pub id: String,
    /// Caminho original (relativo ao *root*).
    pub original: String,
}

/// Directório do *trash* dentro do *root*.
#[must_use]
pub fn trash_dir(root: &Path) -> PathBuf {
    root.join(".ktu").join("trash")
}

/// Caminho do índice do *trash*.
#[must_use]
pub fn index_path(root: &Path) -> PathBuf {
    trash_dir(root).join("index.jsonl")
}

/// Lê o índice do *trash*.
///
/// # Errors
/// Propaga erros de I/O ou JSON.
pub fn list(fs: &dyn Fs, root: &Path) -> Result<Vec<TrashEntry>> {
    let path = index_path(root);
    if !fs.exists(&path) {
        return Ok(Vec::new());
    }
    let bytes = fs.read(&path)?;
    let text = String::from_utf8(bytes).map_err(|_| Error::io(&path, "indice nao e UTF-8"))?;
    let mut entries = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: TrashEntry =
            serde_json::from_str(line).map_err(|source| Error::json("trash", source))?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Restaura a entrada `id`, devolvendo o caminho original.
///
/// # Errors
/// Erro se a entrada não existir ou o movimento falhar.
pub fn restore(fs: &dyn Fs, root: &Path, id: &str) -> Result<String> {
    let entries = list(fs, root)?;
    let Some(entry) = entries.iter().find(|entry| entry.id == id) else {
        return Err(Error::io(
            root,
            format!("entrada de trash inexistente: {id}"),
        ));
    };
    let trashed = trash_dir(root).join(&entry.id);
    let original = root.join(&entry.original);
    fs.rename(&trashed, &original)?;
    let remaining: Vec<TrashEntry> = entries
        .iter()
        .filter(|entry| entry.id != id)
        .cloned()
        .collect();
    write_index(fs, root, &remaining)?;
    Ok(entry.original.clone())
}

fn write_index(fs: &dyn Fs, root: &Path, entries: &[TrashEntry]) -> Result<()> {
    let path = index_path(root);
    let temp = trash_dir(root).join("index.jsonl.tmp");
    let mut text = String::new();
    for entry in entries {
        let line = serde_json::to_string(entry).map_err(|source| Error::json("trash", source))?;
        text.push_str(&line);
        text.push('\n');
    }
    fs.write(&temp, text.as_bytes())?;
    fs.rename(&temp, &path)
}

/// Tool `trash_file`: move um ficheiro para o *trash* (recuperável).
pub struct TrashTool<'a> {
    root: PathBuf,
    fs: &'a dyn Fs,
    clock: &'a dyn Clock,
}

impl<'a> TrashTool<'a> {
    /// Cria a tool com o *root*, a porta `Fs` e a porta `Clock`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, fs: &'a dyn Fs, clock: &'a dyn Clock) -> Self {
        Self {
            root: root.into(),
            fs,
            clock,
        }
    }

    fn run(&self, args: &Value) -> ToolOutcome {
        let Some(requested) = args.get("path").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'path' em falta");
        };
        let path = match resolve_existing(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        let millis = self.clock.now().as_millis();
        let id = format!("{millis}-{requested}").replace('/', "_");
        let destination = trash_dir(&self.root).join(&id);
        if let Err(error) = self.fs.rename(&path, &destination) {
            return ToolOutcome::error(format!("nao foi possivel mover {requested}: {error}"));
        }
        let mut entries = list(self.fs, &self.root).unwrap_or_default();
        entries.push(TrashEntry {
            id: id.clone(),
            original: requested.to_owned(),
        });
        match write_index(self.fs, &self.root, &entries) {
            Ok(()) => ToolOutcome::ok(format!("movido para trash: {id}")),
            Err(error) => ToolOutcome::error(format!("indice do trash falhou: {error}")),
        }
    }
}

#[async_trait]
impl ToolExecutor for TrashTool<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "trash_file".to_owned(),
            description: "Move um ficheiro para o trash (recuperavel) em vez de apagar.".to_owned(),
            parameters:
                r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#
                    .to_owned(),
        }]
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        let args: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
        self.run(&args)
    }
}
