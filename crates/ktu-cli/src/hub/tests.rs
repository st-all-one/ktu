//! Testes do serviço de sessões (actor + porta), com provider `mock`.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ktu_core::plan::{Approval, Mode};
use ktu_core::ports::NoopLogger;
use ktu_core::session::Event;
use ktu_port::host::Host;
use ktu_port::protocol::{PortError, ServerMsg};

use super::SessionHub;
use crate::run::RunConfig;
use crate::session_mode::{SessionMode, SessionTarget};
use crate::turn_runtime::ensure_layout;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn mock_config() -> RunConfig {
    RunConfig {
        provider: "mock".to_owned(),
        script: None,
        base_url: String::new(),
        model: "eco".to_owned(),
        api_key: None,
        effort: "low".to_owned(),
        max_steps: 4,
        deferred_tools: Vec::new(),
        session: SessionTarget::Persistent("default".to_owned()),
        mode: SessionMode::Fresh,
        plan: false,
    }
}

/// Raiz temporária única por teste.
fn fresh_root() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("ktu-hub-{}-{id}", std::process::id()))
}

/// Corre um turno e drena os eventos até `TurnFinished`.
async fn run_one_turn(
    hub: &SessionHub,
    session: &str,
    prompt: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut events = hub.subscribe(session);
    let _turn_id = hub.turn(session, prompt).await?;
    let mut logs = 0usize;
    while let Some(message) = events.recv().await {
        match message {
            ServerMsg::Log { .. } => logs = logs.saturating_add(1),
            ServerMsg::TurnFinished { .. } => break,
            _ => {}
        }
    }
    Ok(logs)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn serves_a_turn_and_publishes_events() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root();
    let _ignored = std::fs::remove_dir_all(&root);
    ensure_layout(&root)?;
    let hub = SessionHub::new(root.clone(), mock_config(), Arc::new(NoopLogger));

    let info = hub.create_session("s1").await?;
    assert_eq!(info.session_id, "s1");
    assert!(!info.resumed);

    let logs = run_one_turn(&hub, "s1", "ola").await?;
    assert!(logs > 0, "deve publicar eventos do log");

    let state = hub.state("s1").await?;
    assert_eq!(state.mode, Mode::Build);
    assert!(!state.pending);
    assert!(state.log.len() >= logs);

    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_mode_is_logged_and_persisted() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root();
    let _ignored = std::fs::remove_dir_all(&root);
    ensure_layout(&root)?;
    let hub = SessionHub::new(root.clone(), mock_config(), Arc::new(NoopLogger));

    hub.create_session("s1").await?;
    let _logs = run_one_turn(&hub, "s1", "primeiro").await?;

    hub.set_mode("s1", Mode::Plan, Approval::Pending).await?;
    let state = hub.state("s1").await?;
    assert_eq!(state.mode, Mode::Plan);
    assert!(matches!(state.log.last(), Some(event) if matches!(event, Event::ModeSet { .. })));

    // Uma sessão nova na mesma raiz retoma o log de disco (modo preservado).
    let resumed = SessionHub::new(root.clone(), mock_config(), Arc::new(NoopLogger));
    let info = resumed.create_session("s1").await?;
    assert!(info.resumed);
    assert_eq!(resumed.state("s1").await?.mode, Mode::Plan);

    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_on_unknown_session_is_not_found() {
    let root = fresh_root();
    let hub = SessionHub::new(root, mock_config(), Arc::new(NoopLogger));
    let error = hub.cancel("nao-existe").await;
    assert!(matches!(error, Err(PortError::NoSession(_))));
}
