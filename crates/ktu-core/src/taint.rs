//! Envelope *taint* para conteúdo não confiável (I13).
//!
//! Output de tool e conteúdo de ficheiro são **dado**, nunca instrução. O
//! payload viaja entre tags; o escape troca `[` por `(` — **mesmo comprimento em
//! bytes** — pelo que uma tag de fecho crua nunca pode aparecer no interior.

/// Prefixo da tag de abertura.
pub const OPEN_PREFIX: &str = "[untrusted: ";
/// Sufixo da tag de abertura.
pub const OPEN_SUFFIX: &str = "]";
/// Tag de fecho.
pub const CLOSE: &str = "[/untrusted]";

/// Envolve `content` (dado não confiável) num envelope identificado por `source`.
#[must_use]
pub fn wrap_untrusted(source: &str, content: &str) -> String {
    let safe_source = sanitize(source);
    let body = escape(content);
    format!("{OPEN_PREFIX}{safe_source}{OPEN_SUFFIX}\n{body}\n{CLOSE}")
}

/// Escapa `[` → `(` (preserva o comprimento em bytes e remove tags cruas).
#[must_use]
pub fn escape(content: &str) -> String {
    content.replace('[', "(")
}

/// Reduz `source` a caracteres seguros para a tag.
fn sanitize(source: &str) -> String {
    source
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CLOSE, escape, wrap_untrusted};

    #[test]
    fn escape_preserves_length_and_removes_open_bracket() {
        let content = "a[b]c[/untrusted]";
        let escaped = escape(content);
        assert_eq!(escaped.len(), content.len());
        assert!(!escaped.contains('['));
    }

    #[test]
    fn wrap_has_exactly_one_close_tag() {
        let wrapped = wrap_untrusted("read_file", "texto [x] [/untrusted] fim");
        assert_eq!(wrapped.matches(CLOSE).count(), 1);
    }

    #[test]
    fn wrap_sanitizes_the_source() {
        let wrapped = wrap_untrusted("read file!", "corpo");
        assert!(wrapped.starts_with("[untrusted: readfile]"));
    }
}
