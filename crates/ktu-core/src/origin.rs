//! Origem de um recurso efectivo: o **piso global** do utilizador ou o **local** do projecto.
//!
//! É a resposta a «quem manda?» que o invariante **M8** torna visível: nada que o
//! runtime use é invisível, e cada recurso efectivo diz de onde veio. A precedência
//! não é uniforme (é declarada, por tipo de recurso):
//!
//! - guardrails e plugins **somam** (o piso global é aditivo — o local nunca o desliga);
//! - skills e receitas **substituem por nome** (o local ganha).

/// Nível de onde um recurso foi carregado.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// `~/.config/ktu/` — o piso do utilizador, comum a todos os projectos.
    Global,
    /// O projecto (`<root>/`), específico deste repositório.
    Local,
}

impl Origin {
    /// Rótulo estável (contrato de `--json` e das listagens humanas).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Local => "local",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Origin;

    #[test]
    fn labels_are_stable() {
        assert_eq!(Origin::Global.as_str(), "global");
        assert_eq!(Origin::Local.as_str(), "local");
    }
}
