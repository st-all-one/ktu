//! `check-surface` — o **teto de superfície** pública (`surface.toml`, H11).
//!
//! Mede o que o binário expõe (`ktu self surface --json`: subcomandos, flags e
//! tools) e compara com o teto versionado. **Crescer exige subir o teto** numa
//! alteração explícita (ver `wiki/adr/0011-*.md` e `wiki/metodo-medicao.md`).

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use serde_json::Value;

/// Ficheiro do teto (na raiz do repositório).
const FILE: &str = "surface.toml";

/// Medida da superfície (subcomandos, flags, tools).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Surface {
    /// Subcomandos visíveis.
    pub(crate) subcommands: u64,
    /// Flags distintas.
    pub(crate) flags: u64,
    /// Tools que o binário pode oferecer.
    pub(crate) tools: u64,
}

impl Surface {
    /// Os três campos, com o nome usado no `surface.toml` e na mensagem de erro.
    fn fields(self) -> [(&'static str, u64); 3] {
        [
            ("subcommands", self.subcommands),
            ("flags", self.flags),
            ("tools", self.tools),
        ]
    }
}

/// Lê o teto, mede a superfície e falha se a medida o exceder.
pub(crate) fn check() -> Result<(), String> {
    let ceiling = read_ceiling(Path::new(FILE))?;
    let measured = measure()?;
    let mut exceeded: Vec<String> = Vec::new();
    for ((name, limit), (_, actual)) in ceiling.fields().iter().zip(measured.fields()) {
        if actual > *limit {
            exceeded.push(format!("{name}: {actual} > {limit}"));
        }
    }
    if exceeded.is_empty() {
        let mut stdout = io::stdout().lock();
        let _ignored = writeln!(
            stdout,
            "superfície: {} subcomandos, {} flags, {} tools (teto {FILE})",
            measured.subcommands, measured.flags, measured.tools
        );
        return Ok(());
    }
    Err(format!(
        "superfície acima do teto ({}) — suba o teto em {FILE} na mesma alteração \
         (e justifique: wiki/adr/0011-limite-de-400-linhas-e-firewall-mecanico.md)",
        exceeded.join(", ")
    ))
}

/// Lê `[ceiling]` de `surface.toml`.
fn read_ceiling(path: &Path) -> Result<Surface, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("falha ao ler {}: {error}", path.display()))?;
    let value: Value =
        toml::from_str(&text).map_err(|error| format!("{} inválido: {error}", path.display()))?;
    let ceiling = value
        .get("ceiling")
        .ok_or_else(|| format!("{} sem secção [ceiling]", path.display()))?;
    Ok(Surface {
        subcommands: field(ceiling, "subcommands")?,
        flags: field(ceiling, "flags")?,
        tools: field(ceiling, "tools")?,
    })
}

/// Extrai um inteiro positivo de `ceiling`.
fn field(ceiling: &Value, name: &str) -> Result<u64, String> {
    ceiling
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{FILE}: [ceiling] sem `{name}` inteiro positivo"))
}

/// Mede a superfície pelo próprio binário (`ktu self surface --json`).
fn measure() -> Result<Surface, String> {
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "ktu-cli", "--", "self", "surface", "--json",
        ])
        .output()
        .map_err(|error| format!("falha ao executar `cargo run -p ktu-cli`: {error}"))?;
    if !output.status.success() {
        return Err("`ktu self surface` terminou com erro".to_owned());
    }
    let envelope: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("`ktu self surface` devolveu JSON inválido: {error}"))?;
    let data = envelope
        .get("data")
        .ok_or_else(|| "`ktu self surface` sem campo `data`".to_owned())?;
    Ok(Surface {
        subcommands: field(data, "subcommands")?,
        flags: field(data, "flags")?,
        tools: field(data, "tools")?,
    })
}
