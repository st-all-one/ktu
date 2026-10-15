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
use std::path::PathBuf;

use crate::path::{resolve_existing, resolve_for_write};
use crate::search;

/// Limite por omissão do conteúdo lido (bytes).
const DEFAULT_MAX_BYTES: usize = 65_536;

/// Conjunto de ferramentas de *filesystem* da sessão.
pub struct FsTools<'a> {
    root: PathBuf,
    fs: &'a dyn Fs,
    max_bytes: usize,
}

impl<'a> FsTools<'a> {
    /// Cria o conjunto com o *root* e a porta de ficheiros.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, fs: &'a dyn Fs) -> Self {
        Self {
            root: root.into(),
            fs,
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }

    fn read(&self, args: &Value) -> ToolOutcome {
        let Some(requested) = args.get("path").and_then(Value::as_str) else {
            return ToolOutcome::error("argumento 'path' em falta");
        };
        let path = match resolve_existing(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        match self.fs.read(&path) {
            Ok(bytes) => {
                let limit = self.max_bytes.min(bytes.len());
                let slice = bytes.get(..limit).unwrap_or(&[]);
                let mut text = String::from_utf8_lossy(slice).into_owned();
                if bytes.len() > self.max_bytes {
                    text.push_str("\n... [conteudo truncado]");
                }
                ToolOutcome::ok(wrap_untrusted("read_file", &text))
            }
            Err(error) => ToolOutcome::error(format!("nao foi possivel ler {requested}: {error}")),
        }
    }

    fn write(&self, args: &Value) -> ToolOutcome {
        let requested = args.get("path").and_then(Value::as_str);
        let content = args.get("content").and_then(Value::as_str);
        let (Some(requested), Some(content)) = (requested, content) else {
            return ToolOutcome::error("argumentos 'path' e 'content' sao obrigatorios");
        };
        let path = match resolve_for_write(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        match self.fs.write(&path, content.as_bytes()) {
            Ok(()) => ToolOutcome::ok(format!("escrito {requested} ({} bytes)", content.len())),
            Err(error) => {
                ToolOutcome::error(format!("nao foi possivel escrever {requested}: {error}"))
            }
        }
    }

    fn edit(&self, args: &Value) -> ToolOutcome {
        let requested = args.get("path").and_then(Value::as_str);
        let old = args.get("old").and_then(Value::as_str);
        let new = args.get("new").and_then(Value::as_str);
        let (Some(requested), Some(old), Some(new)) = (requested, old, new) else {
            return ToolOutcome::error("argumentos 'path', 'old' e 'new' sao obrigatorios");
        };
        let path = match resolve_existing(self.fs, &self.root, requested) {
            Ok(path) => path,
            Err(message) => return ToolOutcome::error(message),
        };
        let Ok(bytes) = self.fs.read(&path) else {
            return ToolOutcome::error(format!("nao foi possivel ler {requested}"));
        };
        let content = String::from_utf8_lossy(&bytes).into_owned();
        match content.matches(old).count() {
            0 => ToolOutcome::error(format!("texto nao encontrado em {requested}")),
            1 => {
                let updated = content.replacen(old, new, 1);
                match self.fs.write(&path, updated.as_bytes()) {
                    Ok(()) => ToolOutcome::ok(format!("editado {requested}")),
                    Err(error) => ToolOutcome::error(format!(
                        "nao foi possivel escrever {requested}: {error}"
                    )),
                }
            }
            count => {
                ToolOutcome::error(format!("texto ambiguo em {requested}: {count} ocorrencias"))
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

#[async_trait]
impl ToolExecutor for FsTools<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![
            spec(
                "read_file",
                "Le um ficheiro de texto dentro do projecto.",
                r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#,
            ),
            spec(
                "write_file",
                "Escreve (cria ou substitui) um ficheiro dentro do projecto.",
                r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#,
            ),
            spec(
                "edit_file",
                "Substitui a unica ocorrencia de um texto num ficheiro existente.",
                r#"{"type":"object","properties":{"path":{"type":"string"},"old":{"type":"string"},"new":{"type":"string"}},"required":["path","old","new"]}"#,
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
            "write_file" => self.write(&args),
            "edit_file" => self.edit(&args),
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
