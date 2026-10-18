//! `.gitattributes` gerido: regras explícitas para o conhecimento versionado.
//!
//! `eol=lf` mantém os ids estáveis entre plataformas; `merge=union` só no log de
//! eventos (append-only, seguro pelo id do evento); o derivado nunca é
//! auto-mergeado. Em [`GitMode::Excluded`] o bloco é removido.

use super::GitMode;
use super::block;

/// Início do bloco gerido.
pub const MARKER_BEGIN: &str = "# ktu:start";
/// Fim do bloco gerido.
pub const MARKER_END: &str = "# ktu:end";
/// Nome do ficheiro.
pub const FILE: &str = ".gitattributes";

/// Regras geridas, na ordem de emissão (linha vazia separa os grupos).
pub const RULES: &[&str] = &[
    "# Memória: LF mantém os ids estáveis; o log de eventos é append-only (union é seguro).",
    "/.agents/memory/notes.jsonl text eol=lf",
    "/.agents/memory/events/*.jsonl text eol=lf merge=union",
    "",
    "# Conhecimento legível (skills, guardrails): texto normalizado.",
    "/.agents/skills/** text eol=lf",
    "/.agents/guardrails/** text eol=lf",
    "",
    "# Derivado e runtime (reconstruíveis, nunca versionados): sem diff/merge automático.",
    "/.agents/memory/index/** binary linguist-generated",
    "/.ktu/** binary linguist-generated",
];

/// Renderiza o bloco gerido (marcadores + [`RULES`]).
#[must_use]
pub fn render() -> String {
    let mut out = String::from(MARKER_BEGIN);
    out.push('\n');
    for rule in RULES {
        out.push_str(rule);
        out.push('\n');
    }
    out.push_str(MARKER_END);
    out.push('\n');
    out
}

/// Garante (ou remove) o bloco gerido no texto conforme o `mode`.
#[must_use]
pub fn upsert(original: &str, mode: GitMode) -> String {
    let block = match mode {
        GitMode::Excluded => None,
        GitMode::Tracked => Some(render()),
    };
    block::upsert(original, MARKER_BEGIN, MARKER_END, block.as_deref())
}

#[cfg(test)]
mod tests {
    use super::{MARKER_BEGIN, MARKER_END, RULES, render, upsert};
    use crate::git::GitMode;

    #[test]
    fn render_has_markers_and_every_rule() {
        let block = render();
        assert!(block.starts_with(MARKER_BEGIN));
        assert!(block.trim_end().ends_with(MARKER_END));
        for rule in RULES {
            assert!(block.contains(rule));
        }
    }

    #[test]
    fn tracked_upsert_is_idempotent() {
        let once = upsert("# existing\n", GitMode::Tracked);
        assert!(once.contains(MARKER_BEGIN));
        assert_eq!(once, upsert(&once, GitMode::Tracked));
    }

    #[test]
    fn excluded_removes_the_block() {
        let tracked = upsert("", GitMode::Tracked);
        let excluded = upsert(&tracked, GitMode::Excluded);
        assert!(!excluded.contains(MARKER_BEGIN));
        assert_eq!(excluded, "");
    }
}
