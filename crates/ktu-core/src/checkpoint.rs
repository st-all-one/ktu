//! Checkpoint **tipado e durável** da sessão (§2.7 / F4).
//!
//! Gravado com `temp → fsync → rename` **depois** de cada turno: sobrevive a um
//! *crash* e permite retomar a sessão com o mesmo corte de compactação.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ports::Fs;
use crate::{Error, Result};

/// Estado versionado e mínimo da sessão (o log cru é a verdade; isto é um índice).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Identificador da sessão.
    pub session: String,
    /// Número de turnos concluídos.
    pub turns: u32,
    /// Se a sessão já sofreu compactação.
    pub compacted: bool,
    /// Índice do corte de compactação, quando aplicado.
    pub keep_from: Option<u32>,
    /// Título curto derivado do primeiro pedido (`None` enquanto não houver pedido).
    #[serde(default)]
    pub title: Option<String>,
    /// Instante da última actualização (ms desde a época).
    pub updated_ms: i64,
}

/// Caminho do checkpoint de uma sessão dentro de `dir`.
#[must_use]
pub fn path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.checkpoint.json"))
}

/// Grava o checkpoint de forma durável (`temp → fsync → rename`).
///
/// # Errors
/// Propaga erros de I/O da porta ou de serialização.
pub fn save(fs: &dyn Fs, dir: &Path, checkpoint: &Checkpoint) -> Result<()> {
    let target = path(dir, &checkpoint.session);
    let temp = dir.join(format!("{}.checkpoint.json.tmp", checkpoint.session));
    let text = serde_json::to_string_pretty(checkpoint)
        .map_err(|source| Error::json("checkpoint", source))?;
    fs.write(&temp, text.as_bytes())?;
    fs.sync(&temp)?;
    fs.rename(&temp, &target)
}

/// Carrega o checkpoint de uma sessão, se existir.
///
/// # Errors
/// Propaga erros de I/O ou um checkpoint malformado.
pub fn load(fs: &dyn Fs, dir: &Path, id: &str) -> Result<Option<Checkpoint>> {
    let target = path(dir, id);
    if !fs.exists(&target) {
        return Ok(None);
    }
    let bytes = fs.read(&target)?;
    let text =
        String::from_utf8(bytes).map_err(|_| Error::io(&target, "checkpoint nao e UTF-8"))?;
    let checkpoint =
        serde_json::from_str(&text).map_err(|source| Error::json("checkpoint", source))?;
    Ok(Some(checkpoint))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::ports::MemFs;

    use super::{Checkpoint, load, path, save};

    fn sample() -> Checkpoint {
        Checkpoint {
            session: "s1".to_owned(),
            turns: 3,
            compacted: true,
            keep_from: Some(6),
            title: Some("arranja o parser".to_owned()),
            updated_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn saves_and_loads_a_checkpoint() {
        let fs = MemFs::new().with_dir("/.ktu/sessions");
        let dir = Path::new("/.ktu/sessions");
        assert!(save(&fs, dir, &sample()).is_ok());
        let loaded = load(&fs, dir, "s1");
        assert_eq!(loaded.ok().flatten(), Some(sample()));
    }

    #[test]
    fn missing_checkpoint_is_none() {
        let fs = MemFs::new();
        assert!(load(&fs, Path::new("/x"), "nope").ok().flatten().is_none());
    }

    #[test]
    fn checkpoint_path_is_per_session() {
        assert_eq!(
            path(Path::new("/s"), "abc"),
            Path::new("/s/abc.checkpoint.json")
        );
    }
}
