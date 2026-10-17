//! T2: as tools correm numa thread dedicada; o kernel fala por mensagens (I8).

use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, ThreadId};

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolCallId, ToolSpec};
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use ktu_tools::ToolThread;

fn call(name: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("c1"),
        name: name.to_owned(),
        arguments: "{}".to_owned(),
    }
}

/// Executor de sonda: regista a thread em que corre e pode falhar.
#[derive(Default)]
struct Probe {
    threads: Mutex<Vec<ThreadId>>,
    fail: bool,
}

impl Probe {
    fn threads(&self) -> Vec<ThreadId> {
        self.threads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl ToolExecutor for Probe {
    fn specs(&self) -> Vec<ToolSpec> {
        vec![ToolSpec {
            name: "probe".to_owned(),
            description: String::new(),
            parameters: "{}".to_owned(),
        }]
    }

    async fn execute(&self, _call: &ToolCall) -> ToolOutcome {
        self.threads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(thread::current().id());
        if self.fail {
            ToolOutcome::error("falha controlada")
        } else {
            ToolOutcome::ok("ok")
        }
    }
}

fn spawn(probe: Arc<Probe>) -> std::io::Result<ToolThread> {
    ToolThread::spawn(move |ctx| ctx.serve(probe.as_ref()))
}

#[tokio::test]
async fn tools_run_on_a_dedicated_thread() -> std::io::Result<()> {
    let probe = Arc::new(Probe::default());
    let tools = spawn(Arc::clone(&probe))?;
    let kernel_thread = thread::current().id();

    assert_eq!(tools.specs().len(), 1);
    let outcome = tools.execute(&call("probe")).await;

    assert!(!outcome.is_error);
    let tool_threads = probe.threads();
    assert_eq!(tool_threads.len(), 1);
    assert_ne!(tool_threads.first().copied(), Some(kernel_thread));
    Ok(())
}

#[tokio::test]
async fn tool_errors_are_contained_and_returned() -> std::io::Result<()> {
    let probe = Arc::new(Probe {
        threads: Mutex::new(Vec::new()),
        fail: true,
    });
    let tools = spawn(Arc::clone(&probe))?;

    let outcome = tools.execute(&call("probe")).await;
    assert!(outcome.is_error);
    assert!(outcome.content.contains("falha controlada"));
    // A T2 continua viva para o próximo pedido.
    assert_eq!(probe.threads().len(), 1);
    let again = tools.execute(&call("probe")).await;
    assert!(again.is_error);
    assert_eq!(probe.threads().len(), 2);
    Ok(())
}
