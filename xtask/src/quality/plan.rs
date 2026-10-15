//! Perguntas da bateria de qualidade e o plano sequencial de execução (F5/G4).

use std::collections::BTreeMap;
use std::path::Path;

use super::frontmatter;

/// Encadeamento da questão com a sessão da bateria.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Chain {
    /// Abre uma sessão nova.
    Fresh,
    /// Retoma a sessão anterior.
    SameSession,
    /// Retoma a sessão anterior depois de compactação real.
    Compacted,
}

impl Chain {
    /// Se a questão retoma a sessão anterior (`same_session`).
    #[must_use]
    pub(crate) const fn resumes(self) -> bool {
        !matches!(self, Self::Fresh)
    }

    /// Se a questão exige compactação real antes de correr.
    #[must_use]
    pub(crate) const fn compacts(self) -> bool {
        matches!(self, Self::Compacted)
    }
}

/// Uma pergunta da bateria, lida do frontmatter de `pN/pN.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Question {
    /// Chave da questão no disco (`p1`…`p4`).
    pub(crate) key: String,
    /// Identificador lógico (`Q1`…`Q4`).
    pub(crate) id: String,
    /// Enunciado original (com os paths dos referenciais).
    pub(crate) issue: String,
    /// Ideia de resposta esperada (critério de julgamento).
    pub(crate) expected: String,
    /// Encadeamento com a sessão.
    pub(crate) chain: Chain,
}

impl Question {
    /// Interpreta uma questão a partir do texto de `pN.md`.
    ///
    /// # Errors
    /// Devolve mensagem quando o frontmatter falta ou não tem `question_id`.
    pub(crate) fn parse(key: &str, text: &str) -> Result<Self, String> {
        let (map, _) = frontmatter::parse(text)
            .ok_or_else(|| format!("{key}: frontmatter ausente ou malformado"))?;
        let id = map
            .get("question_id")
            .cloned()
            .ok_or_else(|| format!("{key}: campo question_id em falta"))?;
        Ok(Self {
            key: key.to_owned(),
            id,
            issue: map.get("question_issue").cloned().unwrap_or_default(),
            expected: map
                .get("expected_answer_ideia")
                .cloned()
                .unwrap_or_default(),
            chain: chain_of(&map),
        })
    }

    /// Carrega a questão de `<output>/<key>/<key>.md`.
    ///
    /// # Errors
    /// Propaga o erro de leitura ou de interpretação.
    pub(crate) fn load(root: &Path, output: &Path, key: &str) -> Result<Self, String> {
        let path = output.join(key).join(format!("{key}.md"));
        let text = std::fs::read_to_string(root.join(&path))
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Self::parse(key, &text)
    }
}

/// Um passo do plano: a questão, a sessão, e se sonda a persistência.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Step {
    /// Chave da questão.
    pub(crate) key: String,
    /// Identificador da sessão partilhada pela bateria.
    pub(crate) session: String,
    /// Encadeamento (retoma e/ou compacta).
    pub(crate) chain: Chain,
    /// Sonda de persistência (último passo da bateria).
    pub(crate) probe: bool,
}

/// Planifica a bateria: Q1 abre a sessão; seguintes retomam; o último sonda.
#[must_use]
pub(crate) fn battery(questions: &[Question], session: &str) -> Vec<Step> {
    let last = questions.len().saturating_sub(1);
    questions
        .iter()
        .enumerate()
        .map(|(index, question)| Step {
            key: question.key.clone(),
            session: session.to_owned(),
            chain: if index == 0 {
                Chain::Fresh
            } else if question.chain.compacts() {
                Chain::Compacted
            } else {
                Chain::SameSession
            },
            probe: index == last,
        })
        .collect()
}

/// Deriva o encadeamento das flags `same_session`/`compacted`.
fn chain_of(map: &BTreeMap<String, String>) -> Chain {
    if flag(map, "compacted") {
        Chain::Compacted
    } else if flag(map, "same_session") {
        Chain::SameSession
    } else {
        Chain::Fresh
    }
}

/// Lê uma flag booleana (`true`/`false`), por omissão `false`.
fn flag(map: &BTreeMap<String, String>, key: &str) -> bool {
    map.get(key).is_some_and(|value| value == "true")
}

#[cfg(test)]
mod tests {
    use super::{Chain, Question, battery};

    fn question(key: &str, chain: Chain) -> Question {
        Question {
            key: key.to_owned(),
            id: key.to_uppercase(),
            issue: String::new(),
            expected: String::new(),
            chain,
        }
    }

    #[test]
    fn parse_reads_flags_and_fields() {
        let text = "---\nquestion_id: Q2\nquestion_issue: \"pergunta\"\nsame_session: true\n---\n";
        let Ok(parsed) = Question::parse("p2", text) else {
            return;
        };
        assert_eq!(parsed.id, "Q2");
        assert_eq!(parsed.issue, "pergunta");
        assert_eq!(parsed.chain, Chain::SameSession);
    }

    #[test]
    fn parse_rejects_missing_id() {
        let text = "---\nquestion_issue: x\n---\n";
        assert!(Question::parse("p9", text).is_err());
    }

    #[test]
    fn parse_detects_compacted() {
        let text = "---\nquestion_id: Q4\ncompacted: true\n---\n";
        let Ok(parsed) = Question::parse("p4", text) else {
            return;
        };
        assert!(parsed.chain.compacts());
    }

    #[test]
    fn battery_chains_sessions_and_probes_last() {
        let questions = vec![
            question("p1", Chain::Fresh),
            question("p2", Chain::SameSession),
            question("p4", Chain::Compacted),
        ];
        let steps = battery(&questions, "ktu");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps.first().map(|step| step.chain.resumes()), Some(false));
        assert_eq!(steps.get(1).map(|step| step.chain.resumes()), Some(true));
        assert_eq!(steps.get(2).map(|step| step.chain.compacts()), Some(true));
        assert_eq!(steps.get(2).map(|step| step.probe), Some(true));
    }
}
