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

/// Raiz temporária única por teste.
fn fresh_root() -> PathBuf {
    std::env::temp_dir().join(format!("ktu-serve-{}", std::process::id()))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_port_drives_a_session() -> Result<(), Box<dyn std::error::Error>> {
    let port = free_port()?;
    let root = fresh_root();
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
