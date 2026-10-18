//! `ktu config` — inspecciona e edita a configuração (global + local).
//!
//! Precedência: `flag > --params > local > global > default` (M6: o projecto ganha
//! chave a chave). As credenciais vivem **só** no global (`auth.toml`, 0600) e
//! nunca são listadas aqui.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ktu_core::config::{self, Auth, Config};

use crate::cli::ConfigAction;
use crate::output::{self, Format};

/// Âmbito de uma operação de config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// Configuração efectiva (local sobre global).
    Effective,
    /// Config local do projecto (`.agents/config.toml`).
    Local,
    /// Config global do utilizador (`~/.config/ktu/config.toml`).
    Global,
}

impl Scope {
    const fn label(self) -> &'static str {
        match self {
            Self::Effective => "effective",
            Self::Local => "local",
            Self::Global => "global",
        }
    }
}

/// Uma linha de `config list`.
struct Row {
    key: &'static str,
    value: String,
    source: &'static str,
}

impl Row {
    fn render(&self) -> String {
        format!("{} = {}  # {}", self.key, self.value, self.source)
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({ "key": self.key, "value": self.value, "source": self.source })
    }
}

/// Configuração efectiva + credenciais (para o `run`/`serve`).
pub(crate) struct Settings {
    /// Config de comportamento (local sobre global).
    pub(crate) config: Config,
    /// Credenciais do utilizador (só global).
    pub(crate) auth: Auth,
}

/// Carrega a config efectiva e as credenciais para `root`.
///
/// # Errors
/// Devolve mensagem legível se um ficheiro existir mas for inválido.
pub(crate) fn load(root: &Path) -> Result<Settings, String> {
    let (local, global) = load_layers(root)?;
    let auth = match global_auth_path() {
        Some(path) => read_auth(&path)?,
        None => Auth::default(),
    };
    Ok(Settings {
        config: Config::merge(local, global),
        auth,
    })
}

/// Executa uma acção de `ktu config`.
pub(crate) fn run(scope: Scope, action: ConfigAction, format: Format) -> ExitCode {
    match action {
        ConfigAction::List => list(scope, format),
        ConfigAction::Get { key } => get(scope, &key, format),
        ConfigAction::Set { key, value } => set(scope, &key, &value, format),
        ConfigAction::Unset { key } => unset(scope, &key, format),
    }
}

fn list(scope: Scope, format: Format) -> ExitCode {
    let root = cwd();
    match entries(scope, &root) {
        Ok(rows) => {
            let text = if rows.is_empty() {
                "(config vazia)".to_owned()
            } else {
                rows.iter().map(Row::render).collect::<Vec<_>>().join("\n")
            };
            let data: Vec<serde_json::Value> = rows.iter().map(Row::json).collect();
            output::emit(
                format,
                "config list",
                &text,
                &serde_json::json!({ "scope": scope.label(), "entries": data }),
            );
            ExitCode::SUCCESS
        }
        Err(message) => fail(format, "config list", &message),
    }
}

fn get(scope: Scope, key: &str, format: Format) -> ExitCode {
    let root = cwd();
    let value = match scope {
        Scope::Effective => match load_layers(&root) {
            Ok((local, global)) => Config::merge(local, global).get(key),
            Err(message) => return fail(format, "config get", &message),
        },
        Scope::Local => match read_config(&config::local_config_path(&root)) {
            Ok(config) => config.get(key),
            Err(message) => return fail(format, "config get", &message),
        },
        Scope::Global => match scope_path(scope, &root).and_then(|path| read_config(&path)) {
            Ok(config) => config.get(key),
            Err(message) => return fail(format, "config get", &message),
        },
    };
    match value {
        Some(value) => {
            output::emit(
                format,
                "config get",
                &value,
                &serde_json::json!({ "key": key, "value": value, "scope": scope.label() }),
            );
            ExitCode::SUCCESS
        }
        None => fail(format, "config get", &format!("chave sem valor: {key}")),
    }
}

fn set(scope: Scope, key: &str, value: &str, format: Format) -> ExitCode {
    let root = cwd();
    let path = match editable_path(scope, &root) {
        Ok(path) => path,
        Err(message) => return fail(format, "config set", &message),
    };
    let mut config = match read_config(&path) {
        Ok(config) => config,
        Err(message) => return fail(format, "config set", &message),
    };
    if let Err(message) = config.set(key, value) {
        return fail(format, "config set", &message);
    }
    if let Err(message) = write_config(&path, &config) {
        return fail(format, "config set", &message);
    }
    output::emit(
        format,
        "config set",
        &format!("{key} = {value}  ({})", path.display()),
        &serde_json::json!({ "key": key, "value": value, "path": path }),
    );
    ExitCode::SUCCESS
}

fn unset(scope: Scope, key: &str, format: Format) -> ExitCode {
    let root = cwd();
    let path = match editable_path(scope, &root) {
        Ok(path) => path,
        Err(message) => return fail(format, "config unset", &message),
    };
    let mut config = match read_config(&path) {
        Ok(config) => config,
        Err(message) => return fail(format, "config unset", &message),
    };
    if let Err(message) = config.unset(key) {
        return fail(format, "config unset", &message);
    }
    if let Err(message) = write_config(&path, &config) {
        return fail(format, "config unset", &message);
    }
    output::emit(
        format,
        "config unset",
        &format!("removido {key}  ({})", path.display()),
        &serde_json::json!({ "key": key, "path": path }),
    );
    ExitCode::SUCCESS
}

/// Linhas de `list` para o âmbito pedido.
fn entries(scope: Scope, root: &Path) -> Result<Vec<Row>, String> {
    match scope {
        Scope::Local => Ok(rows_for(
            &read_config(&config::local_config_path(root))?,
            "local",
        )),
        Scope::Global => Ok(rows_for(&read_config(&scope_path(scope, root)?)?, "global")),
        Scope::Effective => {
            let (local, global) = load_layers(root)?;
            let effective = Config::merge(local.clone(), global);
            Ok(effective
                .entries()
                .into_iter()
                .map(|(key, value)| {
                    let source = if local.get(key).is_some() {
                        "local"
                    } else {
                        "global"
                    };
                    Row { key, value, source }
                })
                .collect())
        }
    }
}

/// Linhas de um nível concreto.
fn rows_for(config: &Config, source: &'static str) -> Vec<Row> {
    config
        .entries()
        .into_iter()
        .map(|(key, value)| Row { key, value, source })
        .collect()
}

/// Lê os dois níveis (local + global); ausentes ⇒ default.
fn load_layers(root: &Path) -> Result<(Config, Config), String> {
    let local = read_config(&config::local_config_path(root))?;
    let global = match global_path() {
        Some(path) => read_config(&path)?,
        None => Config::default(),
    };
    Ok((local, global))
}

/// Caminho do ficheiro de um âmbito concreto (`Effective` não tem ficheiro).
fn scope_path(scope: Scope, root: &Path) -> Result<PathBuf, String> {
    match scope {
        Scope::Local => Ok(config::local_config_path(root)),
        Scope::Global => {
            global_path().ok_or_else(|| "sem HOME/XDG_CONFIG_HOME para a config global".to_owned())
        }
        Scope::Effective => Err("o âmbito efectivo não tem ficheiro".to_owned()),
    }
}

/// Caminho editável (`--local`/`--global`); o efectivo é só de leitura.
fn editable_path(scope: Scope, root: &Path) -> Result<PathBuf, String> {
    match scope {
        Scope::Effective => Err("use --local ou --global para editar".to_owned()),
        _ => scope_path(scope, root),
    }
}

/// Lê o `config.toml` em `path` (ausente ⇒ default).
fn read_config(path: &Path) -> Result<Config, String> {
    if !path.is_file() {
        return Ok(Config::default());
    }
    Config::parse(&read_text(path)?)
}

/// Escreve o config em `path` (cria os directórios).
fn write_config(path: &Path, config: &Config) -> Result<(), String> {
    let text = config.render()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

/// Lê um ficheiro como texto.
fn read_text(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(String::from_utf8_lossy_owned(bytes))
}

/// Valor de uma variável de ambiente (só caminhos — nunca segredos).
fn env_str(key: &str) -> Option<String> {
    env::var_os(key).and_then(|value| value.into_string().ok())
}

/// Caminho do `config.toml` global, se houver `HOME`/`XDG_CONFIG_HOME`.
fn global_path() -> Option<PathBuf> {
    config::global_config_path(
        env_str("XDG_CONFIG_HOME").as_deref(),
        env_str("HOME").as_deref(),
    )
}

/// Directório de configuração do **utilizador** (`~/.config/ktu`), onde se
/// procuram `SYSTEM.md`/`APPEND_SYSTEM.md` (directriz do utilizador).
pub(crate) fn global_dir_path() -> Option<PathBuf> {
    config::global_dir(
        env_str("XDG_CONFIG_HOME").as_deref(),
        env_str("HOME").as_deref(),
    )
}

/// Caminho do `auth.toml` global.
fn global_auth_path() -> Option<PathBuf> {
    config::global_auth_path(
        env_str("XDG_CONFIG_HOME").as_deref(),
        env_str("HOME").as_deref(),
    )
}

/// Lê o `auth.toml` em `path` (ausente ⇒ vazio).
fn read_auth(path: &Path) -> Result<Auth, String> {
    if !path.is_file() {
        return Ok(Auth::default());
    }
    Auth::parse(&read_text(path)?)
}

/// Directório actual (vazio se indisponível).
fn cwd() -> PathBuf {
    env::current_dir().unwrap_or_default()
}

/// Reporta um erro de config e devolve falha.
fn fail(format: Format, command: &str, message: &str) -> ExitCode {
    output::emit_error(format, command, "invalid", message);
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;

    use ktu_core::config::Config;

    use super::{Scope, read_config, write_config};

    #[test]
    fn scope_labels_are_stable() {
        assert_eq!(Scope::Effective.label(), "effective");
        assert_eq!(Scope::Local.label(), "local");
        assert_eq!(Scope::Global.label(), "global");
    }

    #[test]
    fn config_round_trips_through_a_file() {
        let path = env::temp_dir().join(format!("ktu-config-{}.toml", process::id()));
        let _ignored = fs::remove_file(&path);

        let mut config = Config::default();
        assert!(config.set("provider", "mock").is_ok());
        assert!(write_config(&path, &config).is_ok());
        let read = read_config(&path).unwrap_or_default();
        assert_eq!(read.get("provider").as_deref(), Some("mock"));

        // Ausente ⇒ default.
        let _ignored = fs::remove_file(&path);
        assert_eq!(read_config(&path).unwrap_or_default(), Config::default());
    }

    #[test]
    fn missing_file_is_default() {
        let path = PathBuf::from("/nao/existe/ktu-config.toml");
        assert_eq!(read_config(&path).unwrap_or_default(), Config::default());
    }
}
