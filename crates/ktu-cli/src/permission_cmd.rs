//! `ktu permission` — lista e edita as permissões por tool.
//!
//! A política é **dado** em `.agents/permissions.toml` (project-bound): `Always`
//! executa, `Ask` escala (aprovação humana), `Never` nega. O ficheiro é lido no
//! arranque de cada turno e sobrevive a `resume`.

use std::path::Path;
use std::process::ExitCode;

use ktu_core::permission::{self, Permission, Store};
use ktu_tools::StdFs;

use crate::output::{self, Format};

/// Lista as permissões explícitas.
pub(crate) fn list(root: &Path, format: Format) -> ExitCode {
    let store = match load(root) {
        Ok(store) => store,
        Err(message) => {
            output::emit_error(format, "permission ls", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    let entries = store.entries();
    let text = if entries.is_empty() {
        "sem permissoes explicitas (tudo always)".to_owned()
    } else {
        entries
            .iter()
            .map(|(tool, permission)| format!("{tool}\t{}", permission.as_str()))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let data: Vec<serde_json::Value> = entries
        .iter()
        .map(|(tool, permission)| {
            serde_json::json!({ "tool": tool, "permission": permission.as_str() })
        })
        .collect();
    output::emit(
        format,
        "permission ls",
        &text,
        &serde_json::json!({ "permissions": data }),
    );
    ExitCode::SUCCESS
}

/// Define a permissão de uma tool (`always|ask|never`).
pub(crate) fn set(root: &Path, tool: &str, value: &str, format: Format) -> ExitCode {
    let Some(permission) = Permission::parse(value) else {
        output::emit_error(
            format,
            "permission set",
            "invalid",
            &format!("permissao desconhecida: {value} (use always|ask|never)"),
        );
        return ExitCode::FAILURE;
    };
    let mut store = match load(root) {
        Ok(store) => store,
        Err(message) => {
            output::emit_error(format, "permission set", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    store.set(tool, permission);
    save(root, &store, format, "permission set")
}

/// Remove a permissão de uma tool (volta ao *default*).
pub(crate) fn unset(root: &Path, tool: &str, format: Format) -> ExitCode {
    let mut store = match load(root) {
        Ok(store) => store,
        Err(message) => {
            output::emit_error(format, "permission unset", "invalid", &message);
            return ExitCode::FAILURE;
        }
    };
    store.unset(tool);
    save(root, &store, format, "permission unset")
}

/// Carrega o store do projecto.
fn load(root: &Path) -> Result<Store, String> {
    let fs = StdFs;
    permission::load(&fs, root)
}

/// Grava o store e emite o resultado.
fn save(root: &Path, store: &Store, format: Format, command: &str) -> ExitCode {
    let path = root.join(permission::FILE);
    if let Some(parent) = path.parent() {
        let _ignored = std::fs::create_dir_all(parent);
    }
    match std::fs::write(&path, store.render()) {
        Ok(()) => {
            output::emit(
                format,
                command,
                &format!("gravado {}", permission::FILE),
                &serde_json::json!({ "file": permission::FILE }),
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(format, command, "io", &error.to_string());
            ExitCode::FAILURE
        }
    }
}
