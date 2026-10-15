//! Adaptador real da porta `Fs` (borda): o sistema de ficheiros do SO.

use std::path::{Path, PathBuf};

use ktu_core::ports::Fs;
use ktu_core::{Error, Result};

/// Implementação real de [`Fs`] sobre o sistema de ficheiros do SO.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdFs;

impl Fs for StdFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        std::fs::write(path, contents).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(path).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })?;
        let mut out = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| Error::Io {
                path: path.display().to_string(),
                source,
            })?;
            out.push(entry.path());
        }
        out.sort();
        Ok(out)
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        std::fs::canonicalize(path).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to).map_err(|source| Error::Io {
            path: from.display().to_string(),
            source,
        })
    }

    fn sync(&self, path: &Path) -> Result<()> {
        let file = std::fs::File::open(path).map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })?;
        file.sync_all().map_err(|source| Error::Io {
            path: path.display().to_string(),
            source,
        })
    }
}
