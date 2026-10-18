//! Modo de arranque da sessão e compactação forçada (`ktu run --compact`).

use ktu_core::compact::keep_from_for_turns;
use ktu_core::session::SessionLog;

/// Destino da sessão do turno: **efémera** (1 turno) ou **persistente** com id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SessionTarget {
    /// Efémera: não carrega nem grava log/checkpoint (uso #2, pipelines).
    Ephemeral,
    /// Persistente: `.ktu/sessions/<id>.jsonl` (e lock).
    Persistent(String),
}

impl SessionTarget {
    /// Identificador persistente, se houver.
    pub(crate) fn id(&self) -> Option<&str> {
        match self {
            Self::Persistent(id) => Some(id),
            Self::Ephemeral => None,
        }
    }

    /// Rótulo estável para artefactos auxiliares (plano, subagentes).
    pub(crate) fn label(&self) -> &str {
        match self {
            Self::Persistent(id) => id,
            Self::Ephemeral => "ephemeral",
        }
    }

    /// `true` se a sessão persiste em disco.
    pub(crate) const fn is_persistent(&self) -> bool {
        matches!(self, Self::Persistent(_))
    }
}

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

    #[test]
    fn session_target_ephemeral_vs_persistent() {
        use super::SessionTarget;
        let ephemeral = SessionTarget::Ephemeral;
        assert!(!ephemeral.is_persistent());
        assert_eq!(ephemeral.id(), None);
        assert_eq!(ephemeral.label(), "ephemeral");
        let persistent = SessionTarget::Persistent("s1".to_owned());
        assert!(persistent.is_persistent());
        assert_eq!(persistent.id(), Some("s1"));
        assert_eq!(persistent.label(), "s1");
    }
}
