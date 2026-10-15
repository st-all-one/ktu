//! *Test double* HTTP: um servidor TCP mínimo que replica bytes de fixtures.
//!
//! Exercita o caminho **real** do provider (`reqwest` + SSE + acumulador) sem
//! um modelo: o servidor escreve uma resposta HTTP gravada e fecha.

use std::io;

use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Message, ToolSpec};
use ktu_core::provider::{Provider, ProviderRequest};
use ktu_provider::{LlamaConfig, LlamaProvider};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const CHAT_STREAM: &str = include_str!("fixtures/chat_stream.sse");

/// Levanta um servidor que responde `status` na primeira ligação e 200 nas
/// seguintes, copiando `body` (bytes de fixture). Devolve o *base URL*.
async fn serve(statuses: Vec<u16>, body: &'static str) -> io::Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        for status in statuses {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let mut buffer = [0_u8; 4096];
            let _read = socket.read(&mut buffer).await;
            let reason = if status == 200 { "OK" } else { "Server Error" };
            let header = format!(
                "HTTP/1.1 {status} {reason}\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n"
            );
            let _header = socket.write_all(header.as_bytes()).await;
            if status == 200 {
                let _body = socket.write_all(body.as_bytes()).await;
            }
            let _shutdown = socket.shutdown().await;
        }
    });
    Ok(format!("http://{addr}"))
}

fn request() -> ProviderRequest {
    ProviderRequest {
        system: "se".to_owned(),
        messages: vec![Message::User {
            content: "ola".to_owned(),
        }],
        tools: vec![ToolSpec {
            name: "read_file".to_owned(),
            description: "le".to_owned(),
            parameters: r#"{"type":"object"}"#.to_owned(),
        }],
        cancel: CancelToken::new(),
    }
}

#[tokio::test]
async fn streams_sse_into_a_completion() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = serve(vec![200], CHAT_STREAM).await?;
    let provider = LlamaProvider::new(LlamaConfig::new(base_url, "ornith"))?;
    let completion = provider.complete(&request()).await?;
    assert_eq!(completion.text, "Ola mundo");
    assert_eq!(completion.reasoning, "penso");
    assert_eq!(completion.tool_calls.len(), 1);
    assert_eq!(
        completion.tool_calls.first().map(|call| call.name.as_str()),
        Some("read_file")
    );
    Ok(())
}

#[tokio::test]
async fn retries_on_transient_error_then_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = serve(vec![500, 200], CHAT_STREAM).await?;
    let mut config = LlamaConfig::new(base_url, "ornith");
    config.max_attempts = 2;
    config.retry_backoff = std::time::Duration::from_millis(1);
    let provider = LlamaProvider::new(config)?;
    let completion = provider.complete(&request()).await?;
    assert_eq!(completion.text, "Ola mundo");
    Ok(())
}
