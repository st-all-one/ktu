//! Guardrails declarativos (§2.8, camada 2): regras em `.agents/guardrails/*.md`.
//!
//! Fail-closed (I16): um ficheiro inválido **impede o arranque**, nomeando o
//! ficheiro. Uma regra declarativa nunca desliga invariantes estruturais — só
//! pode **apertar** (`decision: deny`); regras que tentam permitir são ignoradas.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;

use crate::guard::{AgentId, Decision, DenyReason, Fact, Guard};
use crate::ports::Fs;

/// Versão do schema de guardrail suportada.
pub const SCHEMA_VERSION: u32 = 1;

/// Sobre que facto uma regra decide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactKind {
    /// Nome da tool.
    Tool,
    /// *Path* pedido (glob simples com `*`).
    Path,
}

/// Uma regra declarativa carregada e validada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    /// Identificador estável da regra.
    pub id: String,
    /// Agente a que se aplica (`None` = qualquer agente).
    pub applies_to: Option<AgentId>,
    /// Facto sobre o qual decide.
    pub fact: FactKind,
    /// Valor a casar (nome de tool ou glob de path).
    pub matcher: String,
    /// Razão humana (o documento é a regra e a razão).
    pub reason: String,
}

impl Rule {
    /// `true` se a regra casa o facto dado.
    #[must_use]
    pub fn matches(&self, fact: &Fact) -> bool {
        if let Some(agent) = self.applies_to
            && agent != fact.agent
        {
            return false;
        }
        match self.fact {
            FactKind::Tool => fact.tool == self.matcher,
            FactKind::Path => fact
                .requested_path
                .as_deref()
                .is_some_and(|path| glob_match(&self.matcher, path)),
        }
    }
}

/// Interpreta uma regra do texto de um `.md`.
///
/// Devolve `Ok(None)` quando a regra é sintacticamente válida mas tenta
/// **permitir** (`decision != deny`): é ignorada, nunca aplicada (I16).
///
/// # Errors
/// Devolve mensagem (com o nome do ficheiro) se o frontmatter faltar, tiver
/// campos em falta, versão não suportada ou valores desconhecidos.
pub fn parse(file: &str, text: &str) -> Result<Option<Rule>, String> {
    let map =
        frontmatter(text).ok_or_else(|| format!("{file}: frontmatter ausente ou malformado"))?;
    let id = required(&map, file, "id")?;
    let version: u32 = required(&map, file, "version")?
        .parse()
        .map_err(|_| format!("{file}: `version` nao e um inteiro"))?;
    if version != SCHEMA_VERSION {
        return Err(format!("{file}: versao de schema nao suportada: {version}"));
    }
    if required(&map, file, "decision")? != "deny" {
        return Ok(None);
    }
    let applies_to = match map.get("applies_to").map(String::as_str) {
        None | Some("*") => None,
        Some(name) => Some(
            AgentId::from_name(name)
                .ok_or_else(|| format!("{file}: `applies_to` desconhecido: {name}"))?,
        ),
    };
    let fact = match required(&map, file, "fact")?.as_str() {
        "tool" => FactKind::Tool,
        "path" => FactKind::Path,
        other => return Err(format!("{file}: `fact` desconhecido: {other}")),
    };
    let matcher = required(&map, file, "match")?;
    let reason = required(&map, file, "reason")?;
    Ok(Some(Rule {
        id,
        applies_to,
        fact,
        matcher,
        reason,
    }))
}

/// Carrega todas as regras de `dir` (determinístico, fail-closed).
///
/// # Errors
/// Propaga o erro do primeiro ficheiro inválido (nomeando-o) ou de leitura.
pub fn load(fs: &dyn Fs, dir: &Path) -> Result<Vec<Rule>, String> {
    if !fs.exists(dir) {
        return Ok(Vec::new());
    }
    let mut entries = fs
        .read_dir(dir)
        .map_err(|error| format!("{}: {error}", dir.display()))?;
    entries.sort();
    let mut rules = Vec::new();
    for path in entries {
        if path.extension().and_then(OsStr::to_str) != Some("md") {
            continue;
        }
        let bytes = fs
            .read(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let text =
            String::from_utf8(bytes).map_err(|_| format!("{}: nao e UTF-8", path.display()))?;
        let name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("guardrail");
        if let Some(rule) = parse(name, &text)? {
            rules.push(rule);
        }
    }
    rules.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(rules)
}

/// Guardrail declarativo: nega se **alguma** regra casar.
pub struct DeclarativeGuard {
    rules: Vec<Rule>,
}

impl DeclarativeGuard {
    /// Cria o guard a partir das regras carregadas.
    #[must_use]
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    /// Número de regras carregadas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// `true` se não há regras.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Identificadores das regras, por ordem.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.rules.iter().map(|rule| rule.id.as_str()).collect()
    }
}

impl Guard for DeclarativeGuard {
    fn evaluate(&self, fact: &Fact) -> Decision {
        if self.rules.iter().any(|rule| rule.matches(fact)) {
            Decision::Deny(DenyReason::Guardrail)
        } else {
            Decision::Allow
        }
    }
}

/// Encadeia dois guardrails: o primeiro é avaliado **primeiro** (I11).
///
/// Só se o primeiro permitir é que o segundo é consultado — um `Deny` nunca é
/// transformado em `Allow` pelo elo seguinte.
pub struct ChainGuard<'a> {
    first: &'a dyn Guard,
    second: &'a dyn Guard,
}

impl<'a> ChainGuard<'a> {
    /// Cria a cadeia `first → second`.
    #[must_use]
    pub fn new(first: &'a dyn Guard, second: &'a dyn Guard) -> Self {
        Self { first, second }
    }
}

impl Guard for ChainGuard<'_> {
    fn evaluate(&self, fact: &Fact) -> Decision {
        match self.first.evaluate(fact) {
            Decision::Allow => self.second.evaluate(fact),
            deny => deny,
        }
    }
}

/// Extrai o frontmatter `---` de um documento estruturado.
fn frontmatter(text: &str) -> Option<BTreeMap<String, String>> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut map = BTreeMap::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            return Some(map);
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once(':')?;
        map.insert(key.trim().to_owned(), unquote(value.trim()));
    }
    None
}

/// Remove aspas envolventes de um valor escalar.
fn unquote(value: &str) -> String {
    let bytes = value.as_bytes();
    let first = bytes.first().copied();
    let last = bytes.last().copied();
    let quoted = matches!(
        (first, last),
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

/// Lê um campo obrigatório e não vazio.
fn required(map: &BTreeMap<String, String>, file: &str, key: &str) -> Result<String, String> {
    map.get(key)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| format!("{file}: campo `{key}` em falta"))
}

/// Casa um glob simples (apenas `*`) contra um texto.
#[must_use]
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let Some((head, tail)) = pattern.split_once('*') else {
        return pattern == text;
    };
    let Some(rest) = text.strip_prefix(head) else {
        return false;
    };
    rest.char_indices()
        .map(|(index, _)| rest.get(index..).unwrap_or(""))
        .chain(std::iter::once(""))
        .any(|candidate| glob_match(tail, candidate))
}

/// *Seed* do catálogo mínimo de guardrails (§2.8), gravado por `ktu init`.
pub const SEED: &[(&str, &str)] = &[
    (
        "deny-dotenv.md",
        "---\nid: deny-dotenv\nversion: 1\napplies_to: '*'\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n\n# Porquê\n\nImpede a leitura de ficheiros de segredos (`.env`, `.env.local`, ...).\n",
    ),
    (
        "deny-secret-files.md",
        "---\nid: deny-secret-files\nversion: 1\napplies_to: '*'\nfact: path\nmatch: '*.pem'\ndecision: deny\nreason: segredos\n---\n\n# Porquê\n\nChaves e certificados nunca entram no contexto do modelo.\n",
    ),
    (
        "deny-explorer-write.md",
        "---\nid: deny-explorer-write\nversion: 1\napplies_to: explorer\nfact: tool\nmatch: write_file\ndecision: deny\nreason: capability\n---\n\n# Porquê\n\nO `explorer` é read-only por construção; esta regra é defesa em profundidade.\n",
    ),
];

#[cfg(test)]
mod tests;
