//! `ktu config doctor` — o schema de cada nível e se migraria (**sem escrever**).
//!
//! Diagnóstico de leitura: diz a versão declarada (`schema`) de cada ficheiro, se a
//! migração o mudaria, e o erro quando a versão é **futura** (fail-closed). Nunca
//! escreve — quem migra é a escrita (`ktu config set`/`unset`), com aviso em `stderr`.

use std::path::PathBuf;
use std::process::ExitCode;

use ktu_core::config;

use crate::config_cmd::{Scope, cwd, global_path, read_text};
use crate::output::{self, Format};

/// Uma linha de `config doctor` (um nível de ficheiro).
///
/// `schema: None` significa **ficheiro ausente** (e não v0) — a distinção importa
/// ao diagnóstico. Um `bool` só (`migrates`), pelo teto de booleanos em structs.
struct Level {
    /// `local` ou `global`.
    name: &'static str,
    /// Caminho do ficheiro.
    path: PathBuf,
    /// Versão do schema declarada (`None` ⇒ ausente; `Some(0)` ⇒ v0).
    schema: Option<u32>,
    /// A migração mudaria o ficheiro.
    migrates: bool,
    /// Erro de leitura (versão futura, TOML inválido).
    error: Option<String>,
}

impl Level {
    fn render(&self) -> String {
        if let Some(error) = &self.error {
            return format!("{}  {}  ERRO: {error}", self.name, self.path.display());
        }
        let Some(schema) = self.schema else {
            return format!("{}  {}  (ausente)", self.name, self.path.display());
        };
        format!(
            "{}  {}  schema={schema}  migraria={}",
            self.name,
            self.path.display(),
            if self.migrates { "sim" } else { "nao" }
        )
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "level": self.name,
            "path": self.path,
            "schema": self.schema,
            "migrates": self.migrates,
            "error": self.error,
        })
    }
}

/// Inspecciona um ficheiro de config **sem escrever**.
fn level(name: &'static str, path: Option<PathBuf>) -> Level {
    let Some(path) = path else {
        return Level {
            name,
            path: PathBuf::new(),
            schema: None,
            migrates: false,
            error: Some("sem HOME/XDG_CONFIG_HOME".to_owned()),
        };
    };
    let mut level = Level {
        name,
        path: path.clone(),
        schema: None,
        migrates: false,
        error: None,
    };
    if !path.is_file() {
        return level;
    }
    match read_text(&path).and_then(|text| config::table(&text)) {
        Ok(table) => {
            level.schema = Some(config::schema_of(&table));
            match config::migrate(table) {
                Ok((_, changed)) => level.migrates = changed,
                Err(message) => level.error = Some(message),
            }
        }
        Err(message) => level.error = Some(message),
    }
    level
}

/// `ktu config doctor` — o schema de cada nível e se migraria (read-only).
pub(crate) fn doctor(scope: Scope, format: Format) -> ExitCode {
    let root = cwd();
    let global = global_path();
    let levels: Vec<Level> = match scope {
        Scope::Local => vec![level("local", Some(config::local_config_path(&root)))],
        Scope::Global => vec![level("global", global)],
        Scope::Effective => vec![
            level("local", Some(config::local_config_path(&root))),
            level("global", global),
        ],
    };
    let text = levels
        .iter()
        .map(Level::render)
        .collect::<Vec<_>>()
        .join("\n");
    let data: Vec<serde_json::Value> = levels.iter().map(Level::json).collect();
    output::emit(
        format,
        "config doctor",
        &text,
        &serde_json::json!({ "scope": scope.label(), "levels": data }),
    );
    if levels.iter().any(|level| level.error.is_some()) {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
