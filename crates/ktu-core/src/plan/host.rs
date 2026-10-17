//! Host da tool `plan` sobre a porta `Fs` (aplicar + persistir).

use std::path::PathBuf;

use super::{Plan, actions, store};
use crate::ports::Fs;
use crate::tools::ToolOutcome;

/// Host da tool `plan` ligado a um directório `.ktu/plan/` e a uma sessão.
pub struct FsPlanHost<'a> {
    fs: &'a dyn Fs,
    dir: PathBuf,
    session: String,
}

impl<'a> FsPlanHost<'a> {
    /// Cria o host para uma sessão e um directório `.ktu/plan/`.
    #[must_use]
    pub fn new(fs: &'a dyn Fs, dir: impl Into<PathBuf>, session: impl Into<String>) -> Self {
        Self {
            fs,
            dir: dir.into(),
            session: session.into(),
        }
    }

    /// Aplica uma acção da tool e devolve o resultado (já persistido).
    #[must_use]
    pub fn execute(&self, arguments: &str) -> ToolOutcome {
        let action = match actions::parse_action(arguments) {
            Ok(action) => action,
            Err(error) => return ToolOutcome::error(error),
        };
        let mut plan = match store::load(self.fs, &self.dir, &self.session) {
            Ok(Some(plan)) => plan,
            Ok(None) => Plan::new(self.session.clone()),
            Err(error) => return ToolOutcome::error(format!("plano indisponivel: {error}")),
        };
        let summary = match actions::apply(&mut plan, action) {
            Ok(summary) => summary,
            Err(error) => return ToolOutcome::error(error),
        };
        match store::save(self.fs, &self.dir, &plan) {
            Ok(()) => ToolOutcome::ok(summary),
            Err(error) => ToolOutcome::error(format!("nao foi possivel gravar o plano: {error}")),
        }
    }
}
