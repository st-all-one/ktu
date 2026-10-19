//! Ferramentas de *filesystem* confinadas ao *root* da sessão.
//!
//! `read_file`, `write_file`, `edit_file`, `list_dir`, `glob`, `grep`. Os erros
//! são accionáveis e o conteúdo lido entra no contexto dentro do envelope
//! *taint* (I13).

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::ports::Fs;
use ktu_core::taint::wrap_untrusted;
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use serde_json::Value;
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::mutation::FileMutationQueue;
use crate::path::{resolve_existing, resolve_for_write};
use crate::search;
use crate::text::{Document, Edit, apply_edits, to_lf};
use crate::truncate;

/// Limite por omissão do conteúdo lido (bytes).
const DEFAULT_MAX_BYTES: usize = 65_536;

/// Conjunto de ferramentas de *filesystem* da sessão.
pub struct FsTools<'a> {
    root: PathBuf,
    fs: &'a dyn Fs,
    max_bytes: usize,
    /// Fila de mutação por caminho (F4): duas escritas ao mesmo ficheiro não se
    /// sobrepõem; a caminhos distintos correm em paralelo.
    ///
    /// É **injectada**: o mesmo processo partilha uma fila entre sessões (o `serve`
    /// corre um actor por sessão, o `--batch --parallel` uma thread por worker) —
    /// uma fila por turno nao serializaria nada entre sessoes.
    queue: &'a FileMutationQueue,
}

impl<'a> FsTools<'a> {
    /// Cria o conjunto com o *root*, a porta de ficheiros e a fila de mutação.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, fs: &'a dyn Fs, queue: &'a FileMutationQueue) -> Self {
        Self {
            root: root.into(),
            fs,
            max_bytes: DEFAULT_MAX_BYTES,
            queue,
        }
    }

    fn read(&self, args: &Value) -> ToolOutcome {
        let Some(requested) = args.get("path").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'path' em falta");
        };
        let offset = args.get("offset").and_then(Value::as_u64).unwrap_or(1);
        let path = match resolve_existing(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) if message.starts_with("caminho inexistente") => {
                return ToolOutcome::error(format!("read.missing: {requested}"));
            }
            Err(message) => return ToolOutcome::error(message),
        };
        let Ok(bytes) = self.fs.read(&path) else {
            return ToolOutcome::error(format!("read.missing: {requested}"));
        };
        let document = Document::parse(&bytes);
        let text = document.body();
        let lines: Vec<&str> = text.lines().collect();
        let start = usize::try_from(offset.saturating_sub(1)).unwrap_or(0);
        let Some(from) = lines.get(start..) else {
            return ToolOutcome::error(format!(
                "read.missing: offset {offset} alem do fim ({} linhas)",
                lines.len()
            ));
        };
        let window = from.join("\n");
        let capped = truncate::truncate(&window, truncate::MAX_LINES, self.max_bytes);
        let mut body = capped.text;
        if capped.is_cut {
            let next = start.saturating_add(body.lines().count());
            let _ignored = write!(
                body,
                "\n[truncado: continua com read_file path={requested} offset={}]",
                next.saturating_add(1)
            );
        }
        ToolOutcome::ok(wrap_untrusted("read_file", &body))
    }

    async fn write(&self, args: &Value) -> ToolOutcome {
        let requested = args.get("path").and_then(Value::as_str);
        let content = args.get("content").and_then(Value::as_str);
        let (Some(requested), Some(content)) = (requested, content) else {
            return ToolOutcome::error("argumentos 'path' e 'content' sao obrigatorios");
        };
        let path = match resolve_for_write(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        let _turn = self.queue.acquire(&path).await;
        match self.fs.write(&path, content.as_bytes()) {
            Ok(()) => ToolOutcome::ok(format!("escrito {requested} ({} bytes)", content.len())),
            Err(error) => {
                ToolOutcome::error(format!("nao foi possivel escrever {requested}: {error}"))
            }
        }
    }

    async fn edit(&self, args: &Value) -> ToolOutcome {
        let Some(requested) = args.get("path").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'path' em falta");
        };
        let edits = match parse_edits(args) {
            Ok(edits) => edits,
            Err(message) => return ToolOutcome::error(message),
        };
        let path = match resolve_existing(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        // A vez é pedida **antes** da leitura: ler-fora/gravar-dentro perdia a
        // alteração de outra edição ao mesmo ficheiro.
        let _turn = self.queue.acquire(&path).await;
        let Ok(bytes) = self.fs.read(&path) else {
            return ToolOutcome::error(format!("read.missing: {requested}"));
        };
        let document = Document::parse(&bytes);
        let updated = match apply_edits(document.body(), &edits) {
            Ok(updated) => updated,
            Err(message) => return ToolOutcome::error(format!("{message} em {requested}")),
        };
        let rendered = document.render(&updated);
        match self.fs.write(&path, &rendered) {
            Ok(()) => ToolOutcome::ok(format!("editado {requested} ({} bloco(s))", edits.len())),
            Err(error) => {
                ToolOutcome::error(format!("nao foi possivel escrever {requested}: {error}"))
            }
        }
    }

    fn list(&self, args: &Value) -> ToolOutcome {
        let requested = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let path = match resolve_existing(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        match self.fs.read_dir(&path) {
            Ok(entries) => {
                let listing = entries
                    .iter()
                    .map(|entry| {
                        let name = entry
                            .strip_prefix(&path)
                            .unwrap_or(entry)
                            .to_string_lossy()
                            .to_string();
                        if self.fs.is_dir(entry) {
                            format!("{name}/")
                        } else {
                            name
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                ToolOutcome::ok(wrap_untrusted("list_dir", &listing))
            }
            Err(error) => {
                ToolOutcome::error(format!("nao foi possivel listar {requested}: {error}"))
            }
        }
    }

    fn glob(&self, args: &Value) -> ToolOutcome {
        let Some(pattern) = args.get("pattern").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'pattern' em falta");
        };
        let matches = search::glob(self.fs, &self.root, pattern);
        if matches.is_empty() {
            return ToolOutcome::ok(format!("sem resultados para {pattern}"));
        }
        ToolOutcome::ok(wrap_untrusted("glob", &matches.join("\n")))
    }

    fn grep(&self, args: &Value) -> ToolOutcome {
        let Some(pattern) = args.get("pattern").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'pattern' em falta");
        };
        let matches = search::grep(self.fs, &self.root, pattern);
        if matches.is_empty() {
            return ToolOutcome::ok(format!("sem resultados para {pattern}"));
        }
        ToolOutcome::ok(wrap_untrusted("grep", &matches.join("\n")))
    }
}

/// Especificação de uma tool de *filesystem*.
fn spec(name: &str, description: &str, parameters: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_owned(),
        description: description.to_owned(),
        parameters: parameters.to_owned(),
    }
}

/// Lê as edições do pedido: `edits` (array) ou o par `old`/`new` (uma edição).
fn parse_edits(args: &Value) -> Result<Vec<Edit>, String> {
    if let Some(items) = args.get("edits").and_then(Value::as_array) {
        if items.is_empty() {
            return Err("lista 'edits' vazia".to_owned());
        }
        return items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let old = item.get("old").and_then(Value::as_str);
                let new = item.get("new").and_then(Value::as_str);
                match (old, new) {
                    (Some(old), Some(new)) => Ok(Edit {
                        old: to_lf(old),
                        new: to_lf(new),
                    }),
                    _ => Err(format!(
                        "bloco {}: 'old' e 'new' sao obrigatorios",
                        index.saturating_add(1)
                    )),
                }
            })
            .collect();
    }
    let old = args.get("old").and_then(Value::as_str);
    let new = args.get("new").and_then(Value::as_str);
    match (old, new) {
        (Some(old), Some(new)) => Ok(vec![Edit {
            old: to_lf(old),
            new: to_lf(new),
        }]),
        _ => Err("argumentos 'old' e 'new' (ou 'edits') sao obrigatorios".to_owned()),
    }
}

#[async_trait]
impl ToolExecutor for FsTools<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![
            spec(
                "read_file",
                "Le um ficheiro de texto dentro do projecto (offset = linha inicial, 1-based).",
                r#"{"type":"object","properties":{"path":{"type":"string"},"offset":{"type":"integer"}},"required":["path"]}"#,
            ),
            spec(
                "write_file",
                "Escreve (cria ou substitui) um ficheiro dentro do projecto.",
                r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#,
            ),
            spec(
                "edit_file",
                "Aplica uma ou mais substituicoes (ancoras unicas) a um ficheiro, atomicamente.",
                r#"{"type":"object","properties":{"path":{"type":"string"},"edits":{"type":"array","items":{"type":"object","properties":{"old":{"type":"string"},"new":{"type":"string"}},"required":["old","new"]}},"old":{"type":"string"},"new":{"type":"string"}},"required":["path"]}"#,
            ),
            spec(
                "list_dir",
                "Lista as entradas de um directório do projecto.",
                r#"{"type":"object","properties":{"path":{"type":"string"}}}"#,
            ),
            spec(
                "glob",
                "Lista ficheiros que casam com um padrao (ex.: src/**/*.rs).",
                r#"{"type":"object","properties":{"pattern":{"type":"string"}},"required":["pattern"]}"#,
            ),
            spec(
                "grep",
                "Procura um texto nos ficheiros do projecto e devolve ficheiro:linha.",
                r#"{"type":"object","properties":{"pattern":{"type":"string"}},"required":["pattern"]}"#,
            ),
        ]
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        let args: Value = serde_json::from_str(&call.arguments).unwrap_or(Value::Null);
        match call.name.as_str() {
            "read_file" => self.read(&args),
            "write_file" => self.write(&args).await,
            "edit_file" => self.edit(&args).await,
            "list_dir" => self.list(&args),
            "glob" => self.glob(&args),
            "grep" => self.grep(&args),
            other => ToolOutcome::error(format!("tool desconhecida: {other}")),
        }
    }
}

/// Lista de nomes das ferramentas (para o `AgentSpec`/`guard`).
#[must_use]
pub fn tool_names() -> Vec<String> {
    [
        "read_file",
        "write_file",
        "edit_file",
        "list_dir",
        "glob",
        "grep",
        "bash",
        "trash_file",
        "memory_ask",
        "memory_write",
    ]
    .iter()
    .map(|name| (*name).to_owned())
    .collect()
}
