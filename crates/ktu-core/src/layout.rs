//! Layout de conhecimento (`.agents/`) e runtime (`.ktu/`), criado por `ktu init`.
//!
//! A separação não é cosmética (§2.11 do `plan.md`): `.agents/` é o que o humano
//! revê e versiona (skills, guardrails, memória); `.ktu/` é o que o agente
//! produz e consome em execução (sessões, plugins, lixo, planos). O **plano** do
//! layout vive aqui (puro); a execução fica na borda (`ktu-cli`).

/// Directórios criados por `ktu init`, por ordem de criação.
pub const INIT_DIRS: &[&str] = &[
    ".agents/skills",
    ".agents/guardrails",
    ".agents/memory",
    ".ktu/sessions",
    ".ktu/plugins",
    ".ktu/trash",
    ".ktu/plan",
];

/// Ficheiro fonte da verdade da memória (criado vazio se ausente).
pub const NOTES_FILE: &str = ".agents/memory/notes.jsonl";

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{INIT_DIRS, NOTES_FILE};

    #[test]
    fn init_dirs_are_unique_and_top_level() {
        let unique: BTreeSet<&str> = INIT_DIRS.iter().copied().collect();
        assert_eq!(unique.len(), INIT_DIRS.len());
        assert!(
            INIT_DIRS
                .iter()
                .all(|dir| dir.starts_with(".agents/") || dir.starts_with(".ktu/"))
        );
    }

    #[test]
    fn notes_file_lives_in_agents_memory() {
        assert!(NOTES_FILE.starts_with(".agents/memory/"));
        assert!(INIT_DIRS.contains(&".agents/memory"));
    }

    #[test]
    fn init_dirs_match_the_golden() {
        let golden = [
            ".agents/skills",
            ".agents/guardrails",
            ".agents/memory",
            ".ktu/sessions",
            ".ktu/plugins",
            ".ktu/trash",
            ".ktu/plan",
        ];
        assert_eq!(INIT_DIRS, golden.as_slice());
    }
}
