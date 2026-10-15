//! Modo de arranque da sessão e compactação forçada (`ktu run --compact`).

use ktu_core::compact::keep_from_for_turns;
use ktu_core::session::SessionLog;

/// Modo de arranque da sessão pedido na linha de comandos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionMode {
    /// Começa uma sessão nova.
    Fresh,
    /// Retoma a sessão existente.
    Resume,
    /// Retoma a sessão e força compactação antes do turno.
    ResumeCompacted,
}

impl SessionMode {
    /// Se deve carregar o log existente.
    pub(crate) const fn resumes(self) -> bool {
        !matches!(self, Self::Fresh)
    }

    /// Se deve forçar uma compactação do contexto antes do turno.
    pub(crate) const fn compacts(self) -> bool {
        matches!(self, Self::ResumeCompacted)
    }
}

/// Força um corte de compactação antes do turno (mantém apenas o último turno).
///
/// Gatilho determinístico usado pela bateria de qualidade na questão marcada
/// `compacted: true`; sem ele a compactação só correria por orçamento — e um
/// contexto pequeno nunca a dispara.
pub(crate) fn force_compaction(log: &mut SessionLog) {
    if let Some(from) = keep_from_for_turns(log.events(), 1) {
        log.set_compaction(from);
    }
}

#[cfg(test)]
mod tests {
    use super::SessionMode;

    #[test]
    fn fresh_does_not_resume_nor_compact() {
        assert!(!SessionMode::Fresh.resumes());
        assert!(!SessionMode::Fresh.compacts());
    }

    #[test]
    fn resume_and_compacted_behave() {
        assert!(SessionMode::Resume.resumes());
        assert!(!SessionMode::Resume.compacts());
        assert!(SessionMode::ResumeCompacted.resumes());
        assert!(SessionMode::ResumeCompacted.compacts());
    }
}
