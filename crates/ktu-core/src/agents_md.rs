//! Instruções por directório: descoberta de `AGENTS.md` **aninhados** (uso #4).
//!
//! O `AGENTS.md` da raiz é o *router* do projecto; um `AGENTS.md` mais fundo dá
//! instruções **locais** (monorepos/módulos). A descoberta é determinística —
//! raiz primeiro e, depois, os aninhados por ordem de caminho (o mais fundo, mais
//! tarde no prompt) — e **limitada** (profundidade, ficheiros, bytes e
//! directórios visitados) para não estourar o contexto nem o tempo de arranque.
//!
//! Fail-open: nada disto é erro; um ficheiro ilegível é simplesmente ignorado.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::layout::AGENTS_FILE;
use crate::ports::Fs;
use crate::prompt::condense;

/// Profundidade máxima de descoberta (níveis abaixo da raiz).
pub const MAX_DEPTH: usize = 8;
/// Directórios visitados no máximo (tecto de custo de arranque).
pub const MAX_DIRS: usize = 512;
/// Tecto de bytes por ficheiro (maior ⇒ ignorado).
pub const MAX_FILE_BYTES: usize = 8 * 1024;
/// Tecto de bytes no total (deixa de acrescentar documentos).
pub const MAX_TOTAL_BYTES: usize = 32 * 1024;
/// Directórios nunca percorridos (referências, *vendor*, artefactos de *build*).
///
/// Os directórios começados por `.` são sempre saltados (`.git`, `.ktu`,
/// `.agents`, `.venv`, …).
pub const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "vendor",
    "third_party",
    "dist",
    "build",
    "out",
    "REF",
    "ref",
    "__pycache__",
    "Pods",
];

/// Um documento de instruções descoberto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doc {
    /// Caminho absoluto do `AGENTS.md`.
    pub path: PathBuf,
    /// Texto do ficheiro, como lido.
    pub text: String,
}

/// Descobre os `AGENTS.md` do projecto (raiz primeiro, aninhados por caminho).
#[must_use]
pub fn discover(fs: &dyn Fs, root: &Path) -> Vec<Doc> {
    let mut docs = Vec::new();
    let mut total = 0usize;
    if let Some(doc) = read_doc(fs, root) {
        total = total.saturating_add(doc.text.len());
        docs.push(doc);
    }
    let mut pending: BTreeSet<PathBuf> = BTreeSet::new();
    for entry in fs.read_dir(root).unwrap_or_default() {
        if is_candidate(fs, root, &entry, 1) {
            let _inserted = pending.insert(entry);
        }
    }
    let mut visited = 0usize;
    while let Some(dir) = pending.pop_first() {
        visited = visited.saturating_add(1);
        if visited > MAX_DIRS {
            break;
        }
        let depth = depth_of(root, &dir);
        if let Some(doc) = read_doc(fs, &dir)
            && total.saturating_add(doc.text.len()) <= MAX_TOTAL_BYTES
        {
            total = total.saturating_add(doc.text.len());
            docs.push(doc);
        }
        if depth >= MAX_DEPTH {
            continue;
        }
        for entry in fs.read_dir(&dir).unwrap_or_default() {
            if is_candidate(fs, root, &entry, depth.saturating_add(1)) {
                let _inserted = pending.insert(entry);
            }
        }
    }
    docs
}

/// `AGENTS.md` **mais próximo** de `path` (o próprio directório, subindo até `root`).
///
/// É a regra "o mais próximo sobrepõe": para um ficheiro em `a/b/c.rs`, ganha o
/// `AGENTS.md` de `a/b`, depois o de `a`, depois o da raiz.
#[must_use]
pub fn nearest(fs: &dyn Fs, root: &Path, path: &Path) -> Option<Doc> {
    let mut dir = path.parent()?.to_path_buf();
    loop {
        if let Some(doc) = read_doc(fs, &dir) {
            return Some(doc);
        }
        if dir == root {
            return None;
        }
        let parent = dir.parent()?;
        if parent.as_os_str().is_empty() {
            return None;
        }
        dir = parent.to_path_buf();
    }
}

/// Junta os documentos num bloco único: a raiz sem cabeçalho, os aninhados com
/// um cabeçalho `## <caminho relativo>` (o modelo sabe de onde vem cada regra).
#[must_use]
pub fn render(docs: &[Doc], root: &Path) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    for doc in docs {
        let text = condense(&doc.text);
        if text.is_empty() {
            continue;
        }
        let rel = label(root, &doc.path);
        if rel == AGENTS_FILE {
            parts.push(text);
        } else {
            parts.push(format!("## {rel}\n\n{text}"));
        }
    }
    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

/// Lê o `AGENTS.md` de um directório (fail-open, com tecto de tamanho).
fn read_doc(fs: &dyn Fs, dir: &Path) -> Option<Doc> {
    let path = dir.join(AGENTS_FILE);
    let bytes = fs.read(&path).ok()?;
    if bytes.is_empty() || bytes.len() > MAX_FILE_BYTES {
        return None;
    }
    let text = String::from_utf8(bytes).ok()?;
    (!text.trim().is_empty()).then_some(Doc { path, text })
}

/// `true` se o directório deve ser percorrido (não é oculto nem conhecido).
fn is_candidate(fs: &dyn Fs, root: &Path, entry: &Path, depth: usize) -> bool {
    if depth > MAX_DEPTH || !fs.is_dir(entry) || !entry.starts_with(root) {
        return false;
    }
    let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    !name.starts_with('.') && !SKIP_DIRS.contains(&name)
}

/// Profundidade de `dir` abaixo de `root` (raiz = 0).
fn depth_of(root: &Path, dir: &Path) -> usize {
    dir.strip_prefix(root)
        .map_or(0, |rel| rel.components().count())
}

/// Rótulo legível de um documento: caminho relativo a `root`, com separadores
/// `/` (estável entre sistemas). É a **chave de deduplicação** das instruções locais.
#[must_use]
pub fn label(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Doc, MAX_DEPTH, discover, label, nearest, render};
    use crate::ports::MemFs;

    const ROOT: &str = "/proj";

    fn fixture() -> MemFs {
        MemFs::new()
            .with_file(Path::new(ROOT).join("AGENTS.md"), "# raiz\n\nregra global")
            .with_file(
                Path::new(ROOT).join("a").join("AGENTS.md"),
                "# a\n\nregra de a",
            )
            .with_file(
                Path::new(ROOT).join("a").join("b").join("AGENTS.md"),
                "# b\n\nregra de b",
            )
            .with_file(
                Path::new(ROOT).join("target").join("AGENTS.md"),
                "# build\n\nnao conta",
            )
            .with_file(
                Path::new(ROOT).join(".git").join("AGENTS.md"),
                "# git\n\nnao conta",
            )
            .with_file(Path::new(ROOT).join("a").join("codigo.rs"), "fn main() {}")
    }

    #[test]
    fn discovers_root_first_then_nested_by_path() {
        let docs = discover(&fixture(), Path::new(ROOT));
        let rels: Vec<String> = docs
            .iter()
            .map(|doc| label(Path::new(ROOT), &doc.path))
            .collect();
        assert_eq!(rels, ["AGENTS.md", "a/AGENTS.md", "a/b/AGENTS.md"]);
    }

    #[test]
    fn skips_hidden_and_known_directories() {
        let docs = discover(&fixture(), Path::new(ROOT));
        assert!(docs.iter().all(|doc| !doc.text.contains("nao conta")));
    }

    #[test]
    fn renders_the_root_without_a_heading_and_the_rest_with_one() {
        let docs = discover(&fixture(), Path::new(ROOT));
        let text = render(&docs, Path::new(ROOT)).unwrap_or_default();
        assert!(text.starts_with("# raiz"));
        assert!(text.contains("regra global"));
        assert!(!text.contains("## AGENTS.md"), "a raiz não leva cabeçalho");
        assert!(text.contains("## a/AGENTS.md\n\n# a\n\nregra de a"));
        assert!(text.contains("## a/b/AGENTS.md\n\n# b\n\nregra de b"));
        assert_eq!(
            render(&[], Path::new(ROOT)),
            None,
            "sem documentos não há bloco"
        );
    }

    #[test]
    fn nearest_returns_the_closest_document_up_to_the_root() {
        let fs = fixture();
        let root = Path::new(ROOT);
        let deep = nearest(&fs, root, &root.join("a").join("b").join("codigo.rs"));
        assert_eq!(
            deep.map(|doc| doc.text),
            Some("# b\n\nregra de b".to_owned())
        );
        let middle = nearest(&fs, root, &root.join("a").join("codigo.rs"));
        assert_eq!(
            middle.map(|doc| doc.text),
            Some("# a\n\nregra de a".to_owned())
        );
        let elsewhere = nearest(&fs, root, &root.join("outro.rs"));
        assert_eq!(
            elsewhere.map(|doc| doc.text),
            Some("# raiz\n\nregra global".to_owned())
        );
        let empty = MemFs::new();
        assert!(nearest(&empty, root, &root.join("outro.rs")).is_none());
    }

    #[test]
    fn stops_at_the_depth_limit() {
        let mut fs = MemFs::new();
        let mut dir = Path::new(ROOT).to_path_buf();
        for level in 0..MAX_DEPTH.saturating_add(2) {
            fs = fs.with_file(dir.join("AGENTS.md"), format!("nivel {level}"));
            dir = dir.join("n");
        }
        let docs = discover(&fs, Path::new(ROOT));
        // raiz (nível 0) + um documento por nível até `MAX_DEPTH`.
        assert_eq!(docs.len(), MAX_DEPTH.saturating_add(1));
    }

    #[test]
    fn empty_and_oversized_files_are_ignored() {
        let fs = MemFs::new()
            .with_file(Path::new(ROOT).join("AGENTS.md"), "   ")
            .with_file(
                Path::new(ROOT).join("a").join("AGENTS.md"),
                "x".repeat(9000),
            );
        assert_eq!(discover(&fs, Path::new(ROOT)), Vec::new());
    }

    #[test]
    fn doc_equality_is_structural() {
        let left = Doc {
            path: Path::new(ROOT).join("AGENTS.md"),
            text: "x".to_owned(),
        };
        assert_eq!(left.clone(), left);
    }
}
