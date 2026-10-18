//! Agentes **declarativos** do projecto (`.agents/agents/*.md`, §2.6).
//!
//! Um agente é **dado**: frontmatter com `name`, `mode` (`primary`/`subagent`) e
//! `tools` (política allow/deny por padrão). O conjunto de tools é o
//! *enforcement* estrutural — não é um pedido no prompt (I15/I16).
//!
//! Fail-closed: um ficheiro inválido **impede o arranque**, nomeando-o. Um agente
//! declarativo nunca alarga o que o guard estrutural permite: só o substitui pelo
//! conjunto que o projecto declarou (mínimo privilégio por omissão).

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::agent::{AgentRegistry, AgentSpec};
use crate::frontmatter::{self, required};
use crate::guard::AgentId;
use crate::guardrails::glob_match;
use crate::ports::Fs;

/// Decisão de acesso de uma tool num agente declarativo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolAccess {
    /// A tool é oferecida ao agente.
    Allow,
    /// A tool é negada ao agente.
    Deny,
}

/// Política de tools: um *default* e regras por padrão (`*` = qualquer).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolPolicy {
    default: ToolAccess,
    rules: BTreeMap<String, ToolAccess>,
}

impl ToolPolicy {
    /// Cria uma política com o *default* e as regras dadas.
    #[must_use]
    pub fn new(default: ToolAccess, rules: BTreeMap<String, ToolAccess>) -> Self {
        Self { default, rules }
    }

    /// Política que **nega tudo** excepto o que for listado (`read_file, glob`).
    #[must_use]
    pub fn allow_list(tools: impl IntoIterator<Item = String>) -> Self {
        let rules = tools
            .into_iter()
            .map(|tool| (tool, ToolAccess::Allow))
            .collect();
        Self::new(ToolAccess::Deny, rules)
    }

    /// Interpreta a sintaxe do frontmatter: `{"*": false, "read_*": true}` ou
    /// uma lista simples `read_file, glob` (tudo o resto negado).
    ///
    /// # Errors
    /// Devolve mensagem legível se a política for malformada.
    pub fn parse(spec: &str) -> Result<Self, String> {
        let trimmed = spec.trim();
        if !trimmed.starts_with('{') {
            let tools: Vec<String> = trimmed
                .split(',')
                .map(|item| frontmatter::unquote(item.trim()))
                .filter(|item| !item.is_empty())
                .collect();
            if tools.is_empty() {
                return Err("politica de tools vazia".to_owned());
            }
            return Ok(Self::allow_list(tools));
        }
        let inner = trimmed
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .ok_or_else(|| format!("politica de tools malformada: {spec}"))?;
        let mut default = ToolAccess::Deny;
        let mut rules = BTreeMap::new();
        for item in inner.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let (key, value) = item
                .split_once(':')
                .ok_or_else(|| format!("entrada malformada na politica: {item}"))?;
            let key = frontmatter::unquote(key.trim());
            let access = match value.trim() {
                "true" => ToolAccess::Allow,
                "false" => ToolAccess::Deny,
                other => return Err(format!("valor invalido na politica: {other}")),
            };
            if key.is_empty() {
                return Err("padrao vazio na politica".to_owned());
            }
            if key == "*" {
                default = access;
            } else {
                let _replaced = rules.insert(key, access);
            }
        }
        Ok(Self::new(default, rules))
    }

    /// Decide se a tool é oferecida: padrão exacto ganha; depois o padrão com
    /// `*` mais longo que casar; por fim o *default*.
    #[must_use]
    pub fn allows(&self, tool: &str) -> bool {
        if let Some(access) = self.rules.get(tool) {
            return *access == ToolAccess::Allow;
        }
        let mut best: Option<(&str, ToolAccess)> = None;
        for (pattern, access) in &self.rules {
            if !pattern.contains('*') || !glob_match(pattern, tool) {
                continue;
            }
            let better = best.is_none_or(|(current, _)| pattern.len() > current.len());
            if better {
                best = Some((pattern, *access));
            }
        }
        match best {
            Some((_, access)) => access == ToolAccess::Allow,
            None => self.default == ToolAccess::Allow,
        }
    }

    /// Regras **exactas** (sem `*`) que não existem no catálogo — erro de escrita.
    #[must_use]
    pub fn unknown(&self, catalog: &[String]) -> Vec<String> {
        self.rules
            .keys()
            .filter(|pattern| {
                !pattern.contains('*') && !catalog.iter().any(|name| name == *pattern)
            })
            .cloned()
            .collect()
    }

    /// Tools do catálogo permitidas por esta política, por ordem do catálogo.
    #[must_use]
    pub fn resolve(&self, catalog: &[String]) -> Vec<String> {
        catalog
            .iter()
            .filter(|name| self.allows(name))
            .cloned()
            .collect()
    }
}

/// Papel de um agente declarativo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentMode {
    /// Substitui o agente primário da sessão (um só, por projecto).
    Primary,
    /// Subagente registado por nome (usável pela tool `subagent`).
    Subagent,
}

impl AgentMode {
    /// Nome estável do modo (contrato de log e de `ktu agents ls`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Subagent => "subagent",
        }
    }
}

/// Um agente declarativo carregado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentFile {
    name: String,
    mode: AgentMode,
    policy: ToolPolicy,
    path: PathBuf,
}

impl AgentFile {
    /// Nome do agente.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Papel do agente.
    #[must_use]
    pub const fn mode(&self) -> AgentMode {
        self.mode
    }

    /// Ficheiro de origem.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Tools efectivas deste agente, dado o catálogo disponível.
    #[must_use]
    pub fn tools(&self, catalog: &[String]) -> Vec<String> {
        self.policy.resolve(catalog)
    }
}

/// Interpreta um agente a partir do texto de um `.md`.
///
/// # Errors
/// Devolve mensagem legível (com o ficheiro) se o frontmatter faltar, se o nome
/// for reservado/inválido, se o modo for desconhecido ou se a política for má.
pub fn parse(file: &str, text: &str) -> Result<AgentFile, String> {
    let map = frontmatter::parse(text)
        .ok_or_else(|| format!("{file}: frontmatter ausente ou malformado"))?;
    let name = required(&map, file, "name")?;
    if name.contains(char::is_whitespace) || name.contains('/') {
        return Err(format!("{file}: `name` invalido: {name}"));
    }
    if AgentId::from_name(&name).is_some() {
        return Err(format!("{file}: `name` reservado: {name}"));
    }
    let mode = match map.get("mode").map_or("subagent", String::as_str) {
        "primary" => AgentMode::Primary,
        "subagent" => AgentMode::Subagent,
        other => return Err(format!("{file}: `mode` desconhecido: {other}")),
    };
    let policy = ToolPolicy::parse(&required(&map, file, "tools")?)
        .map_err(|error| format!("{file}: {error}"))?;
    Ok(AgentFile {
        name,
        mode,
        policy,
        path: PathBuf::from(file),
    })
}

/// Carrega os agentes de `dir` (determinístico, fail-closed).
///
/// # Errors
/// Propaga o erro do primeiro ficheiro inválido (nomeando-o) ou de leitura.
pub fn load(fs: &dyn Fs, dir: &Path) -> Result<Vec<AgentFile>, String> {
    if !fs.exists(dir) {
        return Ok(Vec::new());
    }
    let mut entries = fs
        .read_dir(dir)
        .map_err(|error| format!("{}: {error}", dir.display()))?;
    entries.sort();
    let mut agents: Vec<AgentFile> = Vec::new();
    for path in entries {
        if path.extension().and_then(OsStr::to_str) != Some("md") {
            continue;
        }
        let display = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("agente")
            .to_owned();
        let bytes = fs
            .read(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let text =
            String::from_utf8(bytes).map_err(|_| format!("{}: nao e UTF-8", path.display()))?;
        let mut agent = parse(&display, &text)?;
        if agents.iter().any(|other| other.name == agent.name) {
            return Err(format!("{display}: agente duplicado: {}", agent.name));
        }
        agent.path = path;
        agents.push(agent);
    }
    agents.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(agents)
}

/// Aplica os agentes declarativos a um registo (fail-closed).
///
/// `catalog` é o conjunto de tools disponíveis na sessão (para expandir padrões).
///
/// # Errors
/// Devolve mensagem legível se houver mais de um agente `primary`, tools
/// desconhecidas ou um agente sem tools.
pub fn apply(
    files: &[AgentFile],
    registry: AgentRegistry,
    catalog: &[String],
) -> Result<AgentRegistry, String> {
    let mut primary: Option<AgentSpec> = None;
    let mut named: BTreeMap<String, AgentSpec> = BTreeMap::new();
    for file in files {
        let unknown = file.policy.unknown(catalog);
        if !unknown.is_empty() {
            return Err(format!(
                "{}: tools desconhecidas: {}",
                file.name,
                unknown.join(", ")
            ));
        }
        let tools = file.policy.resolve(catalog);
        if tools.is_empty() {
            return Err(format!("{}: agente sem tools", file.name));
        }
        match file.mode {
            AgentMode::Primary => {
                if primary.is_some() {
                    return Err(format!(
                        "{}: mais de um agente `primary` declarado",
                        file.name
                    ));
                }
                primary = Some(AgentSpec::new(AgentId::Primary, tools, 1));
            }
            AgentMode::Subagent => {
                let _replaced = named.insert(
                    file.name.clone(),
                    AgentSpec::new(AgentId::named(&file.name), tools, 0),
                );
            }
        }
    }
    let registry = registry.with_named(named);
    match primary {
        Some(spec) => Ok(registry.with_primary(spec)),
        None => Ok(registry),
    }
}

#[cfg(test)]
mod tests;
