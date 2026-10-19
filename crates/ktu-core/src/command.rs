//! Comandos do projecto: templates Markdown em `.agents/commands/*.md`.
//!
//! Um comando é **texto** (frontmatter + prompt), invocável por `/nome args` na
//! sessão interactiva. Não tem lógica própria: renderiza o prompt e entrega-o ao
//! turno normal — todos os efeitos continuam a passar pelos guardrails (a tool
//! `bash`, se for usada, é avaliada como qualquer outra).
//!
//! O carregamento é **fail-open**: um ficheiro malformado é ignorado (com
//! mensagem) e nunca impede a sessão.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::frontmatter::{self, Frontmatter};
use crate::layout::{COMMANDS_DIR, RECIPES_DIR};
use crate::origin::Origin;
use crate::ports::Fs;

/// Marcador substituído pelos argumentos da linha de comando.
pub const ARGUMENTS: &str = "$ARGUMENTS";

/// Marcador de **shell embutido** no início de uma linha (`!comando`).
pub const SHELL_PREFIX: char = '!';

/// Acesso ao *shell* embutido de um comando (nunca implícito).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellAccess {
    /// Sem *shell*: as linhas `!…` são texto como as outras (por omissão).
    #[default]
    Disabled,
    /// O comando declara `shell: true`: as linhas `!…` são expandidas **depois**
    /// de passarem por `Guard::evaluate` (nunca antes).
    Guarded,
}

/// Configuração que o comando pode impor ao turno (`settings` do frontmatter).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandSettings {
    /// Provider a usar (`llama`|`opencode-go`|`mock`).
    pub provider: Option<String>,
    /// *Base URL* do provider.
    pub base_url: Option<String>,
    /// Modelo a usar.
    pub model: Option<String>,
    /// Tecto de passos do turno.
    pub max_steps: Option<u32>,
}

impl CommandSettings {
    /// `true` se o comando impõe alguma configuração.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.provider.is_none()
            && self.base_url.is_none()
            && self.model.is_none()
            && self.max_steps.is_none()
    }

    /// Lê as chaves de `settings` do frontmatter (fail-closed em valores inválidos).
    ///
    /// # Errors
    /// Devolve mensagem legível se `max_steps` não for um inteiro.
    pub fn parse(map: &Frontmatter, file: &str) -> Result<Self, String> {
        let max_steps = match map.get("max_steps") {
            Some(value) => Some(
                value
                    .parse()
                    .map_err(|_| format!("{file}: `max_steps` nao e um inteiro: {value}"))?,
            ),
            None => None,
        };
        Ok(Self {
            provider: map.get("provider").cloned(),
            base_url: map.get("base_url").cloned(),
            model: map.get("model").cloned(),
            max_steps,
        })
    }
}

/// Um comando do projecto (nome estável + prompt).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    name: String,
    description: String,
    prompt: String,
    settings: CommandSettings,
    shell: ShellAccess,
    path: PathBuf,
}

impl Command {
    /// Nome de invocação (sem `/`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Descrição curta (para `/help` e `ktu commands ls`); pode ser vazia.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Prompt declarado no ficheiro.
    #[must_use]
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Configuração imposta pelo comando (vazia se não declarar nenhuma).
    #[must_use]
    pub fn settings(&self) -> &CommandSettings {
        &self.settings
    }

    /// Acesso ao *shell* embutido.
    #[must_use]
    pub const fn shell(&self) -> ShellAccess {
        self.shell
    }

    /// Linhas de **shell embutido** (`!comando`), na ordem em que aparecem.
    ///
    /// Vazio quando o comando não declara `shell: true` — sem declaração, `!…` é
    /// texto normal (nunca executa nada por surpresa).
    #[must_use]
    pub fn shell_segments(&self) -> Vec<String> {
        if self.shell == ShellAccess::Disabled {
            return Vec::new();
        }
        self.prompt
            .lines()
            .filter_map(|line| line.trim().strip_prefix(SHELL_PREFIX))
            .map(|rest| rest.trim().to_owned())
            .filter(|rest| !rest.is_empty() && !rest.starts_with('['))
            .collect()
    }

    /// Ficheiro de origem.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Renderiza o prompt com os argumentos.
    ///
    /// Substitui [`ARGUMENTS`] (todos os argumentos) e `$1`…`$9` (posicionais,
    /// separados por espaços, com aspas simples ou duplas); sem marcador nenhum,
    /// acrescenta os argumentos no fim.
    #[must_use]
    pub fn render(&self, args: &str) -> String {
        let args = args.trim();
        let words = split_arguments(args);
        let mut text = self.prompt.clone();
        let mut substituted = false;
        for (index, word) in words.iter().enumerate() {
            let marker = format!("${}", index.saturating_add(1));
            if text.contains(&marker) {
                text = text.replace(&marker, word);
                substituted = true;
            }
        }
        if text.contains(ARGUMENTS) {
            text = text.replace(ARGUMENTS, args);
            substituted = true;
        }
        if substituted || args.is_empty() {
            return text;
        }
        format!("{text}\n\n{args}")
    }
}

/// Interpreta um comando a partir do texto de um `.md`.
///
/// O frontmatter é opcional: sem ele, o documento inteiro é o prompt e o nome
/// vem do ficheiro. `Ok(None)` quando não sobra prompt.
///
/// # Errors
/// Devolve mensagem legível se o nome declarado for inválido.
pub fn parse(file: &str, text: &str) -> Result<Option<Command>, String> {
    let map = frontmatter::parse(text).unwrap_or_default();
    let stem = file.strip_suffix(".md").unwrap_or(file);
    let name = map
        .get("name")
        .map_or(stem, String::as_str)
        .trim()
        .to_owned();
    if name.is_empty() || name.contains(char::is_whitespace) || name.contains('/') {
        return Err(format!("{file}: `name` invalido: {name}"));
    }
    let prompt = frontmatter::body(text).trim().to_owned();
    if prompt.is_empty() {
        return Ok(None);
    }
    let settings = CommandSettings::parse(&map, file)?;
    let shell = match map.get("shell").map_or("false", String::as_str) {
        "true" => ShellAccess::Guarded,
        "false" => ShellAccess::Disabled,
        other => return Err(format!("{file}: `shell` invalido: {other}")),
    };
    Ok(Some(Command {
        name,
        description: map.get("description").cloned().unwrap_or_default(),
        prompt,
        settings,
        shell,
        path: PathBuf::from(file),
    }))
}

/// Carrega os comandos de `dir` (determinístico, fail-open).
///
/// Devolve os comandos ordenados por nome e a lista de problemas encontrados
/// (ficheiros malformados, nomes duplicados), para a borda os registar.
#[must_use]
pub fn load(fs: &dyn Fs, dir: &Path) -> (Vec<Command>, Vec<String>) {
    let mut commands: Vec<Command> = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    let mut entries = fs.read_dir(dir).unwrap_or_default();
    entries.sort();
    for path in entries {
        if path.extension().and_then(OsStr::to_str) != Some("md") {
            continue;
        }
        let display = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("comando")
            .to_owned();
        let Some(text) = fs
            .read(&path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            problems.push(format!("{display}: nao e texto UTF-8"));
            continue;
        };
        match parse(&display, &text) {
            Ok(Some(mut command)) => {
                command.path = path;
                if commands.iter().any(|other| other.name == command.name) {
                    problems.push(format!("{display}: comando duplicado: {}", command.name));
                    continue;
                }
                commands.push(command);
            }
            Ok(None) => problems.push(format!("{display}: sem prompt")),
            Err(error) => problems.push(error),
        }
    }
    commands.sort_by(|left, right| left.name.cmp(&right.name));
    (commands, problems)
}

/// Uma receita com o nível de onde veio (M8).
pub type Loaded = (Origin, Command);

/// Carrega as receitas dos **dois** níveis, com a origem de cada uma.
///
/// O **local ganha por nome**: uma receita do projecto com o mesmo nome substitui a
/// global (precedência declarada, M8). Devolve os pares ordenados por nome e a lista
/// de problemas dos dois níveis (fail-open, como [`load`]).
#[must_use]
pub fn load_layered(
    fs: &dyn Fs,
    user_dir: Option<&Path>,
    root: &Path,
) -> (Vec<Loaded>, Vec<String>) {
    let mut found: BTreeMap<String, Loaded> = BTreeMap::new();
    let mut problems: Vec<String> = Vec::new();
    for (origin, dir) in [
        user_dir.map(|dir| (Origin::Global, dir.join(RECIPES_DIR))),
        Some((Origin::Local, root.join(COMMANDS_DIR))),
    ]
    .into_iter()
    .flatten()
    {
        let (commands, mut issues) = load(fs, &dir);
        problems.append(&mut issues);
        for command in commands {
            found.insert(command.name.clone(), (origin, command));
        }
    }
    (found.into_values().collect(), problems)
}

/// Separa os argumentos de um comando em palavras (aspas simples ou duplas).
fn split_arguments(args: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in args.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

/// Procura um comando pelo nome exacto.
#[must_use]
pub fn find<'a>(commands: &'a [Command], name: &str) -> Option<&'a Command> {
    commands.iter().find(|command| command.name == name)
}

#[cfg(test)]
mod tests;
