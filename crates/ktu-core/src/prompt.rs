//! Montagem do system prompt (§2.11): base do agente + `AGENTS.md` condensado.

/// Base do agente (identidade e disciplina).
pub const BASE: &str = "Es o ktu, um agente de programação. Trabalhas em passos: usa as tools para \
ler e alterar ficheiros e apresenta o teu raciocínio de forma concisa. Nunca inventes resultados: \
baseia-te no que as tools devolvem.";

/// Compõe o system prompt: `base` + `AGENTS.md` condensado (se houver).
#[must_use]
pub fn compose(base: &str, agents_md: Option<&str>) -> String {
    match agents_md {
        Some(text) => {
            let condensed = condense(text);
            if condensed.is_empty() {
                base.to_owned()
            } else {
                format!("{base}\n\n{condensed}")
            }
        }
        None => base.to_owned(),
    }
}

/// Condensa markdown: remove blocos de comentário HTML e colapsa linhas em
/// branco. Puro e sem cache (o `AGENTS.md` é recomposto a cada turno).
#[must_use]
pub fn condense(text: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut in_comment = false;
    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        if in_comment {
            if trimmed.contains("-->") {
                in_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                in_comment = true;
            }
            continue;
        }
        lines.push(line);
    }
    let mut out: Vec<&str> = Vec::new();
    let mut blank = false;
    for line in lines {
        if line.trim().is_empty() {
            if !blank {
                out.push("");
            }
            blank = true;
        } else {
            out.push(line);
            blank = false;
        }
    }
    out.join("\n").trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::{BASE, compose, condense};

    #[test]
    fn condense_drops_html_comments() {
        let text = "# titulo\n<!-- knudge:start -->\nsegredo\n<!-- knudge:end -->\ncorpo";
        let result = condense(text);
        assert!(!result.contains("knudge"));
        assert!(result.contains("corpo"));
    }

    #[test]
    fn condense_collapses_blank_runs() {
        assert_eq!(condense("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn compose_without_agents_md_is_the_base() {
        assert_eq!(compose(BASE, None), BASE);
        assert_eq!(compose(BASE, Some("")), BASE);
    }
}
