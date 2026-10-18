//! `.git/info/exclude` gerido (nunca `.gitignore`).
//!
//! Em [`GitMode::Tracked`] versiona-se `.agents/` e exclui-se só o derivado e o
//! runtime; em [`GitMode::Excluded`] exclui-se `.agents/` e `.ktu/` inteiros.
//! Alternar entre modos **reverte** as linhas do modo anterior (idempotente).

use super::GitMode;

/// Padrões excluídos em [`GitMode::Tracked`] (derivado + runtime).
pub const TRACKED_PATTERNS: &[&str] = &["/.agents/memory/index/", "/.ktu/"];
/// Padrões excluídos em [`GitMode::Excluded`] (conhecimento + runtime).
pub const EXCLUDED_PATTERNS: &[&str] = &["/.agents/", "/.ktu/"];

/// Padrões desejados para `mode`.
#[must_use]
pub fn patterns(mode: GitMode) -> &'static [&'static str] {
    match mode {
        GitMode::Tracked => TRACKED_PATTERNS,
        GitMode::Excluded => EXCLUDED_PATTERNS,
    }
}

/// `true` se a linha pertence ao conjunto gerido (qualquer dos modos).
#[must_use]
pub fn is_managed(line: &str) -> bool {
    let trimmed = line.trim();
    TRACKED_PATTERNS.contains(&trimmed) || EXCLUDED_PATTERNS.contains(&trimmed)
}

/// Devolve o novo conteúdo do exclude (canónico) para `mode`.
#[must_use]
pub fn apply(original: &str, mode: GitMode) -> String {
    let mut lines: Vec<String> = original
        .lines()
        .filter(|line| !is_managed(line))
        .map(str::to_owned)
        .collect();
    for pattern in patterns(mode) {
        if !lines.iter().any(|line| line.trim() == *pattern) {
            lines.push((*pattern).to_owned());
        }
    }
    let mut updated = lines.join("\n");
    if !updated.is_empty() {
        updated.push('\n');
    }
    updated
}

#[cfg(test)]
mod tests {
    use super::{apply, is_managed};
    use crate::git::GitMode;

    #[test]
    fn tracked_excludes_only_derived_and_runtime() {
        let out = apply("", GitMode::Tracked);
        assert!(out.contains("/.agents/memory/index/"));
        assert!(out.contains("/.ktu/"));
        assert!(!out.contains("/.agents/\n"));
    }

    #[test]
    fn excluded_excludes_knowledge_and_runtime() {
        let out = apply("", GitMode::Excluded);
        assert!(out.contains("/.agents/"));
        assert!(out.contains("/.ktu/"));
    }

    #[test]
    fn switching_modes_reverts_the_previous_lines() {
        let tracked = apply("", GitMode::Tracked);
        let excluded = apply(&tracked, GitMode::Excluded);
        assert!(!excluded.contains("/.agents/memory/index/"));
        let back = apply(&excluded, GitMode::Tracked);
        assert!(!back.contains("/.agents/\n"));
        assert_eq!(back, tracked);
    }

    #[test]
    fn user_lines_are_preserved_and_apply_is_idempotent() {
        let original = "/target/\n# nota do humano\n";
        let once = apply(original, GitMode::Tracked);
        assert!(once.contains("/target/"));
        assert!(once.contains("# nota do humano"));
        assert_eq!(once, apply(&once, GitMode::Tracked));
    }

    #[test]
    fn managed_lines_are_recognised() {
        assert!(is_managed("/.ktu/"));
        assert!(is_managed("  /.agents/  "));
        assert!(!is_managed("/target/"));
    }
}
