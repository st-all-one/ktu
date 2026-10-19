//! Integração da porta HTTP: sobe o binário `ktu serve` e fala via [`HttpPort`].
//!
//! Prova o caminho completo (servidor + SSE + `HttpPort`) sem tocar no kernel.

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use ktu_port::HttpPort;
use ktu_port::port::Port;
use ktu_port::protocol::ServerMsg;
use tokio::time::sleep;

/// Mata o servidor no fim (mesmo com falha).
struct ServerGuard(Child);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ignored = self.0.kill();
        let _ignored = self.0.wait();
    }
}

/// Reserva uma porta livre (com corrida mínima aceitável num teste local).
fn free_port() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// Raiz temporária única por teste (o processo é partilhado entre testes).
fn fresh_root(tag: &str) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("ktu-serve-{tag}-{}-{id}", std::process::id()))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_port_drives_a_session() -> Result<(), Box<dyn std::error::Error>> {
    let port = free_port()?;
    let root = fresh_root("drive");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;

    let child = Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args([
            "serve",
            "--provider",
            "mock",
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--root",
        ])
        .arg(&root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let _guard = ServerGuard(child);

    let client = HttpPort::new(format!("http://127.0.0.1:{port}"));
    let mut ready = false;
    for _ in 0..50 {
        if client.health().await.is_ok() {
            ready = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(ready, "servidor nao arrancou");
    assert_eq!(client.health().await?.provider, "mock");

    let info = client.create_session("s1").await?;
    assert_eq!(info.session_id, "s1");
    assert!(!info.resumed);

    let mut events = client.subscribe("s1");
    let turn_id = client.turn("s1", "ola").await?;
    assert_eq!(turn_id, 1);

    let mut finished = false;
    while let Some(message) = events.recv().await {
        if matches!(message, ServerMsg::TurnFinished { .. }) {
            finished = true;
            break;
        }
    }
    assert!(finished, "turno deve terminar");

    let state = client.state("s1").await?;
    assert!(!state.log.is_empty(), "o log deve ter eventos apos o turno");
    assert!(!state.pending);

    client.cancel("s1").await?;
    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}

/// M7 pela porta HTTP: o `id` é do **cliente**; repetir o mesmo `id` não duplica a
/// mensagem, e a admissão fica no log **antes** da entrega.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_steering_is_admitted_once_per_id() -> Result<(), Box<dyn std::error::Error>> {
    let port = free_port()?;
    let root = fresh_root("steer");
    let _ignored = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root)?;

    let child = Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args([
            "serve",
            "--provider",
            "mock",
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--root",
        ])
        .arg(&root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let _guard = ServerGuard(child);

    let client = HttpPort::new(format!("http://127.0.0.1:{port}"));
    let mut ready = false;
    for _ in 0..50 {
        if client.health().await.is_ok() {
            ready = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(ready, "servidor nao arrancou");

    let _ignored = client.create_session("s1").await?;
    // O mesmo id duas vezes (retry do cliente): uma só mensagem.
    client.steer("s1", "h1", "faz antes X").await?;
    client.steer("s1", "h1", "faz antes X").await?;

    let mut events = client.subscribe("s1");
    let _ignored = client.turn("s1", "ola").await?;
    let mut finished = false;
    while let Some(message) = events.recv().await {
        if matches!(message, ServerMsg::TurnFinished { .. }) {
            finished = true;
            break;
        }
    }
    assert!(finished, "turno deve terminar");

    let log = std::fs::read_to_string(root.join(".ktu/sessions/s1.jsonl"))?;
    assert_eq!(
        log.matches("\"steer_queued\"").count(),
        1,
        "admissao uma so vez: {log}"
    );
    assert!(log.contains("\"h1\""), "{log}");
    assert!(log.contains("\"event\":\"steer\""), "entregue: {log}");
    let _ignored = std::fs::remove_dir_all(&root);
    Ok(())
}
