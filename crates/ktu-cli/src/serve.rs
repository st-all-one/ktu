//! `ktu serve` — a porta única em HTTP/1.1 + HTTP/2 (SSE) sobre `127.0.0.1`.
//!
//! Todos os front-ends (CLI, TUI, web) falam os **mesmos** endpoints; nenhum
//! acede ao kernel directamente (§2.3 do `plan.md`).

use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::stream::{Stream, unfold};
use ktu_core::events;
use ktu_core::plan::{Approval, Mode};
use ktu_core::ports::{Level, LogRecord, Logger};
use ktu_port::host::Host;
use ktu_port::protocol::{Health, PortError, ServerMsg, SessionInfo, SessionState};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::runtime::Builder;

use crate::hub::SessionHub;
use crate::run::RunConfig;
use crate::turn_runtime::ensure_layout;

/// Serviço partilhado pelos handlers.
type Hub = Arc<SessionHub>;

/// Sobe o servidor até receber sinal de paragem.
///
/// # Errors
/// Devolve mensagem legível se o runtime, o *bind* ou o servidor falharem.
pub(crate) fn serve(
    root: PathBuf,
    bind: &str,
    config: RunConfig,
    logger: Arc<dyn Logger>,
) -> Result<(), String> {
    let address: SocketAddr = bind
        .parse()
        .map_err(|error| format!("endereco invalido `{bind}`: {error}"))?;
    ensure_layout(&root)?;
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("nao foi possivel iniciar o runtime: {error}"))?;
    runtime.block_on(async move {
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| format!("nao foi possivel escutar em {address}: {error}"))?;
        let local = listener
            .local_addr()
            .map_err(|error| format!("nao foi possivel ler o endereco local: {error}"))?;
        let local_text = local.to_string();
        logger.log(&LogRecord {
            level: Level::Info,
            event: events::SERVER_START,
            message: "servidor iniciado",
            fields: &[("bind", local_text.as_str())],
        });
        let hub: Hub = Arc::new(SessionHub::new(root, config, logger));
        axum::serve(listener, router(hub))
            .await
            .map_err(|error| format!("servidor terminou: {error}"))
    })
}

/// Constrói o router da porta única.
fn router(hub: Hub) -> Router {
    Router::new()
        .route("/", get(web_page))
        .route("/v1/health", get(health))
        .route("/v1/session", post(create_session))
        .route("/v1/session/{id}", get(state))
        .route("/v1/session/{id}/turn", post(turn))
        .route("/v1/session/{id}/events", get(events))
        .route("/v1/session/{id}/cancel", post(cancel))
        .route("/v1/session/{id}/mode", post(set_mode))
        .with_state(hub)
}

/// Erro da API mapeado para `HTTP` (nunca faz *panic*).
struct ApiError(PortError);

impl From<PortError> for ApiError {
    fn from(error: PortError) -> Self {
        Self(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            PortError::NoSession(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        };
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

/// Pedido de criação de sessão.
#[derive(Deserialize)]
struct CreateRequest {
    /// Identificador da sessão.
    session_id: String,
}

/// Pedido de turno.
#[derive(Deserialize)]
struct TurnRequest {
    /// Instrução do utilizador.
    prompt: String,
}

/// Pedido de mudança de modo.
#[derive(Deserialize)]
struct ModeRequest {
    /// Modo alvo.
    mode: Mode,
    /// Aprovação da transição `plan→build`.
    approval: Approval,
}

/// `GET /` — página web de demonstração (cliente da porta).
async fn web_page() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// `GET /v1/health`.
async fn health(State(hub): State<Hub>) -> Json<Health> {
    Json(hub.health().await)
}

/// `POST /v1/session`.
async fn create_session(
    State(hub): State<Hub>,
    Json(request): Json<CreateRequest>,
) -> Result<Json<SessionInfo>, ApiError> {
    Ok(Json(hub.create_session(&request.session_id).await?))
}

/// `GET /v1/session/{id}`.
async fn state(
    State(hub): State<Hub>,
    Path(id): Path<String>,
) -> Result<Json<SessionState>, ApiError> {
    Ok(Json(hub.state(&id).await?))
}

/// `POST /v1/session/{id}/turn` (202 aceite).
async fn turn(
    State(hub): State<Hub>,
    Path(id): Path<String>,
    Json(request): Json<TurnRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let turn_id = hub.turn(&id, &request.prompt).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "turn_id": turn_id }))))
}

/// `POST /v1/session/{id}/cancel`.
async fn cancel(State(hub): State<Hub>, Path(id): Path<String>) -> Result<StatusCode, ApiError> {
    hub.cancel(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /v1/session/{id}/mode` (I15).
async fn set_mode(
    State(hub): State<Hub>,
    Path(id): Path<String>,
    Json(request): Json<ModeRequest>,
) -> Result<StatusCode, ApiError> {
    hub.set_mode(&id, request.mode, request.approval).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /v1/session/{id}/events` — stream SSE dos eventos do log.
async fn events(
    State(hub): State<Hub>,
    Path(id): Path<String>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let receiver = hub.subscribe(&id);
    let stream = unfold(receiver, |mut receiver| async move {
        receiver
            .recv()
            .await
            .map(|message| (Ok::<_, Infallible>(to_sse(message)), receiver))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Converte uma mensagem do servidor num evento SSE (JSON).
fn to_sse(message: ServerMsg) -> SseEvent {
    SseEvent::default()
        .json_data(message)
        .unwrap_or_else(|_ignored| SseEvent::default().comment("evento nao serializavel"))
}

/// Página web de demonstração: `fetch` para criar/turno e `EventSource` para o SSE.
const INDEX_HTML: &str = r#"<!doctype html>
<html lang="pt"><head><meta charset="utf-8"><title>ktu — demonstração</title>
<style>body{font:14px ui-monospace,monospace;max-width:60rem;margin:2rem auto;padding:0 1rem}
#log{white-space:pre-wrap;border:1px solid #ccc;padding:.5rem;height:70vh;overflow:auto}
input{width:60%}button{padding:.3rem .8rem}</style></head>
<body><h1>ktu</h1>
<p><input id="prompt" placeholder="instrução" value="olá"><button id="send">enviar</button>
<button id="cancel">cancelar</button></p>
<div id="log"></div>
<script>
const sid = "web";
const log = (m) => { const el = document.getElementById("log"); el.textContent += m + "\n"; el.scrollTop = el.scrollHeight; };
async function api(path, body, method) {
  const r = await fetch(path, { method, headers: { "content-type": "application/json" }, body: body ? JSON.stringify(body) : undefined });
  if (!r.ok && r.status !== 202 && r.status !== 204) log("erro " + r.status + ": " + await r.text());
  return r;
}
(async () => {
  await api("/v1/session", { session_id: sid }, "POST");
  const st = await (await api("/v1/session/" + sid, null, "GET")).json();
  for (const ev of st.log) log(JSON.stringify(ev));
  const es = new EventSource("/v1/session/" + sid + "/events");
  es.onmessage = (e) => log(JSON.stringify(JSON.parse(e.data)));
})();
document.getElementById("send").onclick = () => api("/v1/session/" + sid + "/turn", { prompt: document.getElementById("prompt").value }, "POST");
document.getElementById("cancel").onclick = () => api("/v1/session/" + sid + "/cancel", null, "POST");
</script></body></html>"#;
