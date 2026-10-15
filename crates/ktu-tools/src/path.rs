//! Confinamento de caminhos ao *root* da sessão (defesa em profundidade).
//!
//! O `guard` já valida lexicalmente; aqui resolve-se o caminho **real** (segue
//! *symlinks*) e confirma-se que continua dentro do *root*.

use std::path::{Path, PathBuf};

use ktu_core::guard::resolve_within;
use ktu_core::ports::Fs;

/// Resolve um caminho que deve **existir**, dentro do *root*.
///
/// # Errors
/// Devolve a mensagem de recusa se o caminho escapar (lexical ou via *symlink*).
pub(crate) fn resolve_existing(
    fs: &dyn Fs,
    root: &Path,
    requested: &str,
) -> Result<PathBuf, String> {
    let lexical = resolve_within(root, requested)
        .ok_or_else(|| format!("path recusado: {requested} sai do root da sessao"))?;
    let real = fs
        .canonicalize(&lexical)
        .map_err(|_| format!("caminho inexistente: {requested}"))?;
    if real.starts_with(real_root(fs, root)) {
        Ok(real)
    } else {
        Err(format!("path recusado: {requested} sai do root (symlink)"))
    }
}

/// Resolve um caminho para **escrita** (o ficheiro pode não existir ainda).
///
/// # Errors
/// Devolve a mensagem de recusa se o directório pai não existir ou escapar.
pub(crate) fn resolve_for_write(
    fs: &dyn Fs,
    root: &Path,
    requested: &str,
) -> Result<PathBuf, String> {
    let lexical = resolve_within(root, requested)
        .ok_or_else(|| format!("path recusado: {requested} sai do root da sessao"))?;
    let parent = lexical
        .parent()
        .ok_or_else(|| format!("path sem directório: {requested}"))?;
    let name = lexical
        .file_name()
        .ok_or_else(|| format!("path sem nome de ficheiro: {requested}"))?;
    let parent_real = fs
        .canonicalize(parent)
        .map_err(|_| format!("directório inexistente: {}", parent.display()))?;
    if !parent_real.starts_with(real_root(fs, root)) {
        return Err(format!("path recusado: {requested} sai do root (symlink)"));
    }
    Ok(parent_real.join(name))
}

/// Caminho real do *root* (ou o próprio, se ainda não existir).
fn real_root(fs: &dyn Fs, root: &Path) -> PathBuf {
    fs.canonicalize(root).unwrap_or_else(|_| root.to_path_buf())
}
