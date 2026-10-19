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

/// Directório do conhecimento do projecto (versionável).
pub const AGENTS_DIR: &str = ".agents";

/// Directório dos comandos do projecto (`/nome` na sessão interactiva).
pub const COMMANDS_DIR: &str = ".agents/commands";

/// Directório das receitas **globais** (por baixo do directório do utilizador).
///
/// Uma receita global é um comando do projecto que o utilizador quer em todos os
/// projectos; o local ganha por nome.
pub const RECIPES_DIR: &str = "recipes";

/// Directório dos agentes declarativos do projecto.
pub const AGENTS_FILES_DIR: &str = ".agents/agents";

/// Router de instruções do projecto (na raiz).
pub const AGENTS_FILE: &str = "AGENTS.md";

/// Ficheiro que **substitui** a base do prompt de sistema do projecto.
pub const SYSTEM_FILE: &str = ".agents/SYSTEM.md";

/// Ficheiro que **acrescenta** ao prompt de sistema do projecto.
pub const APPEND_SYSTEM_FILE: &str = ".agents/APPEND_SYSTEM.md";

/// Directório das sessões persistidas (log JSONL + *checkpoint*).
pub const SESSIONS_DIR: &str = ".ktu/sessions";

/// Caminho da configuração do projecto.
pub const CONFIG_FILE: &str = ".agents/config.toml";

/// Template da configuração do projecto (gerada por `ktu init`).
pub const CONFIG_TEMPLATE: &str = "\
# ktu — configuração do projecto (gerada por `ktu init`).
#
# Precedência: flag > `.agents/config.toml` > `~/.config/ktu/config.toml` > defaults.
# O schema completo é definido em H2 (config); este ficheiro é o ponto de partida.
";

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        AGENTS_DIR, AGENTS_FILE, APPEND_SYSTEM_FILE, COMMANDS_DIR, CONFIG_FILE, CONFIG_TEMPLATE,
        INIT_DIRS, NOTES_FILE, SESSIONS_DIR, SYSTEM_FILE,
    };

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
    fn sessions_dir_lives_in_runtime() {
        assert_eq!(SESSIONS_DIR, ".ktu/sessions");
        assert!(INIT_DIRS.contains(&SESSIONS_DIR));
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

    #[test]
    fn skill_and_config_live_under_agents() {
        assert!(CONFIG_FILE.starts_with(".agents/"));
        assert!(CONFIG_TEMPLATE.starts_with('#'));
    }

    #[test]
    fn directives_live_under_agents_and_agents_file_is_at_the_root() {
        assert_eq!(AGENTS_DIR, ".agents");
        assert_eq!(AGENTS_FILE, "AGENTS.md");
        assert_eq!(SYSTEM_FILE, ".agents/SYSTEM.md");
        assert_eq!(APPEND_SYSTEM_FILE, ".agents/APPEND_SYSTEM.md");
        assert_eq!(COMMANDS_DIR, ".agents/commands");
    }
}
