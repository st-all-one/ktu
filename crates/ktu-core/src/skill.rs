//! Skills declarativas: protocolos em `<dir>/<nome>/SKILL.md`.
//!
//! Uma skill é um **documento** (o `SKILL.md` de cada directório, com o frontmatter
//! do pi/knudge) que compõe o system prompt: é um *router* — diz **quando** usar o
//! protocolo e onde estão os detalhes, que o agente lê com `read_file` se precisar.
//!
//! Dois níveis (M8): `~/.config/ktu/skills/` (global) e `.agents/skills/` (local). O
//! **nome é o do directório** e o local **ganha** ao global com o mesmo nome — a
//! precedência é declarada, não emergente. O tecto de tamanho é o do `AGENTS.md`
//! (H10): uma skill acima do tecto é ignorada com aviso, nunca truncada.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::agents_md::{MAX_FILE_BYTES, MAX_TOTAL_BYTES};
use crate::frontmatter::unquote;
use crate::layout::AGENTS_DIR;
use crate::origin::Origin;
use crate::ports::Fs;

/// Nome do ficheiro de uma skill dentro do seu directório.
pub const SKILL_FILE: &str = "SKILL.md";

/// Directório das skills **globais** (por baixo do directório do utilizador).
pub const SKILLS_DIR: &str = "skills";

/// Uma skill carregada (nome estável + corpo que entra no prompt).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skill {
    name: String,
    description: String,
    body: String,
    path: PathBuf,
    origin: Origin,
}

impl Skill {
    /// Nome de invocação (o nome do directório; identidade da skill).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Descrição curta do frontmatter (pode ser vazia).
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Corpo do documento (sem frontmatter) — é o que compõe o prompt.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Ficheiro de origem (para o `ls` e para o agente o poder ler).
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Nível de onde a skill veio (M8).
    #[must_use]
    pub const fn origin(&self) -> Origin {
        self.origin
    }
}

/// Carrega as skills efectivas dos **dois** níveis (local ganha por nome).
///
/// Devolve as skills por ordem de nome e a lista de problemas encontrados
/// (directório sem `SKILL.md`, ficheiro acima do tecto, ficheiro ilegível), para a
/// borda os registar — fail-open: uma skill malformada nunca impede o arranque.
#[must_use]
pub fn load(fs: &dyn Fs, user_dir: Option<&Path>, root: &Path) -> (Vec<Skill>, Vec<String>) {
    let mut found: BTreeMap<String, Skill> = BTreeMap::new();
    let mut problems: Vec<String> = Vec::new();
    for (origin, dir) in [
        user_dir.map(|dir| (Origin::Global, dir.join(SKILLS_DIR))),
        Some((Origin::Local, root.join(AGENTS_DIR).join(SKILLS_DIR))),
    ]
    .into_iter()
    .flatten()
    {
        collect(fs, &dir, origin, &mut found, &mut problems);
    }
    let mut kept: Vec<Skill> = Vec::new();
    let mut total = 0usize;
    for skill in found.into_values() {
        if total.saturating_add(skill.body.len()) > MAX_TOTAL_BYTES {
            problems.push(format!(
                "{}: ignorada (acima do tecto total de {MAX_TOTAL_BYTES} bytes)",
                skill.name
            ));
            continue;
        }
        total = total.saturating_add(skill.body.len());
        kept.push(skill);
    }
    (kept, problems)
}

/// Secção do system prompt com as skills efectivas, ou `None` se não houver nenhuma.
///
/// Uma skill entra como `## <nome> — <descrição>` + o corpo: o modelo vê **quando**
/// usar cada protocolo e onde estão os detalhes (o corpo é um *router*).
#[must_use]
pub fn section(skills: &[Skill]) -> Option<String> {
    if skills.is_empty() {
        return None;
    }
    let parts: Vec<String> = skills.iter().map(render).collect();
    Some(format!("# Skills\n\n{}", parts.join("\n\n")))
}

/// Uma skill como bloco de prompt.
fn render(skill: &Skill) -> String {
    if skill.description.is_empty() {
        return format!("## {}\n\n{}", skill.name, skill.body);
    }
    format!(
        "## {} — {}\n\n{}",
        skill.name, skill.description, skill.body
    )
}

/// Acrescenta as skills de um nível ao mapa (o último a escrever ganha).
fn collect(
    fs: &dyn Fs,
    dir: &Path,
    origin: Origin,
    found: &mut BTreeMap<String, Skill>,
    problems: &mut Vec<String>,
) {
    if !fs.exists(dir) {
        return;
    }
    let mut entries = fs.read_dir(dir).unwrap_or_default();
    entries.sort();
    for path in entries {
        if !fs.is_dir(&path) {
            continue;
        }
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        let file = path.join(SKILL_FILE);
        if !fs.exists(&file) {
            problems.push(format!("{name}: sem {SKILL_FILE}"));
            continue;
        }
        let bytes = match fs.read(&file) {
            Ok(bytes) => bytes,
            Err(error) => {
                problems.push(format!("{name}: {error}"));
                continue;
            }
        };
        if bytes.len() > MAX_FILE_BYTES {
            problems.push(format!(
                "{name}: {SKILL_FILE} acima do tecto de {MAX_FILE_BYTES} bytes"
            ));
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            problems.push(format!("{name}: nao e UTF-8"));
            continue;
        };
        let (meta, body) = split_header(&text);
        found.insert(
            name.to_owned(),
            Skill {
                name: name.to_owned(),
                description: meta.get("description").cloned().unwrap_or_default(),
                body: body.trim().to_owned(),
                path: file,
                origin,
            },
        );
    }
}

/// Separa o cabeçalho `---`…`---` do corpo, **tolerante** ao YAML das skills.
///
/// O frontmatter dos documentos do ktu é escalar ([`crate::frontmatter::split`]);
/// as skills seguem o formato do pi/knudge, com valores dobrados (`>`/`|`) e
/// listas. Aqui junta-se a continuação indentada e **não** se valida nada: um
/// cabeçalho que não feche é tratado como corpo (fail-open, N2).
fn split_header(text: &str) -> (BTreeMap<String, String>, &str) {
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (BTreeMap::new(), text);
    };
    if first.trim() != "---" {
        return (BTreeMap::new(), text);
    }
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut offset = first.len();
    // Chave cujo valor continua nas linhas indentadas seguintes (`>`/`|`).
    let mut pending: Option<String> = None;
    for line in lines {
        offset = offset.saturating_add(line.len());
        let trimmed = line.trim();
        if trimmed == "---" {
            return (map, text.get(offset..).unwrap_or(""));
        }
        if trimmed.is_empty() {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            // Continuação do valor anterior: junta-se numa linha só.
            if let Some(key) = &pending
                && let Some(value) = map.get_mut(key)
            {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(trimmed);
            }
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim().to_owned();
        let value = value.trim();
        if value.is_empty() || value == ">" || value == "|" || value == ">-" {
            map.insert(key.clone(), String::new());
            pending = Some(key);
        } else {
            map.insert(key, unquote(value));
            pending = None;
        }
    }
    (BTreeMap::new(), text)
}

#[cfg(test)]
mod tests;
