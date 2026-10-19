//! `check-links` — verifica os links internos da documentação (L3, F7).
//!
//! A wiki do ktu é feita de decisões que se citam umas às outras: um link quebrado
//! torna uma decisão **sem fonte** (PM-0003). Nenhum outro travão olhava para ela —
//! o `cargo doc` cobre os links *intra-doc* do Rust, não o Markdown.
//!
//! Contrato:
//! - varre `wiki/**/*.md` + `README.md` + `STAGING.md` + `plan.md`;
//! - resolve cada destino relativo `](...)` contra o directório do ficheiro;
//! - ignora `http(s)://`, `mailto:` e links só-de-âncora (`#…`);
//! - ignora o que está em *código* (blocos cercados por crases e `código em linha`):
//!   ali um `](…)` é um exemplo, não um link — com a semântica do *`CommonMark`*, uma
//!   sequência de crases sem fecho do mesmo tamanho é texto literal (não engole o
//!   resto do ficheiro) e uma crase escapada por barra invertida não abre código;
//! - a âncora (`#…`) **não** é erro: valida-se o **ficheiro**; se a âncora não
//!   existir no destino, fica um aviso (as âncoras do GitHub derivam dos títulos e
//!   uma mudança de título não deve quebrar o portão).

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Ficheiros de topo que também são documentação.
const ROOT_FILES: [&str; 3] = ["README.md", "STAGING.md", "plan.md"];

/// Um link relativo encontrado (ficheiro, alvo literal e destino resolvido).
struct Link {
    /// Ficheiro onde o link aparece (relativo à raiz).
    source: PathBuf,
    /// Alvo como escrito (`../gates/G19-edicao.md#ancora`).
    target: String,
    /// Destino resolvido (sem âncora).
    resolved: PathBuf,
    /// Âncora pedida (`Some` se o alvo tinha `#…` não vazio).
    anchor: Option<String>,
}

/// Verifica os links sob `--root` (por omissão, a raiz do repositório).
///
/// `--root` existe para o gate G22: aponta a verificação a uma árvore de ensaio com
/// um link deliberadamente quebrado (a verificação real corre na raiz).
pub(crate) fn check(args: &[String]) -> Result<(), String> {
    let mut root = PathBuf::from(".");
    let mut raw = args.iter();
    while let Some(arg) = raw.next() {
        match arg.as_str() {
            "--root" => root = PathBuf::from(raw.next().ok_or("--root exige um caminho")?),
            other => return Err(format!("argumento desconhecido: {other}")),
        }
    }
    let mut sources: Vec<PathBuf> = Vec::new();
    collect_markdown(&root.join("wiki"), &mut sources)?;
    for name in ROOT_FILES {
        let path = root.join(name);
        if path.exists() {
            sources.push(path);
        }
    }
    sources.sort();
    let mut broken: BTreeSet<String> = BTreeSet::new();
    let mut warnings: BTreeSet<String> = BTreeSet::new();
    let mut total = 0usize;
    for source in &sources {
        let text =
            fs::read_to_string(source).map_err(|error| format!("{}: {error}", source.display()))?;
        for link in links_of(source, &strip_code(&text)) {
            total = total.saturating_add(1);
            if !link.resolved.exists() {
                broken.insert(format!("{} → {}", link.source.display(), link.target));
                continue;
            }
            if let Some(anchor) = &link.anchor
                && !has_anchor(&link.resolved, anchor)
            {
                warnings.insert(format!(
                    "{} → {} (ancora `#{anchor}` nao encontrada)",
                    link.source.display(),
                    link.target
                ));
            }
        }
    }
    report(total, &warnings)?;
    if broken.is_empty() {
        return Ok(());
    }
    let list = broken.into_iter().collect::<Vec<_>>().join("\n  ");
    Err(format!("links quebrados:\n  {list}"))
}

/// Escreve o aviso das âncoras em `stderr` (logs) e devolve o número de links vistos.
fn report(total: usize, warnings: &BTreeSet<String>) -> Result<(), String> {
    let mut stderr = io::stderr().lock();
    let mut write = |text: String| -> Result<(), String> {
        write!(stderr, "{text}").map_err(|error| error.to_string())
    };
    for warning in warnings {
        write(format!("check-links: aviso: {warning}\n"))?;
    }
    write(format!(
        "check-links: {total} links relativos, {} avisos\n",
        warnings.len()
    ))
}

/// Reúne os `*.md` de um directório (ordem estável, recursivo).
fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let mut children: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        children.push(entry.path());
    }
    children.sort();
    for path in children {
        if path.is_dir() {
            collect_markdown(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
    Ok(())
}

/// Extrai os links relativos de um texto (já sem código), contra o directório do ficheiro.
fn links_of(source: &Path, text: &str) -> Vec<Link> {
    let base = source.parent().unwrap_or_else(|| Path::new("."));
    let mut out = Vec::new();
    for line in text.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let tail = rest.get(start.saturating_add(2)..).unwrap_or_default();
            let Some(end) = tail.find(')') else {
                break;
            };
            let target = tail.get(..end).unwrap_or_default().trim();
            rest = tail.get(end..).unwrap_or_default();
            if let Some(link) = resolve(source, base, target) {
                out.push(link);
            }
        }
    }
    out
}

/// Constrói o link, ou `None` se for externo/só-âncora/vazio.
fn resolve(source: &Path, base: &Path, target: &str) -> Option<Link> {
    if target.is_empty()
        || target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
    {
        return None;
    }
    let (path, anchor) = match target.split_once('#') {
        Some((path, anchor)) if !anchor.is_empty() => (path, Some(anchor.to_owned())),
        Some((path, _)) => (path, None),
        None => (target, None),
    };
    if path.is_empty() {
        return None;
    }
    let path = path.split('?').next().unwrap_or(path);
    let resolved = if let Some(absolute) = path.strip_prefix('/') {
        PathBuf::from(absolute)
    } else {
        base.join(path)
    };
    Some(Link {
        source: source.to_path_buf(),
        target: target.to_owned(),
        resolved: normalise(&resolved),
        anchor,
    })
}

/// Resolve `.` e `..` sem tocar no disco (o destino pode não existir).
fn normalise(path: &Path) -> PathBuf {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                let _popped = parts.pop();
            }
            other => parts.push(other.as_os_str().to_os_string()),
        }
    }
    let mut out = PathBuf::new();
    for part in parts {
        out.push(part);
    }
    out
}

/// Remove blocos de código e código em linha (onde um `](…)` é exemplo).
///
/// Segue o essencial do `CommonMark`: uma sequência de crases sem fecho do **mesmo**
/// tamanho é texto literal (não engole o resto do ficheiro) e uma crase escapada por
/// barra invertida não abre código — é assim que a wiki real escreve a dica do
/// `kd init` dentro de um trecho de código.
fn strip_code(text: &str) -> String {
    let mut open_fence = false;
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            open_fence = !open_fence;
        } else if !open_fence {
            out.push_str(line);
        }
        out.push('\n');
    }
    strip_inline_code(&out)
}

/// Remove os trechos entre crases de um texto.
fn strip_inline_code(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let Some(index) = find_open_tick(rest) else {
            out.push_str(rest);
            return out;
        };
        let (before, after) = rest.split_at(index);
        out.push_str(before);
        let fence = after
            .chars()
            .take_while(|character| *character == '`')
            .count();
        let open = "`".repeat(fence);
        let tail = after.get(fence..).unwrap_or_default();
        let Some(close) = tail.find(&open) else {
            // Sem fecho: a sequência é texto literal (o resto continua a ser verificado).
            out.push_str(&open);
            rest = tail;
            continue;
        };
        rest = tail.get(close.saturating_add(fence)..).unwrap_or_default();
    }
}

/// Primeira crase que **não** está escapada por barra invertida.
fn find_open_tick(text: &str) -> Option<usize> {
    let mut from = 0;
    loop {
        let index = text.get(from..)?.find('`')?;
        let absolute = from.saturating_add(index);
        let escaped = text.get(absolute.saturating_sub(1)..absolute) == Some("\\");
        if !escaped {
            return Some(absolute);
        }
        from = absolute.saturating_add(1);
    }
}

/// `true` se o ficheiro tem um título que gera a âncora (slug do GitHub).
fn has_anchor(path: &Path, anchor: &str) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let wanted = anchor.to_lowercase();
    for line in text.lines() {
        let Some(title) = line.trim_start().strip_prefix('#') else {
            continue;
        };
        if slug(title.trim_start_matches('#').trim()) == wanted {
            return true;
        }
    }
    false
}

/// Slug de um título, como o GitHub: minúsculas, pontuação fora, espaços → `-`.
fn slug(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    for character in title.chars() {
        if character.is_alphanumeric() {
            out.extend(character.to_lowercase());
        } else if character == ' ' || character == '-' || character == '_' {
            out.push('-');
        }
    }
    out
}
