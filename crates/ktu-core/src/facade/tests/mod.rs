//! Testes da fachada: incorporação com *fakes*, determinística (sem modelo real).

use std::path::Path;

use super::{Ktu, final_text, system_prompt};
use crate::Result;
use crate::agent::{AgentRegistry, AgentSpec};
use crate::budget::BudgetGate;
use crate::cancel::CancelToken;
use crate::domain::{Completion, StopReason, ToolCall, ToolCallId, ToolSpec};
use crate::guard::AgentId;
use crate::plan::Mode;
use crate::ports::{FixedClock, MemFs, NoopLogger, NoopMetrics, Timestamp};
use crate::session::{Event, SessionLog};
use crate::testing::{FakeProvider, FakeTools};
use crate::tools::ToolOutcome;

const ROOT: &str = "/projeto";

fn spec(name: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_owned(),
        description: String::new(),
        parameters: "{}".to_owned(),
    }
}

fn call(id: &str, name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new(id),
        name: name.to_owned(),
        arguments: arguments.to_owned(),
    }
}

fn text(value: &str) -> Completion {
    Completion {
        text: value.to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    }
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

/// Projecto de teste: ficheiros em memória, relógio fixo e *fakes* de provider/tools.
struct Fixture {
    fs: MemFs,
    clock: FixedClock,
    logger: NoopLogger,
    metrics: NoopMetrics,
    provider: FakeProvider,
    tools: FakeTools,
    registry: AgentRegistry,
    plan: AgentSpec,
    budget: BudgetGate,
    cancel: CancelToken,
}

impl Fixture {
    fn new(script: Vec<Completion>) -> Self {
        Self {
            fs: MemFs::new(),
            clock: FixedClock::new(Timestamp::from_millis(0)),
            logger: NoopLogger,
            metrics: NoopMetrics,
            provider: FakeProvider::new(script),
            tools: FakeTools::new(
                vec![spec("read_file"), spec("write_file")],
                vec![ToolOutcome::ok("conteudo")],
            ),
            registry: AgentRegistry::new(
                AgentSpec::new(AgentId::Primary, names(&["read_file", "write_file"]), 1),
                None,
            ),
            plan: AgentSpec::new(AgentId::Primary, names(&["read_file"]), 1),
            budget: BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS),
            cancel: CancelToken::new(),
        }
    }

    /// Projecto com `AGENTS.md` (raiz + aninhado) e uma regra declarativa.
    fn with_project_files(self) -> Self {
        Self {
            fs: self
                .fs
                .with_file(
                    Path::new(ROOT).join("AGENTS.md"),
                    "# Regras\n\nusa tools.\n",
                )
                .with_file(Path::new(ROOT).join("sub/AGENTS.md"), "regra local.\n")
                .with_dir(Path::new(ROOT).join(".agents").join("guardrails"))
                .with_file(
                    Path::new(ROOT)
                        .join(".agents")
                        .join("guardrails")
                        .join("deny-env.md"),
                    "---\nid: deny-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n",
                ),
            ..self
        }
    }

    fn build(&self) -> Result<Ktu<'_>> {
        Ktu::builder()
            .root(ROOT)
            .fs(&self.fs)
            .clock(&self.clock)
            .logger(&self.logger)
            .metrics(&self.metrics)
            .provider(&self.provider)
            .tools(&self.tools)
            .budget(&self.budget)
            .cancel(&self.cancel)
            .registry(&self.registry)
            .plan_spec(&self.plan)
            .build()
    }

    /// Monta com o directório do **utilizador** (directriz global).
    fn build_with_user<'u>(&'u self, user: &'u Path) -> Result<Ktu<'u>> {
        Ktu::builder()
            .root(ROOT)
            .fs(&self.fs)
            .clock(&self.clock)
            .logger(&self.logger)
            .metrics(&self.metrics)
            .provider(&self.provider)
            .tools(&self.tools)
            .budget(&self.budget)
            .cancel(&self.cancel)
            .registry(&self.registry)
            .plan_spec(&self.plan)
            .user_dir(user)
            .build()
    }
}

#[test]
fn build_names_the_missing_port() {
    let fixture = Fixture::new(vec![text("x")]);
    let error = Ktu::builder()
        .root(ROOT)
        .clock(&fixture.clock)
        .logger(&fixture.logger)
        .metrics(&fixture.metrics)
        .provider(&fixture.provider)
        .tools(&fixture.tools)
        .budget(&fixture.budget)
        .cancel(&fixture.cancel)
        .registry(&fixture.registry)
        .build();
    let message = match error {
        Ok(_) => String::new(),
        Err(error) => error.to_string(),
    };
    assert!(message.contains("porta Fs"), "{message}");
}

#[tokio::test]
async fn turn_runs_with_fake_ports_and_reports_the_outcome() {
    let fixture = Fixture::new(vec![text("feito")]);
    let ktu = fixture.build();
    assert!(ktu.is_ok());
    let Ok(ktu) = ktu else {
        return;
    };
    let mut log = SessionLog::new();
    let outcome = ktu.turn(&mut log, "faz X").await;
    assert!(outcome.is_ok());
    let Ok(outcome) = outcome else {
        return;
    };
    assert_eq!(outcome.reason, StopReason::EndTurn);
    assert_eq!(outcome.text, "feito");
    assert!(outcome.is_ok());
    // O log é a sessão: turno aberto e fechado, com a resposta final.
    assert_eq!(log.verify(), Vec::new());
    assert!(matches!(
        log.events().first(),
        Some(Event::TurnStart { .. })
    ));
    assert!(matches!(
        log.events().last(),
        Some(Event::TurnEnd {
            reason: StopReason::EndTurn
        })
    ));
}

#[tokio::test]
async fn a_tool_call_runs_through_the_injected_executor() {
    let fixture = Fixture::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "read_file", r#"{"path":"a.txt"}"#)],
        },
        text("lido"),
    ]);
    let ktu = fixture.build();
    let Ok(ktu) = ktu else {
        return;
    };
    let mut log = SessionLog::new();
    let outcome = ktu.turn(&mut log, "le a.txt").await;
    assert!(outcome.is_ok());
    assert_eq!(outcome.map(|value| value.text).unwrap_or_default(), "lido");
    assert_eq!(fixture.tools.recorded_calls().len(), 1);
}

#[test]
fn agents_md_enters_the_system_prompt() {
    let fixture = Fixture::new(vec![text("x")]).with_project_files();
    let ktu = fixture.build();
    let Ok(ktu) = ktu else {
        return;
    };
    assert!(ktu.system().contains("usa tools."));
    assert!(ktu.system().contains("## sub/AGENTS.md\n\nregra local."));
    assert!(ktu.root().ends_with("projeto"));
    assert_eq!(system_prompt(&fixture.fs, Path::new(ROOT)), ktu.system());
}

#[test]
fn directives_replace_and_append_the_system_prompt() {
    let fixture = Fixture::new(vec![text("x")]);
    let agents = Path::new(ROOT).join(".agents");
    let user = Path::new("/utilizador");
    let fixture = Fixture {
        fs: fixture
            .fs
            .with_file(user.join("SYSTEM.md"), "persona do utilizador")
            .with_file(user.join("APPEND_SYSTEM.md"), "extra do utilizador"),
        ..fixture
    };
    // Só directriz do utilizador: ela entra no prompt.
    let ktu = fixture.build_with_user(user);
    let Ok(ktu) = ktu else {
        return;
    };
    assert_eq!(ktu.system(), "persona do utilizador\n\nextra do utilizador");

    // Com directriz de projecto, o projecto ganha ficheiro a ficheiro.
    let fixture = Fixture {
        fs: fixture
            .fs
            .with_dir(agents.clone())
            .with_file(agents.join("SYSTEM.md"), "persona do projeto")
            .with_file(agents.join("APPEND_SYSTEM.md"), "lembra-te disto"),
        ..fixture
    };
    let ktu = fixture.build_with_user(user);
    let Ok(ktu) = ktu else {
        return;
    };
    assert_eq!(ktu.system(), "persona do projeto\n\nlembra-te disto");
}

#[test]
fn a_broken_guardrail_stops_the_build() {
    let fixture = Fixture::new(vec![text("x")]);
    let fixture = Fixture {
        fs: fixture
            .fs
            .with_dir(Path::new(ROOT).join(".agents").join("guardrails"))
            .with_file(
                Path::new(ROOT)
                    .join(".agents")
                    .join("guardrails")
                    .join("quebrada.md"),
                "sem frontmatter\n",
            ),
        ..fixture
    };
    let error = fixture.build();
    let message = match error {
        Ok(_) => String::new(),
        Err(error) => error.to_string(),
    };
    assert!(message.contains("quebrada.md"), "{message}");
}

#[tokio::test]
async fn plan_mode_without_a_read_only_spec_is_refused() {
    let fixture = Fixture::new(vec![text("x")]);
    let ktu = Ktu::builder()
        .root(ROOT)
        .fs(&fixture.fs)
        .clock(&fixture.clock)
        .logger(&fixture.logger)
        .metrics(&fixture.metrics)
        .provider(&fixture.provider)
        .tools(&fixture.tools)
        .budget(&fixture.budget)
        .cancel(&fixture.cancel)
        .registry(&fixture.registry)
        .build();
    let Ok(ktu) = ktu else {
        return;
    };
    let mut log = SessionLog::new();
    log.set_mode(Mode::Plan);
    let outcome = ktu.turn(&mut log, "faz X").await;
    assert!(outcome.is_err());
    assert!(
        !log.events()
            .iter()
            .any(|event| matches!(event, Event::TurnStart { .. })),
        "o turno recusado nao comeca"
    );
}

#[tokio::test]
async fn plan_mode_offers_only_the_read_only_tools() {
    let fixture = Fixture::new(vec![text("plano")]);
    let ktu = fixture.build();
    let Ok(ktu) = ktu else {
        return;
    };
    let mut log = SessionLog::new();
    log.set_mode(Mode::Plan);
    let outcome = ktu.turn(&mut log, "planeia").await;
    assert!(outcome.is_ok());
    assert_eq!(fixture.provider.tool_counts(), vec![1]);
}

#[tokio::test]
async fn sessions_round_trip_through_the_facade() {
    let fixture = Fixture::new(vec![text("feito")]);
    let ktu = fixture.build();
    let Ok(ktu) = ktu else {
        return;
    };
    assert_eq!(
        ktu.sessions_dir(),
        Path::new(ROOT).join(".ktu").join("sessions")
    );
    let mut log = SessionLog::new();
    let _ignored = ktu.turn(&mut log, "faz X").await;
    assert!(ktu.save_session("s1", &log).is_ok());
    let loaded = ktu.session("s1");
    assert!(loaded.is_ok());
    assert_eq!(
        loaded.unwrap_or_default().map(|value| value.events().len()),
        Some(log.events().len())
    );
    // Sessão inexistente não é erro: é ausência.
    assert!(ktu.session("nope").unwrap_or_default().is_none());
}

#[test]
fn final_text_reads_only_the_current_turn() {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "primeiro".to_owned(),
    });
    log.push(Event::Assistant {
        content: "antigo".to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    });
    log.push(Event::TurnEnd {
        reason: StopReason::EndTurn,
    });
    assert_eq!(final_text(&log).as_deref(), Some("antigo"));
    log.push(Event::TurnStart {
        prompt: "segundo".to_owned(),
    });
    assert_eq!(final_text(&log), None);
    log.push(Event::Assistant {
        content: "novo".to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    });
    assert_eq!(final_text(&log).as_deref(), Some("novo"));
}

#[cfg(test)]
mod guardrails;
