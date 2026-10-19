//! Integração do loop com o provider e as tools falsas (sem modelo real).

use async_trait::async_trait;
use std::sync::{Mutex, PoisonError};

use ktu_core::agent::{Capabilities, Deps, LoopConfig, Ports, Steering, Telemetry, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Completion, Message, StopReason, ToolCall, ToolCallId, ToolSpec};
use ktu_core::events;
use ktu_core::guard::StructuralGuard;
use ktu_core::memory::{Hint, Memory, MemoryDraft, MemoryResult, MemoryWrite};
use ktu_core::plugin::{HookPayload, HookVerdict, Hooks, NoHooks};
use ktu_core::ports::{FixedClock, MapInstructions, NoopMetrics, RecordingLogger, Timestamp};
use ktu_core::session::{Event, SessionLog, SteerMessage};
use ktu_core::subagent::{SubagentHost, SubagentReport, SubagentRequest};
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "escreve", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.verify(), Vec::new());
    assert_eq!(executor.recorded_calls(), Vec::new());
    assert!(has_denied_decision(&log));
    let records = logger.snapshot();
    assert!(
        records
            .iter()
            .any(|record| record.event == events::GUARD_DECISION),
        "guard.decision deve ir pela porta Logger"
    );
    assert!(
        records
            .iter()
            .any(|record| record.event == events::TOOL_RESULT
                && record
                    .fields
                    .iter()
                    .any(|(key, value)| key.as_str() == "is_error" && value.as_str() == "true")),
        "tool.result (negado) deve ir pela porta Logger com is_error=true"
    );
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_steps: 10,
        max_stagnation: 2,
        ..LoopConfig::default()
    };

    let reason = run_turn(&mut log, &deps, "sys", "rele", &config).await;

    assert_eq!(reason, StopReason::Stalled);
    assert_eq!(log.verify(), Vec::new());
    assert!(
        logger
            .snapshot()
            .iter()
            .any(|record| record.event == events::TOOL_RESULT),
        "tool.result sintetico (estagnacao) deve ir pela porta Logger"
    );
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_steps: 2,
        max_stagnation: 5,
        ..LoopConfig::default()
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: Some(&memory),
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: Some(&memory),
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
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
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
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

/// Host de subagentes falso: regista os pedidos e devolve um relatório fixo.
#[derive(Default)]
struct RecordingSubagentHost {
    requests: Mutex<Vec<SubagentRequest>>,
}

impl RecordingSubagentHost {
    fn requests(&self) -> Vec<SubagentRequest> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl SubagentHost for RecordingSubagentHost {
    async fn run(&self, request: SubagentRequest) -> Result<SubagentReport, String> {
        self.requests
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(request);
        Ok(SubagentReport {
            session_id: "ktu-sub-1".to_owned(),
            text: "ABCDEFGHIJKLMNOPQRST".to_owned(),
        })
    }
}

#[tokio::test]
async fn subagent_delegation_returns_a_capped_report() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call(
                "1",
                "subagent",
                r#"{"agent":"explorer","description":"mapear","prompt":"lista"}"#,
            )],
        },
        Completion {
            text: "fim".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&[], Vec::new());
    let guard = StructuralGuard::new("/r", ["subagent".to_owned()], 1);
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let host = RecordingSubagentHost::default();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: Some(&host),
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        subagent_report_chars: 8,
        ..LoopConfig::default()
    };

    let reason = run_turn(&mut log, &deps, "sys", "delega", &config).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.verify(), Vec::new());
    let requests = host.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests.first().map(|r| r.agent.as_str()), Some("explorer"));
    let report = log.events().iter().find_map(|event| match event {
        Event::ToolResult { name, content, .. } if name == "subagent" => Some(content.clone()),
        _ => None,
    });
    assert!(report.is_some_and(|text| text.contains("truncado") && !text.contains("IJ")));
}

/// Host que ecoa o prompt — útil para verificar a ordem dos resultados.
struct EchoSubagentHost;

#[async_trait]
impl SubagentHost for EchoSubagentHost {
    async fn run(&self, request: SubagentRequest) -> Result<SubagentReport, String> {
        Ok(SubagentReport {
            session_id: format!("sub-{}", request.prompt),
            text: request.prompt,
        })
    }
}

#[tokio::test]
async fn fan_out_runs_subagents_and_keeps_result_order() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![
                call("1", "subagent", r#"{"agent":"explorer","prompt":"alpha"}"#),
                call("2", "subagent", r#"{"agent":"explorer","prompt":"beta"}"#),
            ],
        },
        Completion {
            text: "fim".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&[], Vec::new());
    let guard = StructuralGuard::new("/r", ["subagent".to_owned()], 1);
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let host = EchoSubagentHost;
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: Some(&host),
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(
        &mut log,
        &deps,
        "sys",
        "delega dois",
        &LoopConfig::default(),
    )
    .await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.verify(), Vec::new());
    let results: Vec<String> = log
        .events()
        .iter()
        .filter_map(|event| match event {
            Event::ToolResult { name, content, .. } if name == "subagent" => Some(content.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(results, vec!["alpha".to_owned(), "beta".to_owned()]);
}

#[tokio::test]
async fn fan_out_respects_the_per_turn_cap() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![
                call("1", "subagent", r#"{"agent":"explorer","prompt":"a"}"#),
                call("2", "subagent", r#"{"agent":"explorer","prompt":"b"}"#),
            ],
        },
        Completion {
            text: "fim".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&[], Vec::new());
    let guard = StructuralGuard::new("/r", ["subagent".to_owned()], 1);
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let host = EchoSubagentHost;
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: Some(&host),
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_subagents_per_turn: 1,
        ..LoopConfig::default()
    };

    let reason = run_turn(&mut log, &deps, "sys", "delega dois", &config).await;

    assert_eq!(reason, StopReason::EndTurn);
    let capped = log.events().iter().any(|event| {
        matches!(
            event,
            Event::ToolResult { content, .. } if content.contains("limite de subagentes")
        )
    });
    assert!(capped);
}

/// *Hooks* de teste: nega `bash` e permite tudo o resto (I11).
struct DenyBashHooks;

#[async_trait]
impl Hooks for DenyBashHooks {
    async fn dispatch(&self, payload: &mut HookPayload) -> HookVerdict {
        if let HookPayload::ActionBefore { tool, .. } = payload
            && tool == "bash"
        {
            return HookVerdict {
                allowed: false,
                reason: Some("plugin_negou_bash".to_owned()),
                generation: 1,
                transforms: 0,
            };
        }
        HookVerdict::pass(1)
    }

    fn generation(&self) -> u32 {
        1
    }
}

#[tokio::test]
async fn plugin_gate_denies_and_the_denial_is_evidence() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "bash", r#"{"command":"echo oi"}"#)],
        },
        Completion {
            text: "fim".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&["bash"], vec![ToolOutcome::ok("nao devia correr")]);
    let guard = StructuralGuard::new("/r", ["bash".to_owned()], 1);
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let hooks = DenyBashHooks;
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: Some(&hooks),
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "corre bash", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(executor.recorded_calls().len(), 0);
    let denied = log.events().iter().any(|event| {
        matches!(
            event,
            Event::GuardDecision { allowed: false, reason, .. } if reason.contains("plugin")
        )
    });
    assert!(denied);
    let dispatched = log.events().iter().any(|event| {
        matches!(
            event,
            Event::PluginDispatch { hook, allowed: false, .. } if hook == "action.before"
        )
    });
    assert!(dispatched);
}

#[tokio::test]
async fn a_plugin_cannot_free_a_structural_deny() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "write_file", r#"{"path":"x"}"#)],
        },
        Completion {
            text: "fim".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(&["write_file"], vec![ToolOutcome::ok("nao devia correr")]);
    // O guard só permite `read_file`; `write_file` é negado por capacidade.
    let guard = StructuralGuard::new("/r", ["read_file".to_owned()], 1);
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: Some(&NoHooks),
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "escreve x", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(executor.recorded_calls().len(), 0);
    let denied = log.events().iter().any(|event| {
        matches!(
            event,
            Event::GuardDecision { allowed: false, reason, .. } if reason.contains("capability")
        )
    });
    assert!(denied);
    // O *gate* de plugin **não** é consultado depois de um `Deny` estrutural (I11).
    let consulted = log.events().iter().any(
        |event| matches!(event, Event::PluginDispatch { hook, .. } if hook == "action.before"),
    );
    assert!(!consulted);
}

/// Passo do modelo que pede uma tool.
fn tool_step(id: &str) -> Completion {
    Completion {
        text: String::new(),
        reasoning: String::new(),
        tool_calls: vec![call(id, "read_file", r#"{"path":"a.txt"}"#)],
    }
}

/// Passo do modelo com texto final.
fn text_step(text: &str) -> Completion {
    Completion {
        text: text.to_owned(),
        reasoning: String::new(),
        tool_calls: Vec::new(),
    }
}

#[tokio::test]
async fn max_steps_ends_with_a_textual_step_without_tools() {
    let provider = FakeProvider::new(vec![
        tool_step("1"),
        tool_step("2"),
        text_step("resumo: feito a, falta b"),
    ]);
    let executor = tools(
        &["read_file"],
        vec![ToolOutcome::ok("c1"), ToolOutcome::ok("c2")],
    );
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_steps: 3,
        ..LoopConfig::default()
    };

    let reason = run_turn(&mut log, &deps, "sys", "faz", &config).await;

    assert_eq!(reason, StopReason::MaxSteps);
    // O passo final (3.º) é pedido com zero tools.
    assert_eq!(provider.tool_counts(), vec![1, 1, 0]);
    assert_eq!(executor.recorded_calls().len(), 2);
    assert_eq!(log.verify(), Vec::new());
}

#[tokio::test]
async fn the_final_step_never_executes_tools() {
    let provider = FakeProvider::new(vec![tool_step("1"), tool_step("2"), tool_step("3")]);
    let executor = tools(
        &["read_file"],
        vec![ToolOutcome::ok("c1"), ToolOutcome::ok("c2")],
    );
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();
    let config = LoopConfig {
        max_steps: 3,
        ..LoopConfig::default()
    };

    let reason = run_turn(&mut log, &deps, "sys", "faz", &config).await;

    assert_eq!(reason, StopReason::MaxSteps);
    // A 3.ª chamada (passo final) nunca é executada.
    assert_eq!(executor.recorded_calls().len(), 2);
    assert_eq!(log.verify(), Vec::new());
}

#[tokio::test]
async fn echo_is_never_accepted() {
    let provider = FakeProvider::new(vec![text_step("le a.txt"), text_step("le a.txt")]);
    let executor = tools(&["read_file"], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "le a.txt", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::Echo);
    let last = log
        .events()
        .iter()
        .rev()
        .find_map(|event| match event {
            Event::Assistant { content, .. } => Some(content.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(last.contains("eco"), "{last}");
}

#[tokio::test]
async fn a_legit_answer_is_not_an_echo() {
    let provider = FakeProvider::new(vec![text_step("O ficheiro a.txt tem tres linhas.")]);
    let executor = tools(&["read_file"], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "le a.txt", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
}

#[tokio::test]
async fn declared_tool_calls_are_decoded_and_executed() {
    let provider = FakeProvider::new(vec![
        text_step(r#"{"name":"read_file","arguments":{"path":"a.txt"}}"#),
        text_step("feito"),
    ]);
    let executor = tools(&["read_file"], vec![ToolOutcome::ok("conteudo")]);
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "le a.txt", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(executor.recorded_calls().len(), 1);
    // O texto declarado **nao** entra no log como resposta do assistente.
    let leaked = log.events().iter().any(
        |event| matches!(event, Event::Assistant { content, .. } if content.contains("\"name\"")),
    );
    assert!(!leaked);
}

/// Fonte de *steering* de teste: devolve as mensagens dadas, uma só vez.
struct FakeSteering {
    messages: Mutex<Vec<SteerMessage>>,
}

impl FakeSteering {
    fn new(messages: &[&str]) -> Self {
        Self {
            messages: Mutex::new(
                messages
                    .iter()
                    .enumerate()
                    .map(|(index, text)| SteerMessage {
                        id: format!("s{index}"),
                        text: (*text).to_owned(),
                    })
                    .collect(),
            ),
        }
    }
}

impl Steering for FakeSteering {
    fn take(&self) -> Vec<SteerMessage> {
        std::mem::take(&mut *self.messages.lock().unwrap_or_else(PoisonError::into_inner))
    }
}

#[tokio::test]
async fn steering_is_logged_and_delivered() {
    let provider = FakeProvider::new(vec![tool_step("1"), text_step("feito")]);
    let executor = tools(&["read_file"], vec![ToolOutcome::ok("c1")]);
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let steering = FakeSteering::new(&["muda de plano"]);
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: Some(&steering),
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "faz", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    // A mensagem é **admitida** (`SteerQueued`) e depois **entregue** (`Steer`) — M7.
    let admitted = log
        .events()
        .iter()
        .any(|event| matches!(event, Event::SteerQueued { text, .. } if text == "muda de plano"));
    assert!(admitted, "admissao registada no log");
    let in_log = log
        .events()
        .iter()
        .any(|event| matches!(event, Event::Steer { text, .. } if text == "muda de plano"));
    assert!(in_log, "entrega registada no log");
    // A admissão vem **antes** da entrega (durável antes de entrar no contexto).
    let admitted_at = log.events().iter().position(
        |event| matches!(event, Event::SteerQueued { text, .. } if text == "muda de plano"),
    );
    let delivered_at = log
        .events()
        .iter()
        .position(|event| matches!(event, Event::Steer { text, .. } if text == "muda de plano"));
    assert!(
        admitted_at < delivered_at,
        "admissao antes da entrega: {admitted_at:?} vs {delivered_at:?}"
    );
    assert_eq!(log.pending_steers(), Vec::new(), "nada fica pendente");
    let in_context = log
        .derive_messages()
        .iter()
        .any(|message| matches!(message, Message::User { content } if content == "muda de plano"));
    assert!(in_context, "model-visible <=> logged");
    assert_eq!(log.verify(), Vec::new());
}

/// M7: uma admissão que fica pendente no fim de um turno é entregue no **turno seguinte**
/// (o mesmo caminho que recupera uma admissão que sobreviveu a um *crash*).
#[tokio::test]
async fn a_pending_steer_is_delivered_in_the_next_turn() {
    let provider = FakeProvider::new(vec![text_step("feito"), text_step("feito de novo")]);
    let executor = tools(&[], Vec::new());
    let guard = read_guard();
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let steering = FakeSteering::new(&["muda de plano"]);
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: Some(&steering),
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    // Turno de **um** passo: a mensagem é admitida na única fronteira e fica pendente.
    let reason = run_turn(&mut log, &deps, "sys", "faz", &LoopConfig::default()).await;
    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.pending_steers().len(), 1, "admissao fica pendente");
    assert!(
        !log.events()
            .iter()
            .any(|event| matches!(event, Event::Steer { .. })),
        "sem entrega no mesmo turno"
    );

    // Turno seguinte (mesmo log): a pendente entra na **primeira** fronteira.
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let reason = run_turn(&mut log, &deps, "sys", "continua", &LoopConfig::default()).await;
    assert_eq!(reason, StopReason::EndTurn);
    assert_eq!(log.pending_steers(), Vec::new(), "entregue uma vez");
    let count = log
        .derive_messages()
        .iter()
        .filter(
            |message| matches!(message, Message::User { content } if content == "muda de plano"),
        )
        .count();
    assert_eq!(count, 1, "entra no contexto uma so vez");
    assert_eq!(log.verify(), Vec::new());
}

#[tokio::test]
async fn tool_search_promotes_a_deferred_tool_to_the_next_request() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("1", "tool_search", r#"{"query":"procura padrao"}"#)],
        },
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("2", "grep", r#"{"pattern":"x"}"#)],
        },
        Completion {
            text: "feito".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let deferred = ToolSpec {
        name: "grep".to_owned(),
        description: "Procura um padrao no conteudo dos ficheiros".to_owned(),
        parameters: r#"{"type":"object","properties":{"pattern":{"type":"string"}}}"#.to_owned(),
    };
    let executor =
        tools(&["read_file"], vec![ToolOutcome::ok("achado")]).with_deferred(vec![deferred]);
    // O conjunto do agente é a capacidade (inclui as diferidas e o `tool_search`).
    let guard = StructuralGuard::new(
        "/r",
        [
            "read_file".to_owned(),
            "grep".to_owned(),
            "tool_search".to_owned(),
        ],
        1,
    );
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: None,
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "procura", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    // A tool promovida foi executada (o guard conhecia-a) — e só ela.
    let called: Vec<String> = executor
        .recorded_calls()
        .into_iter()
        .map(|call| call.name)
        .collect();
    assert_eq!(called, ["grep"]);
    // Catálogo por passo: `read_file` + `tool_search`; depois + `grep` (promovida).
    assert_eq!(provider.tool_counts(), vec![2, 3, 3]);
    // O carregamento nunca é silencioso.
    assert!(
        logger
            .snapshot()
            .iter()
            .any(|record| record.event == events::TOOL_LOADED),
        "tool.loaded deve ir pela porta Logger"
    );
    assert_eq!(log.verify(), Vec::new());
}

#[tokio::test]
async fn local_instructions_are_attached_once_per_source() {
    let provider = FakeProvider::new(vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![
                call("1", "read_file", r#"{"path":"sub/a.md"}"#),
                call("2", "read_file", r#"{"path":"sub/b.md"}"#),
                call("3", "read_file", r#"{"path":"outro.md"}"#),
            ],
        },
        Completion {
            text: "feito".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        },
    ]);
    let executor = tools(
        &["read_file"],
        vec![
            ToolOutcome::ok("a"),
            ToolOutcome::ok("b"),
            ToolOutcome::ok("c"),
        ],
    );
    let guard = StructuralGuard::new("/r", ["read_file".to_owned()], 1);
    let budget = BudgetGate::new(100_000);
    let logger = RecordingLogger::new();
    let metrics = NoopMetrics;
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let cancel = CancelToken::new();
    let instructions = MapInstructions::new()
        .with_dir("", "AGENTS.md", "regra global")
        .with_dir("sub/", "sub/AGENTS.md", "regra local");
    let deps = Deps {
        ports: Ports {
            provider: &provider,
            tools: &executor,
            guard: &guard,
            budget: &budget,
            cancel: &cancel,
        },
        telemetry: Telemetry {
            logger: &logger,
            metrics: &metrics,
            clock: &clock,
        },
        capabilities: Capabilities {
            memory: None,
            subagents: None,
            plugins: None,
            steering: None,
            instructions: Some(&instructions),
        },
    };
    let mut log = SessionLog::new();

    let reason = run_turn(&mut log, &deps, "sys", "le", &LoopConfig::default()).await;

    assert_eq!(reason, StopReason::EndTurn);
    let results: Vec<String> = log
        .events()
        .iter()
        .filter_map(|event| match event {
            Event::ToolResult { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(results.len(), 3);
    // `sub/AGENTS.md` entra uma vez (o mais próximo ganha); a segunda leitura em
    // `sub/` não o repete; a terceira (fora de `sub/`) traz o da raiz.
    assert!(
        results
            .first()
            .is_some_and(|text| text.contains("regra local"))
    );
    assert!(
        results
            .get(1)
            .is_some_and(|text| !text.contains("instrucoes locais")),
        "a segunda leitura nao repete as instrucoes"
    );
    assert!(
        results
            .get(2)
            .is_some_and(|text| text.contains("regra global"))
    );
    assert!(
        results
            .first()
            .is_some_and(|text| text.contains("[instrucoes locais: sub/AGENTS.md]"))
    );
    // O evento de contexto vai pela porta Logger (nunca em silêncio).
    assert_eq!(
        logger
            .snapshot()
            .iter()
            .filter(|record| record.event == events::CONTEXT_LOCAL)
            .count(),
        2
    );
    assert_eq!(log.verify(), Vec::new());
}
