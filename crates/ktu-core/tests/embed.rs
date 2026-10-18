//! Teste de incorporação: um crate **externo** usa a fachada [`Ktu`] com portas *fake*.
//!
//! Prova o contrato do uso #1 (§2.1 do `plan.md`): o embedder só vê `Ktu`,
//! `TurnOutcome`, `SessionLog`/`Event` e as portas que injecta — nunca `Deps`,
//! `Ports`, `Capabilities`, `LoopConfig` nem `run_turn`.

use std::path::Path;

use ktu_core::agent::{AgentRegistry, AgentSpec};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Completion, StopReason, ToolCall, ToolCallId, ToolSpec};
use ktu_core::facade::Ktu;
use ktu_core::guard::AgentId;
use ktu_core::ports::{FixedClock, MemFs, NoopLogger, NoopMetrics, Timestamp};
use ktu_core::session::{Event, SessionLog};
use ktu_core::store;
use ktu_core::testing::{FakeProvider, FakeTools};
use ktu_core::tools::ToolOutcome;

const ROOT: &str = "/projeto";

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

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

/// Script de dois passos: chama `read_file` e responde.
fn script() -> Vec<Completion> {
    vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "read_file", r#"{"path":"a.txt"}"#)],
        },
        text("li a.txt"),
    ]
}

/// Monta a fachada com *fakes* (o embedder não constrói adaptadores reais).
#[allow(
    clippy::too_many_arguments,
    reason = "composicao explicita de DI no teste"
)]
fn build<'a>(
    fs: &'a MemFs,
    clock: &'a FixedClock,
    logger: &'a NoopLogger,
    metrics: &'a NoopMetrics,
    provider: &'a FakeProvider,
    tools: &'a FakeTools,
    registry: &'a AgentRegistry,
    budget: &'a BudgetGate,
    cancel: &'a CancelToken,
) -> ktu_core::Result<Ktu<'a>> {
    Ktu::builder()
        .root(ROOT)
        .fs(fs)
        .clock(clock)
        .logger(logger)
        .metrics(metrics)
        .provider(provider)
        .tools(tools)
        .budget(budget)
        .cancel(cancel)
        .registry(registry)
        .build()
}

#[tokio::test]
async fn an_embedder_runs_a_tool_turn_and_persists_the_session() {
    let fs = MemFs::new().with_dir(Path::new(ROOT).join(".ktu").join("sessions"));
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let logger = NoopLogger;
    let metrics = NoopMetrics;
    let provider = FakeProvider::new(script());
    let tools = FakeTools::new(vec![spec("read_file")], vec![ToolOutcome::ok("conteudo")]);
    let registry = AgentRegistry::new(
        AgentSpec::new(AgentId::Primary, names(&["read_file"]), 1),
        None,
    );
    let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
    let cancel = CancelToken::new();

    let ktu = build(
        &fs, &clock, &logger, &metrics, &provider, &tools, &registry, &budget, &cancel,
    );
    assert!(ktu.is_ok(), "a fachada monta com portas fake");
    let Ok(ktu) = ktu else {
        return;
    };

    let mut log = SessionLog::new();
    let outcome = ktu.turn(&mut log, "le a.txt").await;
    assert!(outcome.is_ok());
    assert_eq!(
        outcome.map(|value| value.text).unwrap_or_default(),
        "li a.txt"
    );
    assert_eq!(tools.recorded_calls().len(), 1);
    assert_eq!(log.verify(), Vec::new(), "I1/I2 preservadas");

    // Persistência pela porta `Fs`: gravar e voltar a carregar dá o mesmo log.
    assert!(ktu.save_session("s1", &log).is_ok());
    let reloaded = ktu.session("s1");
    assert!(reloaded.is_ok());
    let reloaded = reloaded.unwrap_or_default();
    let original = store::to_jsonl(&log);
    assert!(original.is_ok());
    assert_eq!(
        reloaded.map(|value| store::to_jsonl(&value).unwrap_or_default()),
        original.ok()
    );
}

#[tokio::test]
async fn the_same_script_gives_byte_identical_logs() {
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let logger = NoopLogger;
    let metrics = NoopMetrics;
    let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
    let cancel = CancelToken::new();
    let registry = AgentRegistry::new(
        AgentSpec::new(AgentId::Primary, names(&["read_file"]), 1),
        None,
    );
    let mut rendered: Vec<String> = Vec::new();
    for _ in 0..2 {
        let fs = MemFs::new();
        let provider = FakeProvider::new(script());
        let tools = FakeTools::new(vec![spec("read_file")], vec![ToolOutcome::ok("conteudo")]);
        let ktu = build(
            &fs, &clock, &logger, &metrics, &provider, &tools, &registry, &budget, &cancel,
        );
        let Ok(ktu) = ktu else {
            return;
        };
        let mut log = SessionLog::new();
        let _ignored = ktu.turn(&mut log, "le a.txt").await;
        rendered.push(store::to_jsonl(&log).unwrap_or_default());
    }
    assert_eq!(rendered.len(), 2);
    assert_eq!(
        rendered.first(),
        rendered.last(),
        "I6: replay deterministico"
    );
}

#[tokio::test]
async fn the_injected_cancel_token_stops_the_turn() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let logger = NoopLogger;
    let metrics = NoopMetrics;
    let provider = FakeProvider::new(script());
    let tools = FakeTools::new(vec![spec("read_file")], vec![ToolOutcome::ok("conteudo")]);
    let registry = AgentRegistry::new(
        AgentSpec::new(AgentId::Primary, names(&["read_file"]), 1),
        None,
    );
    let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
    let cancel = CancelToken::new();
    cancel.cancel();

    let ktu = build(
        &fs, &clock, &logger, &metrics, &provider, &tools, &registry, &budget, &cancel,
    );
    let Ok(ktu) = ktu else {
        return;
    };
    let mut log = SessionLog::new();
    let outcome = ktu.turn(&mut log, "le a.txt").await;
    assert_eq!(
        outcome.map_or(StopReason::Error, |value| value.reason),
        StopReason::Cancelled
    );
    assert!(matches!(
        log.events().first(),
        Some(Event::TurnStart { .. })
    ));
}
