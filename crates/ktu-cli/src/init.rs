//! `ktu init` — cria o layout `.agents/` (conhecimento) e `.ktu/` (runtime).
//!
//! O **plano** do layout é puro ([`ktu_core::layout`]); aqui apenas se executa o
//! efeito no sistema de ficheiros, na borda. É idempotente: só cria o que falta.

use std::fs;
use std::io;
use std::path::Path;

use ktu_core::layout;

/// Cria o layout sob `root`. Devolve os caminhos criados (relativos), por ordem.
///
/// Uma segunda chamada sobre um layout já existente devolve um vector vazio.
pub(crate) fn run(root: &Path) -> io::Result<Vec<&'static str>> {
    let mut created = Vec::new();
    for dir in layout::INIT_DIRS {
        let path = root.join(dir);
        if !path.is_dir() {
            fs::create_dir_all(&path)?;
            created.push(*dir);
        }
    }
    let notes = root.join(layout::NOTES_FILE);
    if !notes.is_file() {
        fs::write(&notes, [])?;
        created.push(layout::NOTES_FILE);
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io;
    use std::process;

    use ktu_core::layout;

    use super::run;

    #[test]
    fn init_creates_layout_idempotently() -> io::Result<()> {
        let suffix = process::id();
        let root = env::temp_dir().join(format!("ktu-init-test-{suffix}"));
        let _ignored = fs::remove_dir_all(&root);

        let created = run(&root)?;
        assert_eq!(created.len(), layout::INIT_DIRS.len().saturating_add(1));
        assert!(root.join(".agents/memory/notes.jsonl").is_file());
        assert!(root.join(".ktu/sessions").is_dir());

        let second = run(&root)?;
        assert_eq!(second, Vec::<&'static str>::new());

        let _ignored = fs::remove_dir_all(&root);
        Ok(())
    }
}
