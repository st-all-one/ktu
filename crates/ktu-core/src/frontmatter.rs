//! Frontmatter mínimo (`---` … `---`) dos documentos declarativos do projecto.
//!
//! É o mesmo formato dos guardrails (`.agents/guardrails/*.md`), dos comandos
//! (`.agents/commands/*.md`) e dos agentes (`.agents/agents/*.md`): chaves
//! escalares `chave: valor`, sem listas nem aninhamento. Um parser só, para que
//! todos os documentos do projecto se leiam da mesma maneira.

use std::collections::BTreeMap;

/// Frontmatter de um documento declarativo (chave → valor escalar).
pub type Frontmatter = BTreeMap<String, String>;

/// Divide um documento em frontmatter e corpo.
///
/// Devolve `None` quando o documento não começa por `---`, quando não fecha o
/// bloco ou quando uma linha do bloco não é `chave: valor`.
#[must_use]
pub fn split(text: &str) -> Option<(Frontmatter, &str)> {
    let mut offset = 0usize;
    let mut started = false;
    let mut map = BTreeMap::new();
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if !started {
            if trimmed != "---" {
                return None;
            }
            started = true;
            offset = offset.saturating_add(line.len());
            continue;
        }
        if trimmed == "---" {
            let body = text.get(offset.saturating_add(line.len())..).unwrap_or("");
            return Some((map, body));
        }
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let (key, value) = trimmed.split_once(':')?;
            map.insert(key.trim().to_owned(), unquote(value.trim()));
        }
        offset = offset.saturating_add(line.len());
    }
    None
}

/// Só o frontmatter, sem o corpo.
#[must_use]
pub fn parse(text: &str) -> Option<Frontmatter> {
    split(text).map(|(map, _)| map)
}

/// Corpo do documento (tudo o que vem depois do frontmatter, ou o texto todo
/// quando não há frontmatter).
#[must_use]
pub fn body(text: &str) -> &str {
    match split(text) {
        Some((_, body)) => body,
        None => text,
    }
}

/// Remove aspas envolventes de um valor escalar.
#[must_use]
pub fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    let quoted = matches!(
        (bytes.first().copied(), bytes.last().copied()),
        (Some(b'"'), Some(b'"')) | (Some(b'\''), Some(b'\''))
    );
    if quoted && bytes.len() >= 2 {
        return value
            .get(1..value.len().saturating_sub(1))
            .unwrap_or("")
            .to_owned();
    }
    value.to_owned()
}

/// Lê um campo obrigatório e não vazio, nomeando o ficheiro no erro.
///
/// # Errors
/// Devolve mensagem legível quando o campo falta ou está vazio.
pub fn required(map: &Frontmatter, file: &str, key: &str) -> Result<String, String> {
    map.get(key)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| format!("{file}: campo `{key}` em falta"))
}

#[cfg(test)]
mod tests {
    use super::{body, parse, required, split, unquote};

    #[test]
    fn splits_frontmatter_and_body() {
        let text = "---\nname: x\ndescription: 'com aspas'\n---\n\ncorpo\n";
        let (map, rest) = split(text).unwrap_or_default();
        assert_eq!(map.get("name").map(String::as_str), Some("x"));
        assert_eq!(
            map.get("description").map(String::as_str),
            Some("com aspas")
        );
        assert_eq!(rest.trim(), "corpo");
    }

    #[test]
    fn rejects_malformed_documents() {
        assert_eq!(parse("sem frontmatter\n"), None);
        assert_eq!(parse("---\nname: x\n"), None, "sem fecho");
        assert_eq!(parse("---\nlinha sem dois pontos\n---\n"), None);
    }

    #[test]
    fn body_falls_back_to_the_whole_text() {
        assert_eq!(body("corpo apenas"), "corpo apenas");
        assert_eq!(body("---\na: 1\n---\ncorpo"), "corpo");
        assert_eq!(body("---\na: 1\n---\n\ncorpo").trim(), "corpo");
    }

    #[test]
    fn unquotes_only_matching_quotes() {
        assert_eq!(unquote("'x'"), "x");
        assert_eq!(unquote("\"x\""), "x");
        assert_eq!(unquote("x"), "x");
        assert_eq!(unquote("'x"), "'x");
    }

    #[test]
    fn required_names_the_file_and_the_key() {
        let map = parse("---\nid: a\nvazio:\n---\n").unwrap_or_default();
        assert_eq!(required(&map, "f.md", "id").ok().as_deref(), Some("a"));
        assert_eq!(
            required(&map, "f.md", "falta").err().as_deref(),
            Some("f.md: campo `falta` em falta")
        );
        assert!(required(&map, "f.md", "vazio").is_err());
    }
}
