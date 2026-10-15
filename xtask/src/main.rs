//! `xtask` — tarefas de verificação do workspace ktu (`cargo xtask <alvo>`).
//!
//! Alvos:
//! - `check-layers`: impõe o firewall de dependências entre camadas (§2.1 do `plan.md`).
//! - `quality`: corre a bateria de qualidade `p1`–`p4` com o ktu isolado (F5/G4).
#![forbid(unsafe_code)]

mod quality;

use std::collections::BTreeSet;
use std::env;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

use serde_json::Value;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("check-layers") => check_layers(),
        Some("quality") => quality::run(args.get(1..).unwrap_or(&[])),
        Some(other) => Err(format!(
            "alvo desconhecido: `{other}` (disponíveis: check-layers, quality)"
        )),
        None => Err("alvo ausente (disponíveis: check-layers, quality)".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let mut stderr = io::stderr().lock();
            let _ignored = writeln!(stderr, "xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Firewall de camadas (§2.1 do `plan.md`): para cada crate interno, os crates
/// internos que **não** pode importar como dependência normal.
fn forbidden_deps(crate_name: &str) -> Option<&'static [&'static str]> {
    match crate_name {
        "ktu-core" => Some(&[
            "ktu-tools",
            "ktu-provider",
            "ktu-port",
            "ktu-memory",
            "ktu-cli",
        ]),
        "ktu-tools" => Some(&["ktu-provider", "ktu-port"]),
        "ktu-memory" => Some(&["ktu-tools", "ktu-provider", "ktu-port", "ktu-cli"]),
        "ktu-port" => Some(&["ktu-tools", "ktu-provider", "ktu-memory", "ktu-cli"]),
        "ktu-provider" => Some(&["ktu-tools", "ktu-port", "ktu-memory", "ktu-cli"]),
        _ => None,
    }
}

/// Lê `cargo metadata` e verifica que nenhuma dependência proibida existe entre
/// os crates do workspace. Devolve o conjunto ordenado de violações encontradas.
fn check_layers() -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|error| format!("falha ao executar `cargo metadata`: {error}"))?;
    if !output.status.success() {
        return Err("`cargo metadata` terminou com erro".to_owned());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("`cargo metadata` devolveu JSON inválido: {error}"))?;
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "`cargo metadata` sem campo `packages`".to_owned())?;
    let workspace: BTreeSet<&str> = packages
        .iter()
        .filter_map(|package| package.get("name").and_then(Value::as_str))
        .collect();
    let mut violations: BTreeSet<String> = BTreeSet::new();
    for package in packages {
        let Some(name) = package.get("name").and_then(Value::as_str) else {
            continue;
        };
        let Some(forbidden) = forbidden_deps(name) else {
            continue;
        };
        let Some(deps) = package.get("dependencies").and_then(Value::as_array) else {
            continue;
        };
        for dep in deps {
            let kind = dep.get("kind");
            let is_normal = kind.is_none() || kind == Some(&Value::Null);
            if !is_normal {
                continue;
            }
            let Some(dep_name) = dep.get("name").and_then(Value::as_str) else {
                continue;
            };
            if workspace.contains(dep_name) && forbidden.contains(&dep_name) {
                violations.insert(format!("{name} → {dep_name}"));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        let joined = violations.into_iter().collect::<Vec<_>>().join(", ");
        Err(format!("firewall de camadas violado: {joined}"))
    }
}
