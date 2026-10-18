//! Porta de **instruções locais**: o `AGENTS.md` mais próximo de um ficheiro.
//!
//! Quando o modelo lê um ficheiro, as instruções do directório desse ficheiro
//! (e dos seus pais, até à raiz) são as mais relevantes — mais do que as globais
//! já no prompt. A porta devolve-as; o kernel anexa-as ao **resultado da tool**
//! (uma vez por turno e por fonte), para que *model-visible ⟺ logged* se mantenha.

/// Instruções locais aplicáveis a um caminho.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Local {
    /// Fonte legível (ex.: `sub/AGENTS.md`) — a chave de deduplicação.
    pub source: String,
    /// Texto das instruções.
    pub text: String,
}

/// Porta: instruções locais de um caminho pedido (relativo à raiz do projecto).
pub trait Instructions: Send + Sync {
    /// O `AGENTS.md` **mais próximo** de `path` (`None` se não houver).
    fn nearest(&self, path: &str) -> Option<Local>;
}

#[cfg(test)]
mod tests {
    use super::Local;

    #[test]
    fn local_is_comparable() {
        let left = Local {
            source: "sub/AGENTS.md".to_owned(),
            text: "regra".to_owned(),
        };
        assert_eq!(left.clone(), left);
    }
}
