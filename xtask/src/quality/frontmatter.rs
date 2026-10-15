//! Leitura e emissão de *frontmatter* `chave: valor` (subconjunto YAML).

use std::collections::BTreeMap;

/// Cerca que delimita o bloco de frontmatter.
const FENCE: &str = "---";

/// Separa o bloco de frontmatter do corpo. `None` se o bloco faltar ou for malformado.
#[must_use]
pub(crate) fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix(FENCE)?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))?;
    let marker = format!("\n{FENCE}");
    let end = rest.find(&marker)?;
    let block = rest.get(..end)?;
    let tail = rest.get(end..)?;
    let after = tail.strip_prefix(&marker)?;
    let body = after
        .strip_prefix("\r\n")
        .or_else(|| after.strip_prefix('\n'))
        .unwrap_or(after);
    Some((block, body))
}

/// Mapa de chaves do frontmatter e o corpo restante.
pub(crate) type Parsed = (BTreeMap<String, String>, String);

/// Interpreta o frontmatter devolvendo o mapa de chaves e o corpo.
#[must_use]
pub(crate) fn parse(text: &str) -> Option<Parsed> {
    let (block, body) = split(text)?;
    let mut map = BTreeMap::new();
    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once(':')?;
        map.insert(key.trim().to_owned(), unquote(value.trim()));
    }
    Some((map, body.to_owned()))
}

/// Emite um bloco de frontmatter com os pares dados (ordem preservada) e o corpo.
#[must_use]
pub(crate) fn render(pairs: &[(&str, &str)], body: &str) -> String {
    let mut out = String::new();
    out.push_str(FENCE);
    out.push('\n');
    for (key, value) in pairs {
        out.push_str(key);
        out.push_str(": ");
        out.push_str(&quote(value));
        out.push('\n');
    }
    out.push_str(FENCE);
    out.push('\n');
    out.push_str(body.trim_end());
    out.push('\n');
    out
}

/// Remove aspas e expande escapes simples de um valor.
fn unquote(raw: &str) -> String {
    let trimmed = raw.trim();
    let quoted = trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"');
    if !quoted {
        return trimmed.to_owned();
    }
    let inner = trimmed
        .get(1..trimmed.len().saturating_sub(1))
        .unwrap_or("");
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// Cita um valor quando ele não é um token simples.
fn quote(value: &str) -> String {
    let simple = !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/'));
    if simple {
        return value.to_owned();
    }
    let mut out = String::with_capacity(value.len().saturating_add(2));
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::{parse, render, split};

    #[test]
    fn split_separates_block_and_body() {
        let text = "---\na: 1\n---\ncorpo\nlinha\n";
        let Some((block, body)) = split(text) else {
            return;
        };
        assert_eq!(block, "a: 1");
        assert_eq!(body, "corpo\nlinha\n");
    }

    #[test]
    fn parse_reads_bare_and_quoted_values() {
        let text = "---\nid: Q1\ntitulo: \"com: dois pontos\"\n---\n";
        let Some((map, _)) = parse(text) else {
            return;
        };
        assert_eq!(map.get("id").map(String::as_str), Some("Q1"));
        assert_eq!(
            map.get("titulo").map(String::as_str),
            Some("com: dois pontos")
        );
    }

    #[test]
    fn render_round_trips_through_parse() {
        let text = render(&[("id", "Q2"), ("titulo", "a: b")], "corpo\n");
        let Some((map, body)) = parse(&text) else {
            return;
        };
        assert_eq!(map.get("titulo").map(String::as_str), Some("a: b"));
        assert_eq!(body, "corpo\n");
    }
}
