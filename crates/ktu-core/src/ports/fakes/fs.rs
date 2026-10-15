//! *Fakes* da porta [`Fs`](super::super::Fs).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::super::Fs;
use crate::{Error, Result};

/// Sistema de ficheiros em memória, indexado por caminho.
#[derive(Debug, Default)]
pub struct MemFs {
    files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
    dirs: Mutex<BTreeSet<PathBuf>>,
}

impl MemFs {
    /// Cria um `MemFs` vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta um ficheiro pré-existente (construtor de teste).
    #[must_use]
    pub fn with_file(mut self, path: impl Into<PathBuf>, contents: impl Into<Vec<u8>>) -> Self {
        let path = path.into();
        self.register_parents(&path);
        let files = self.files.get_mut().unwrap_or_else(PoisonError::into_inner);
        let _replaced = files.insert(path, contents.into());
        self
    }

    /// Regista um directório pré-existente (construtor de teste).
    #[must_use]
    pub fn with_dir(self, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        self.register_parents(&path);
        let _inserted = self.dirs().insert(path);
        self
    }

    fn register_parents(&self, path: &Path) {
        let mut dirs = self.dirs();
        let mut current = path.parent();
        while let Some(dir) = current {
            if dir.as_os_str().is_empty() {
                break;
            }
            let _inserted = dirs.insert(dir.to_path_buf());
            current = dir.parent();
        }
    }

    fn files(&self) -> MutexGuard<'_, BTreeMap<PathBuf, Vec<u8>>> {
        self.files.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn dirs(&self) -> MutexGuard<'_, BTreeSet<PathBuf>> {
        self.dirs.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Fs for MemFs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.files()
            .get(path)
            .cloned()
            .ok_or_else(|| Error::io(path, "ficheiro inexistente"))
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        self.register_parents(path);
        let _replaced = self.files().insert(path.to_path_buf(), contents.to_vec());
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files().contains_key(path) || self.dirs().contains(path)
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut entries: BTreeSet<PathBuf> = BTreeSet::new();
        for file in self.files().keys() {
            if file.parent() == Some(path) {
                let _inserted = entries.insert(file.clone());
            }
        }
        let dirs = self.dirs();
        for dir in &*dirs {
            if dir.parent() == Some(path) {
                let _inserted = entries.insert(dir.clone());
            }
        }
        Ok(entries.into_iter().collect())
    }

    fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        if self.exists(path) {
            Ok(path.to_path_buf())
        } else {
            Err(Error::io(path, "caminho inexistente"))
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs().contains(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let mut files = self.files();
        let contents = files
            .remove(from)
            .ok_or_else(|| Error::io(from, "ficheiro inexistente"))?;
        let _replaced = files.insert(to.to_path_buf(), contents);
        Ok(())
    }

    fn sync(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}
