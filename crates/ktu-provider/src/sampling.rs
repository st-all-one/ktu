//! Amostragem do modelo (§2.5) e o esforço de raciocínio.
//!
//! A amostragem é **configuração explícita** — nunca "mágica" escondida no
//! provider. Os valores por omissão são os do modelo de referência (Ornith-1.5-9B,
//! modo *thinking*, tarefas gerais).

use serde::{Deserialize, Serialize};

/// Esforço de raciocínio pedido ao modelo (quando o template o suporta).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// Respostas diretas, pouco pensamento.
    Low,
    /// Equilíbrio entre qualidade e latência.
    Medium,
    /// Muito pensamento.
    High,
    /// Pensamento máximo (omissão).
    #[default]
    Xhigh,
}

impl ReasoningEffort {
    /// Texto estável (contrato de log / wire).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

/// Parâmetros de amostragem enviados ao `llama-server`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Sampling {
    /// Temperatura.
    pub temperature: f32,
    /// *Nucleus sampling*.
    pub top_p: f32,
    /// Top-k.
    pub top_k: u32,
    /// Min-p.
    pub min_p: f32,
    /// Penalidade de presença.
    pub presence_penalty: f32,
    /// Penalidade de repetição.
    pub repetition_penalty: f32,
}

impl Sampling {
    /// Amostragem de *thinking mode* do modelo de referência (tarefas gerais).
    #[must_use]
    pub const fn ornith() -> Self {
        Self {
            temperature: 1.0,
            top_p: 0.95,
            top_k: 20,
            min_p: 0.0,
            presence_penalty: 1.5,
            repetition_penalty: 1.0,
        }
    }

    /// Amostragem para tarefas de código precisas (documentada pelo modelo).
    #[must_use]
    pub const fn ornith_coding() -> Self {
        Self {
            temperature: 0.6,
            top_p: 0.95,
            top_k: 20,
            min_p: 0.0,
            presence_penalty: 0.0,
            repetition_penalty: 1.0,
        }
    }
}

impl Default for Sampling {
    fn default() -> Self {
        Self::ornith()
    }
}

#[cfg(test)]
mod tests {
    use super::{ReasoningEffort, Sampling};

    #[test]
    fn ornith_defaults_match_the_model_card() {
        let sampling = Sampling::ornith();
        assert!((sampling.temperature - 1.0).abs() < f32::EPSILON);
        assert_eq!(sampling.top_k, 20);
        assert!((sampling.presence_penalty - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn default_effort_is_xhigh() {
        assert_eq!(ReasoningEffort::default(), ReasoningEffort::Xhigh);
        assert_eq!(ReasoningEffort::Medium.as_str(), "medium");
    }
}
