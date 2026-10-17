//! Integração do loop com o provider e as tools falsas (sem modelo real).

use async_trait::async_trait;
use std::sync::{Mutex, PoisonError};

use ktu_core::agent::{Capabilities, Deps, LoopConfig, Ports, Telemetry, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Completion, StopReason, ToolCall, ToolCallId, ToolSpec};
use ktu_core::events;
use ktu_core::guard::StructuralGuard;
use ktu_core::memory::{Hint, Memory, MemoryDraft, MemoryResult, MemoryWrite};
use ktu_core::plugin::{HookPayload, HookVerdict, Hooks, NoHooks};
use ktu_core::ports::{FixedClock, NoopMetrics, RecordingLogger, Timestamp};
use ktu_core::session::{Event, SessionLog};
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
