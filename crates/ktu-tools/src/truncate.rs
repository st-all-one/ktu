//! Truncagem sistemática de output (linhas + bytes) — **uma** política para todas
//! as tools.
//!
//! Dois limites independentes, o que vier primeiro: linhas e bytes. Nunca devolve
//! linhas parciais (salvo o caso de uma linha única maior que o teto, em que se
//! devolve um prefixo para garantir progresso). O output completo pode ser
//! guardado em `.ktu/out/` e o caminho devolvido ao modelo ([`spill`]).

use std::path::Path;

/// Limite por omissão de linhas devolvidas.
pub const MAX_LINES: usize = 2_000;

/// Limite por omissão de bytes devolvidos.
pub const MAX_BYTES: usize = 50_000;

/// Resultado de uma truncagem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Truncated {
    /// Texto dentro do teto.
    pub text: String,
    /// Linhas totais do texto original.
    pub total_lines: usize,
    /// `true` se houve corte.
    pub is_cut: bool,
}

/// Trunca `text` por linhas **e** bytes (o que vier primeiro), sem linhas parciais.
#[must_use]
pub fn truncate(text: &str, max_lines: usize, max_bytes: usize) -> Truncated {
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();
    let mut out = String::new();
    let mut count = 0usize;
    for line in &lines {
        if count >= max_lines {
            break;
        }
        let projected = out.len().saturating_add(line.len()).saturating_add(1);
        if projected > max_bytes {
            if count == 0 {
                out.push_str(prefix_bytes(line, max_bytes));
                out.push('\n');
            }
            break;
        }
        out.push_str(line);
        out.push('\n');
        count = count.saturating_add(1);
    }
    if count == total_lines {
        return Truncated {
            text: text.to_owned(),
            total_lines,
            is_cut: false,
        };
    }
    Truncated {
        text: out,
        total_lines,
        is_cut: true,
    }
}

/// Guarda o output completo em `.ktu/out/<tool>-<hash>.txt` e devolve o caminho
/// relativo ao *root* (ou `None` se a gravação falhar).
#[must_use]
pub fn spill(root: &Path, tool: &str, text: &str) -> Option<String> {
    let dir = root.join(".ktu").join("out");
    std::fs::create_dir_all(&dir).ok()?;
    let name = format!("{tool}-{:08x}.txt", fnv1a(text.as_bytes()));
    std::fs::write(dir.join(&name), text).ok()?;
    Some(format!(".ktu/out/{name}"))
}

/// Prefixo de `text` em bytes, preservando fronteiras de caractere.
fn prefix_bytes(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    text.get(..end).unwrap_or("")
}

/// FNV-1a de 32 bits (determinístico, sem dependências).
fn fnv1a(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{MAX_BYTES, MAX_LINES, spill, truncate};

    #[test]
    fn keeps_small_text_untouched() {
        let result = truncate("a\nb\n", MAX_LINES, MAX_BYTES);
        assert!(!result.is_cut);
        assert_eq!(result.text, "a\nb\n");
        assert_eq!(result.total_lines, 2);
    }

    #[test]
    fn cuts_at_the_line_limit() {
        let text = (1..=10)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let result = truncate(&text, 3, MAX_BYTES);
        assert!(result.is_cut);
        assert_eq!(result.text, "1\n2\n3\n");
        assert_eq!(result.total_lines, 10);
    }

    #[test]
    fn cuts_at_the_byte_limit() {
        let result = truncate("aaaa\nbbbb\ncccc\n", MAX_LINES, 9);
        assert!(result.is_cut);
        assert_eq!(result.text, "aaaa\n");
    }

    #[test]
    fn keeps_a_prefix_of_a_giant_single_line() {
        let giant = "x".repeat(100);
        let result = truncate(&giant, MAX_LINES, 10);
        assert!(result.is_cut);
        assert_eq!(result.text, "xxxxxxxxxx\n");
    }

    #[test]
    fn spill_is_deterministic() {
        let root = std::env::temp_dir().join(format!("ktu-spill-{}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        let first = spill(&root, "bash", "conteudo");
        let second = spill(&root, "bash", "conteudo");
        assert_eq!(first, second);
        assert!(first.is_some_and(|path| root.join(path).is_file()));
        let _ignored = std::fs::remove_dir_all(&root);
    }
}
