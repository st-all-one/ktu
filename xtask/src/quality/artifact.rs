//! Renderização dos artefactos `pN/<subject>-pN.md` (condições + resposta).

use super::frontmatter;
use super::plan::Question;

/// Condições do sujeito registadas no frontmatter do artefacto.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Conditions<'a> {
    /// Provider usado (`llama`).
    pub(crate) provider: &'a str,
    /// Modelo usado.
    pub(crate) model: &'a str,
    /// Nível de esforço de raciocínio (`thinking-level`).
    pub(crate) effort: &'a str,
    /// Nome do sujeito (`agent`).
    pub(crate) agent: &'a str,
}

/// Resultado observado de um passo (compactação e sonda de persistência).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Outcome {
    /// Se a compactação real foi observada no checkpoint.
    pub(crate) compacted: bool,
    /// Resultado da sonda de persistência (só no último passo).
    pub(crate) probe: Option<bool>,
}

/// Renderiza o artefacto: frontmatter de condições + resposta do sujeito.
#[must_use]
pub(crate) fn render(
    question: &Question,
    conditions: &Conditions<'_>,
    answer: &str,
    outcome: Outcome,
) -> String {
    let same_session = if question.chain.resumes() {
        "true"
    } else {
        "false"
    };
    let compacted = if outcome.compacted { "true" } else { "false" };
    let mut pairs: Vec<(&str, &str)> = vec![
        ("question_id", question.id.as_str()),
        ("question_issue", question.issue.as_str()),
        ("expected_answer_ideia", question.expected.as_str()),
        ("provider", conditions.provider),
        ("model", conditions.model),
        ("thinking-level", conditions.effort),
        ("cache", "true"),
        ("same_session", same_session),
        ("compacted", compacted),
    ];
    if let Some(probe) = outcome
        .probe
        .map(|value| if value { "true" } else { "false" })
    {
        pairs.push(("persistence_probe", probe));
    }
    pairs.push(("agent", conditions.agent));
    frontmatter::render(&pairs, answer)
}

#[cfg(test)]
mod tests {
    use super::{Conditions, Outcome, render};
    use crate::quality::plan::{Chain, Question};

    fn question() -> Question {
        Question {
            key: "p1".to_owned(),
            id: "Q1".to_owned(),
            issue: "faça um resumo".to_owned(),
            expected: "resumo".to_owned(),
            chain: Chain::Fresh,
        }
    }

    #[test]
    fn render_emits_conditions_and_body() {
        let conditions = Conditions {
            provider: "llama",
            model: "ornith",
            effort: "low",
            agent: "ktu",
        };
        let outcome = Outcome {
            compacted: false,
            probe: Some(true),
        };
        let text = render(&question(), &conditions, "resposta", outcome);
        assert!(text.contains("question_id: Q1"));
        assert!(text.contains("agent: ktu"));
        assert!(text.contains("persistence_probe: true"));
        assert!(text.ends_with("resposta\n"));
    }
}
