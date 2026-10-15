//! Integração do loop com o provider e as tools falsas (sem modelo real).

use std::sync::{Mutex, PoisonError};

use ktu_core::agent::{Deps, LoopConfig, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Completion, StopReason, ToolCall, ToolCallId, ToolSpec};
use ktu_core::events;
use ktu_core::guard::StructuralGuard;
use ktu_core::memory::{Hint, Memory, MemoryDraft, MemoryResult, MemoryWrite};
use ktu_core::ports::{FixedClock, NoopMetrics, RecordingLogger, Timestamp};
use ktu_core::session::{Event, SessionLog};
use ktu_core::testing::{FakeProvider, FakeTools};
use ktu_core::tools::ToolOutcome;

fn call(id: &str, name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new(id),
        name: name.to_owned(),
        arguments: arguments.to_owned(),
    }
}

fn spec(name: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_owned(),
        description: String::new(),
        parameters: "{}".to_owned(),
    }
}

fn tools(names: &[&str], responses: Vec<ToolOutcome>) -> FakeTools {
    FakeTools::new(names.iter().map(|name| spec(name)).collect(), responses)
}

fn read_guard() -> StructuralGuard {
    StructuralGuard::new("/r", ["read_file".to_owned()], 1)
}

fn has_denied_decision(log: &SessionLog) -> bool {
    log.events()
        .iter()
        .any(|event| matches!(event, Event::GuardDecision { allowed, .. } if !allowed))
}

#[tokio::test]
async fn loop_runs_a_tool_then_finishes() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "read_file", r#"{"path":"a.txt"}"#)],
        },
        Completion {
            text: "feito".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&["read_file"], vec![ToolOutcome::ok("conteudo")]);
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: None,
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "le a.txt", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.verify(), Vec::new());
    assert_eq!(executor.recorded_calls().len(), 1);
    assert!(!has_denied_decision(&log));
}

#[tokio::test]
async fn guard_denies_a_tool_outside_the_spec() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "write_file", r#"{"path":"a.txt"}"#)],
        },
        Completion {
            text: "ok".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&["read_file"], vec![ToolOutcome::ok("nao usado")]);
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: None,
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "escreve", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.verify(), Vec::new());
    assert_eq!(executor.recorded_calls(), Vec::new());
    assert!(has_denied_decision(&log));
}

#[tokio::test]
async fn cancel_before_the_first_step_closes_the_turn() {
    let provider = FakeProvider::new(vec![Completion {
        text: "nao devia correr".to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    }]);
    let executor = tools(&["read_file"], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    cancel.cancel();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: None,
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "para", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::Cancelled);
    assert_eq!(log.verify(), Vec::new());
}

#[tokio::test]
async fn repeated_calls_are_detected_as_stagnation() {
    let repeated = || Completion {
        text: String::new(),
        reasoning: String::new(),
        tool_calls: vec![call("1", "read_file", r#"{"path":"a.txt"}"#)],
    };
    let provider = FakeProvider::new(vec![repeated(), repeated(), repeated(), repeated()]);
    let executor = tools(
        &["read_file"],
        vec![
            ToolOutcome::ok("a"),
            ToolOutcome::ok("a"),
            ToolOutcome::ok("a"),
        ],
    );
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: None,
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_steps: 10,
        max_stagnation: 2,
    };

    let reason = run_turn(&mut log, &deps, "sys", "rele", &config).await;

    assert_eq!(reason, StopReason::Stalled);
    assert_eq!(log.verify(), Vec::new());
}

#[tokio::test]
async fn distinct_calls_exhaust_the_step_cap() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "read_file", r#"{"path":"f0.txt"}"#)],
        },
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("2", "read_file", r#"{"path":"f1.txt"}"#)],
        },
    ]);
    let executor = tools(
        &["read_file"],
        vec![ToolOutcome::ok("0"), ToolOutcome::ok("1")],
    );
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: None,
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_steps: 2,
        max_stagnation: 5,
    };

    let reason = run_turn(&mut log, &deps, "sys", "duas", &config).await;

    assert_eq!(reason, StopReason::MaxSteps);
    assert_eq!(log.verify(), Vec::new());
}

/// Memória falsa que registra as consultas de recall (para testar o gate).
#[derive(Default)]
struct CountingMemory {
    queries: Mutex<Vec<String>>,
    drafts: Mutex<Vec<String>>,
}

impl CountingMemory {
    fn queries(&self) -> Vec<String> {
        self.queries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn drafts(&self) -> Vec<String> {
        self.drafts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Memory for CountingMemory {
    fn recall(&self, query: &str, _limit: usize) -> MemoryResult<Vec<Hint>> {
        self.queries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(query.to_owned());
        Ok(vec![Hint::new("fact_01", "ha memoria relevante", 0.9)])
    }

    fn write(&self, draft: &MemoryDraft) -> MemoryResult<MemoryWrite> {
        self.drafts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(draft.statement.clone());
        Ok(MemoryWrite::Created {
            id: "fact_01".to_owned(),
        })
    }
}

#[tokio::test]
async fn memory_is_recalled_only_at_session_start() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: "primeiro".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
        Completion {
            text: "segundo".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&[], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let memory = CountingMemory::default();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: Some(&memory),
    };
    let mut log = SessionLog::new();

    let _first = run_turn(
        &mut log,
        &deps,
        "sys",
        "primeira pergunta",
        &LoopConfig::default(),
    )
    .await;
    let _second = run_turn(
        &mut log,
        &deps,
        "sys",
        "segunda pergunta",
        &LoopConfig::default(),
    )
    .await;

    let queries = memory.queries();
    assert_eq!(queries, vec!["primeira pergunta".to_owned()]);
    let recalls = logger
        .snapshot()
        .into_iter()
        .filter(|record| record.event == events::MEMORY_RECALL)
        .count();
    assert_eq!(recalls, 1);
}

#[tokio::test]
async fn memory_is_written_automatically_at_turn_end() {
    let provider = FakeProvider::new(vec![Completion {
        text: "a resposta final do turno".to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    }]);
    let executor = tools(&[], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let memory = CountingMemory::default();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: Some(&memory),
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "faz algo", &LoopConfig::default()).await;
    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(
        memory.drafts(),
        vec!["a resposta final do turno".to_owned()]
    );
    let writes = logger
        .snapshot()
        .into_iter()
        .filter(|record| record.event == events::MEMORY_WRITE)
        .count();
    assert_eq!(writes, 1);
}

#[tokio::test]
async fn a_tight_budget_compacts_instead_of_failing() {
    let mut log = SessionLog::new();
    for index in 0..10 {
        log.push(Event::TurnStart {
            prompt: format!("turno {index}"),
        });
        log.push(Event::Assistant {
            content: "y".repeat(300),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        });
        log.push(Event::TurnEnd {
            reason: StopReason::EndTurn,
        });
    }

    let provider = FakeProvider::new(vec![Completion {
        text: "ok".to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    }]);
    let executor = tools(&[], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(500);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        provider: &provider,
        tools: &executor,
        guard: &guard,
        budget: &budget,
        logger: &logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: None,
    };

    let reason = run_turn(
        &mut log,
        &deps,
        "sys",
        "nova pergunta",
        &LoopConfig::default(),
    )
    .await;

    assert_eq!(reason, StopReason::EndTurn);
    assert!(log.is_compacted());
    let compacted = logger
        .snapshot()
        .into_iter()
        .any(|record| record.event == events::SESSION_COMPACT);
    assert!(compacted);
}
