//! Guardrails declarativos (§2.8, camada 2): regras em `.agents/guardrails/*.md`.
//!
//! Fail-closed (I16): um ficheiro inválido **impede o arranque**, nomeando o
//! ficheiro. Uma regra declarativa nunca desliga invariantes estruturais — só
//! pode **apertar** (`decision: deny`); regras que tentam permitir são ignoradas.

use std::ffi::OsStr;
use std::path::Path;

use crate::frontmatter::{self, required};
use crate::guard::{AgentId, Decision, DenyReason, Fact, Guard};
use crate::layout::AGENTS_DIR;
use crate::origin::Origin;
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

/// Veredicto de uma regra declarativa (só pode **apertar**, I16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleDecision {
    /// Nega o efeito.
    Deny,
    /// Requer aprovação humana (nunca é transformado em `Allow`).
    Escalate,
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
    /// Veredicto (`deny` ou `escalate`).
    pub decision: RuleDecision,
    /// Razão humana (o documento é a regra e a razão).
    pub reason: String,
    /// Remédio acionável próprio da regra (dado opcional).
    pub remedy: Option<String>,
}

impl Rule {
    /// `true` se a regra casa o facto dado.
    #[must_use]
    pub fn matches(&self, fact: &Fact) -> bool {
        if self
            .applies_to
            .as_ref()
            .is_some_and(|agent| *agent != fact.agent)
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
    let map = frontmatter::parse(text)
        .ok_or_else(|| format!("{file}: frontmatter ausente ou malformado"))?;
    let id = required(&map, file, "id")?;
    let version: u32 = required(&map, file, "version")?
        .parse()
        .map_err(|_| format!("{file}: `version` nao e um inteiro"))?;
    if version != SCHEMA_VERSION {
        return Err(format!("{file}: versao de schema nao suportada: {version}"));
    }
    let decision = match required(&map, file, "decision")?.as_str() {
        "deny" => RuleDecision::Deny,
        "escalate" => RuleDecision::Escalate,
        // Uma regra que tente permitir é ignorada, nunca aplicada (I16).
        _ => return Ok(None),
    };
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
    let remedy = map.get("remedy").cloned().filter(|value| !value.is_empty());
    Ok(Some(Rule {
        id,
        applies_to,
        fact,
        matcher,
        decision,
        reason,
        remedy,
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

/// Uma regra com o nível de onde veio (M8).
pub type Loaded = (Origin, Rule);

/// Carrega as regras dos **dois** níveis, com a origem de cada uma.
///
/// A ordem é a **efectiva**: global primeiro (piso aditivo, M5), o projecto depois;
/// dentro de cada nível, por `id`. Um ficheiro inválido é erro (I16), nomeando-o.
///
/// # Errors
/// Devolve mensagem legível se uma regra de qualquer dos níveis não carregar.
pub fn load_layered(
    fs: &dyn Fs,
    user_dir: Option<&Path>,
    root: &Path,
) -> Result<Vec<Loaded>, String> {
    let mut rules: Vec<Loaded> = Vec::new();
    if let Some(dir) = user_dir {
        rules.extend(
            load(fs, &dir.join("guardrails"))?
                .into_iter()
                .map(|rule| (Origin::Global, rule)),
        );
    }
    rules.extend(
        load(fs, &root.join(AGENTS_DIR).join("guardrails"))?
            .into_iter()
            .map(|rule| (Origin::Local, rule)),
    );
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
        match self.rules.iter().find(|rule| rule.matches(fact)) {
            Some(rule) => match rule.decision {
                RuleDecision::Deny => Decision::Deny(DenyReason::Guardrail),
                RuleDecision::Escalate => Decision::Escalate,
            },
            None => Decision::Allow,
        }
    }

    fn remedy(&self, fact: &Fact) -> Option<String> {
        self.rules
            .iter()
            .find(|rule| rule.matches(fact))
            .and_then(|rule| rule.remedy.clone())
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

    fn remedy(&self, fact: &Fact) -> Option<String> {
        match self.first.evaluate(fact) {
            Decision::Allow => self.second.remedy(fact),
            _ => self.first.remedy(fact),
        }
    }
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
