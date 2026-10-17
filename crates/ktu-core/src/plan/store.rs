//! Persistência do artefacto do plano (`.ktu/plan/<sessão>.json`), durável.

use std::path::{Path, PathBuf};

use super::Plan;
use crate::ports::Fs;
use crate::{Error, Result};

/// Caminho do artefacto do plano para uma sessão.
#[must_use]
pub fn path(dir: &Path, session: &str) -> PathBuf {
    dir.join(format!("{session}.json"))
}

/// Grava o plano de forma durável (`temp → fsync → rename`).
///
/// # Errors
/// Propaga erros de I/O da porta ou de serialização.
pub fn save(fs: &dyn Fs, dir: &Path, plan: &Plan) -> Result<()> {
    let target = path(dir, &plan.session);
    let temp = dir.join(format!("{}.json.tmp", plan.session));
    let text = serde_json::to_string_pretty(plan).map_err(|source| Error::json("plan", source))?;
    fs.write(&temp, text.as_bytes())?;
    fs.sync(&temp)?;
    fs.rename(&temp, &target)
}

/// Carrega o plano de uma sessão, se existir.
///
/// # Errors
/// Propaga erros de I/O da porta ou um plano malformado.
pub fn load(fs: &dyn Fs, dir: &Path, session: &str) -> Result<Option<Plan>> {
    let target = path(dir, session);
    if !fs.exists(&target) {
        return Ok(None);
    }
    let bytes = fs.read(&target)?;
    let text = String::from_utf8(bytes).map_err(|_| Error::io(&target, "plano nao e UTF-8"))?;
    let plan: Plan = serde_json::from_str(&text).map_err(|source| Error::json("plan", source))?;
    Ok(Some(plan))
}
