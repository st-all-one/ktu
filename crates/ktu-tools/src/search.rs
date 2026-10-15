//! Pesquisa no *root*: `glob` (padrão) e `grep` (texto), sem crates externas.
//!
//! O *walk* usa a porta [`Fs`] (nada de `std::fs` aqui); o *matching* é puro e
//! testável (`*`, `**`, `?`).

use std::path::Path;

use ktu_core::ports::Fs;

/// Máximo de resultados devolvidos por pesquisa.
pub(crate) const MAX_RESULTS: usize = 200;

/// Profundidade máxima de recursão do *walk*.
const MAX_DEPTH: u32 = 32;

/// Devolve os caminhos relativos (com `/`) que casam com `pattern`.
pub(crate) fn glob(fs: &dyn Fs, root: &Path, pattern: &str) -> Vec<String> {
    let mut out = Vec::new();
    walk(fs, root, root, 0, &mut |relative| {
        if out.len() < MAX_RESULTS && glob_match(pattern, relative) {
            out.push(relative.to_owned());
        }
    });
    out.sort();
    out
}

/// Devolve `ficheiro:linha: texto` das linhas que contêm `query`.
pub(crate) fn grep(fs: &dyn Fs, root: &Path, query: &str) -> Vec<String> {
    let mut out = Vec::new();
    walk(fs, root, root, 0, &mut |relative| {
        if out.len() >= MAX_RESULTS {
            return;
        }
        let shot = relative.to_owned();
        let path = relative_path(root, relative);
        let Ok(bytes) = fs.read(&path) else {
            return;
        };
        let text = String::from_utf8_lossy(&bytes);
        for (index, line) in text.lines().enumerate() {
            if line.contains(query) {
                let number = index.saturating_add(1);
                out.push(format!("{shot}:{number}: {}", line.trim()));
                if out.len() >= MAX_RESULTS {
                    return;
                }
            }
        }
    });
    out
}

fn relative_path(root: &Path, relative: &str) -> std::path::PathBuf {
    let mut path = root.to_path_buf();
    for segment in relative.split('/') {
        path.push(segment);
    }
    path
}

/// Percorre `dir` recursivamente, chamando `visit` com o caminho relativo.
fn walk(fs: &dyn Fs, root: &Path, dir: &Path, depth: u32, visit: &mut dyn FnMut(&str)) {
    if depth > MAX_DEPTH {
        return;
    }
    let Ok(entries) = fs.read_dir(dir) else {
        return;
    };
    for entry in entries {
        let relative = entry
            .strip_prefix(root)
            .unwrap_or(&entry)
            .to_string_lossy()
            .replace('\\', "/");
        if fs.is_dir(&entry) {
            walk(fs, root, &entry, depth.saturating_add(1), visit);
        } else {
            visit(&relative);
        }
    }
}

/// Casa um padrão glob (com `/`, `*`, `**`, `?`) contra um caminho relativo.
#[must_use]
pub(crate) fn glob_match(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    match_segments(&pattern, &path)
}

fn match_segments(pattern: &[&str], path: &[&str]) -> bool {
    match (pattern.split_first(), path.split_first()) {
        (None, None) => true,
        (None, Some(_)) | (Some(_), None) => false,
        (Some((&"**", rest)), _) => {
            match_segments(rest, path)
                || path
                    .split_first()
                    .is_some_and(|(_, tail)| match_segments(pattern, tail))
        }
        (Some((&segment, rest)), Some((&part, tail))) => {
            match_segment(segment, part) && match_segments(rest, tail)
        }
    }
}

fn match_segment(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    wildcard(&pattern, &text)
}

fn wildcard(pattern: &[char], text: &[char]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some((&'*', rest)) => {
            wildcard(rest, text)
                || text
                    .split_first()
                    .is_some_and(|(_, tail)| wildcard(pattern, tail))
        }
        Some((&'?', rest)) => text
            .split_first()
            .is_some_and(|(_, tail)| wildcard(rest, tail)),
        Some((&expected, rest)) => text
            .split_first()
            .is_some_and(|(&actual, tail)| actual == expected && wildcard(rest, tail)),
    }
}

#[cfg(test)]
mod tests {
    use super::glob_match;

    #[test]
    fn matches_simple_and_recursive_patterns() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.toml"));
        assert!(glob_match("src/**/*.rs", "src/a/b/main.rs"));
        assert!(glob_match("src/**/*.rs", "src/main.rs"));
        assert!(glob_match("a/?.txt", "a/b.txt"));
        assert!(!glob_match("a/?.txt", "a/bb.txt"));
    }
}
