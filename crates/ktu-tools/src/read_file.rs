//! Tool `read_file`: lê um ficheiro dentro do *root* da sessão.
//!
//! O conteúdo entra no contexto dentro do envelope *taint* (I13): é **dado**,
//! nunca instrução. O confinamento de path é verificado duas vezes — no guard
//! (kernel) e aqui (defesa em profundidade).

use std::path::PathBuf;

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::guard::{path_argument, resolve_within};
use ktu_core::ports::Fs;
use ktu_core::taint::wrap_untrusted;
use ktu_core::tools::{ToolExecutor, ToolOutcome};

/// Limite por omissão do conteúdo lido (bytes).
const DEFAULT_MAX_BYTES: usize = 65_536;

/// Tool `read_file`.
pub struct ReadFileTool<'a> {
    root: PathBuf,
    fs: &'a dyn Fs,
    max_bytes: usize,
}

impl<'a> ReadFileTool<'a> {
    /// Cria a tool com o *root* e a porta de ficheiros.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, fs: &'a dyn Fs) -> Self {
        Self {
            root: root.into(),
            fs,
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }
}

#[async_trait]
impl ToolExecutor for ReadFileTool<'_> {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "read_file".to_owned(),
            description: "Le um ficheiro de texto dentro do projecto.".to_owned(),
            parameters:
                r#"{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}"#
                    .to_owned(),
        }]
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        let Some(requested) = path_argument(&call.arguments) else {
            return ToolOutcome::error("argumento 'path' em falta ou invalido");
        };
        let Some(resolved) = resolve_within(&self.root, &requested) else {
            return ToolOutcome::error("path recusado: sai do root da sessao");
        };
        match self.fs.read(&resolved) {
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
}
