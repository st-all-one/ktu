//! Espaço de trabalho isolado do sujeito (F5/G4).
//!
//! Cada sujeito corre num *root* privado (`<work>`), com o corpus de referência
//! **copiado e tornado read-only** e memória/sessões limpas — nenhum sujeito lê ou
//! escreve o que é de outro, e o `REF/` do repositório fica intocado.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Diretórios pesados/derivados ignorados na cópia do corpus.
const SKIP: [&str; 3] = [".git", "target", "node_modules"];

/// Prepara `<work>`: corpus de referência read-only + diretórios de estado limpos.
///
/// # Errors
/// Propaga falhas de remoção, cópia ou `chmod` do espaço de trabalho.
pub(super) fn prepare(root: &Path, work: &Path) -> Result<(), String> {
    if work.exists() {
        make_writable(work)?;
        fs::remove_dir_all(work).map_err(|error| format!("{}: {error}", work.display()))?;
    }
    let reference = work.join("REF").join("knudge");
    copy_tree(&root.join("REF").join("knudge"), &reference)?;
    make_read_only(&reference)?;
    for dir in [".agents/memory", ".ktu/sessions", "exploration"] {
        let path = work.join(dir);
        fs::create_dir_all(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(())
}

/// Arquiva os entregáveis do sujeito (`<work>/exploration`) sob a saída.
///
/// # Errors
/// Propaga falhas de cópia dos ficheiros produzidos.
pub(super) fn archive(work: &Path, output: &Path, subject: &str) -> Result<(), String> {
    let source = work.join("exploration");
    if !source.is_dir() {
        return Ok(());
    }
    let destination = output.join(format!("{subject}-files"));
    copy_tree(&source, &destination)
}

/// Copia recursivamente uma árvore, saltando diretórios de build/versionamento.
///
/// # Errors
/// Propaga erros de leitura de diretório, cópia de ficheiro ou criação de pasta.
pub(super) fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("{}: {error}", destination.display()))?;
    let entries = fs::read_dir(source).map_err(|error| format!("{}: {error}", source.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", source.display()))?;
        let name = entry.file_name();
        if name.to_str().is_some_and(|name| SKIP.contains(&name)) {
            continue;
        }
        let path = entry.path();
        let target = destination.join(&name);
        let kind = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if kind.is_dir() {
            copy_tree(&path, &target)?;
        } else if kind.is_file() {
            fs::copy(&path, &target).map_err(|error| format!("{}: {error}", target.display()))?;
        }
    }
    Ok(())
}

/// Remove a permissão de escrita da árvore (best-effort fora do Unix).
#[cfg(unix)]
fn make_read_only(path: &Path) -> Result<(), String> {
    chmod(path, "a-w")
}

/// Devolve a permissão de escrita à árvore, para poder apagá-la.
#[cfg(unix)]
fn make_writable(path: &Path) -> Result<(), String> {
    chmod(path, "u+w")
}

/// Aplica `chmod -R` a uma árvore.
#[cfg(unix)]
fn chmod(path: &Path, mode: &str) -> Result<(), String> {
    let status = Command::new("chmod")
        .arg("-R")
        .arg(mode)
        .arg(path)
        .status()
        .map_err(|error| format!("chmod {}: {error}", path.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("chmod {} terminou com erro", path.display()))
    }
}

/// Fora do Unix não há `chmod`: a proteção fica por conta do guard do agente.
#[cfg(not(unix))]
fn make_read_only(_path: &Path) -> Result<(), String> {
    Ok(())
}

/// Fora do Unix não há `chmod`.
#[cfg(not(unix))]
fn make_writable(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SKIP;

    #[test]
    fn skips_build_and_vcs_directories() {
        for name in SKIP {
            assert!(SKIP.contains(&name), "{name} deve ser saltado");
        }
        assert!(!SKIP.contains(&"src"));
    }
}
