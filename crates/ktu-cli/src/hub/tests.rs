//! Testes do serviço de sessões (actor + porta), com provider `mock`.

use std::fmt::Write as _;
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

/// Configuração com o provider `scripted` (um guião lido por turno).
fn scripted_config(script: PathBuf) -> RunConfig {
    RunConfig {
        provider: "scripted".to_owned(),
        script: Some(script),
        ..mock_config()
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

/// `/compact` corta o contexto, grava o checkpoint e o corte **sobrevive** a retomar.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn compact_is_persisted_and_restored_on_resume() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root();
    let _ignored = std::fs::remove_dir_all(&root);
    ensure_layout(&root)?;
    let hub = SessionHub::new(root.clone(), mock_config(), Arc::new(NoopLogger));

    hub.create_session("s1").await?;
    for prompt in ["um", "dois", "tres"] {
        let _logs = run_one_turn(&hub, "s1", prompt).await?;
    }
    assert_eq!(hub.state("s1").await?.keep_from, None);

    hub.compact("s1").await?;
    let keep_from = hub.state("s1").await?.keep_from;
    assert!(keep_from.is_some(), "o corte tem de ficar no estado");
    // O log cru fica intacto (a compactacao e uma projecao).
    assert_eq!(
        hub.state("s1")
            .await?
            .log
            .iter()
            .filter(|event| matches!(event, Event::TurnStart { .. }))
            .count(),
        3
    );

    // Uma sessao nova na mesma raiz retoma o corte do checkpoint.
    let resumed = SessionHub::new(root.clone(), mock_config(), Arc::new(NoopLogger));
    resumed.create_session("s1").await?;
    assert_eq!(resumed.state("s1").await?.keep_from, keep_from);

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

/// A fila de mutação é **do processo**: duas sessões (cada uma com a sua T2) editam o
/// mesmo ficheiro pelo **mesmo** `FileMutationQueue`.
///
/// O `ktu serve` corre um actor por sessão; uma fila por turno não serializaria nada
/// entre elas. A prova é a chave: uma só entrada na fila do hub.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_hub_shares_one_mutation_queue_across_sessions()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root();
    let _ignored = std::fs::remove_dir_all(&root);
    ensure_layout(&root)?;
    let mut initial = String::new();
    for index in 0..8 {
        let _ignored = writeln!(initial, "marca-{index}");
    }
    std::fs::write(root.join("alvo.txt"), &initial)?;
    // Dois turnos (um por sessão), cada um a editar uma âncora distinta.
    let mut script = String::new();
    for index in 0..2 {
        let arguments = serde_json::json!({
            "path": "alvo.txt",
            "edits": [{ "old": format!("marca-{index}\n"), "new": format!("editada-{index}\n") }],
        });
        let call = serde_json::json!({
            "text": "",
            "tool_calls": [{ "id": "c1", "name": "edit_file", "arguments": arguments.to_string() }],
        });
        let _ignored = writeln!(script, "{call}\n{{\"text\":\"fim\"}}");
    }
    let script_path = root.join("guiao.jsonl");
    std::fs::write(&script_path, script)?;
    let hub = SessionHub::new(
        root.clone(),
        scripted_config(script_path),
        Arc::new(NoopLogger),
    );

    for session in ["s1", "s2"] {
        hub.create_session(session).await?;
        let _logs = run_one_turn(&hub, session, "edita").await?;
    }

    let after = std::fs::read_to_string(root.join("alvo.txt"))?;
    assert!(after.contains("editada-0\n"), "{after}");
    assert!(after.contains("editada-1\n"), "{after}");
    assert_eq!(
        hub.inner.mutations.tracked(),
        1,
        "as duas sessoes nao usaram a fila do processo"
    );
    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}
