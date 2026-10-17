//! `ktu run "<instrução>"` — turno do agente em modo *embedded* (sem socket).
//!
//! `stdout` recebe a resposta final; `stderr` recebe os eventos estruturados.
//! O provider é o `llama-server` real (llama.cpp); `--provider mock` usa o
//! eco determinístico (sem modelo) para exercitar o loop.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_core::cancel::CancelToken;
use ktu_core::domain::StopReason;
use ktu_core::plan::Mode;
use ktu_core::ports::Logger;
use ktu_core::provider::Provider;
use ktu_core::session::{Event, SessionLog};
use ktu_provider::{Flavor, LlamaConfig, LlamaProvider, ReasoningEffort};
use ktu_tools::StdFs;
use tokio::runtime::Builder;

use crate::clock::SystemClock;
use crate::provider_fake::EchoProvider;
use crate::session_mode::SessionMode;
use crate::session_store::{apply_session_start, load_session, save_checkpoint, save_session};
use crate::turn_runtime::{ensure_layout, execute_turn};

/// Configuração do turno vinda da linha de comandos.
#[derive(Debug, Clone)]
pub(crate) struct RunConfig {
    /// Provider: `llama` (real) ou `mock` (eco).
    pub provider: String,
    /// *Base URL* do `llama-server`.
    pub base_url: String,
    /// Nome/alias do modelo.
    pub model: String,
    /// Chave do gateway remoto (`opencode-go`), quando exigida.
    pub api_key: Option<String>,
    /// Esforço de raciocínio (`low|medium|high|xhigh`).
    pub effort: String,
    /// Tecto de passos do turno.
    pub max_steps: u32,
    /// Identificador da sessão persistida.
    pub session: String,
    /// Modo de arranque da sessão (nova, retoma, retoma+compacta).
    pub mode: SessionMode,
    /// Arranca em modo plano (read-only + tool `plan`) — §2.13.
    pub plan: bool,
}

/// Corre um turno e imprime a resposta final em `stdout`.
pub(crate) fn run(root: &Path, prompt: &str, logger: &dyn Logger, config: &RunConfig) -> ExitCode {
    let runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            report_error(&format!("nao foi possivel iniciar o runtime: {error}"));
            return ExitCode::FAILURE;
        }
    };
    runtime.block_on(drive(root, prompt, logger, config))
}

/// Constrói o provider pedido (real ou de demonstração).
///
/// # Errors
/// Devolve mensagem legível se o provider for desconhecido ou a configuração for inválida.
pub(crate) fn build_provider(config: &RunConfig) -> Result<Box<dyn Provider>, String> {
    match config.provider.as_str() {
        "mock" => Ok(Box::new(EchoProvider)),
        "llama" => {
            let mut llama = LlamaConfig::new(config.base_url.clone(), config.model.clone());
            llama.effort = parse_effort(&config.effort)?;
            let provider = LlamaProvider::new(llama).map_err(|error| error.to_string())?;
            let boxed: Box<dyn Provider> = Box::new(provider);
            Ok(boxed)
        }
        "opencode-go" => {
            let key = config.api_key.clone().ok_or_else(|| {
                "chave ausente: exporte OPENCODE_GO_API_KEY ou use --api-key".to_owned()
            })?;
            let mut llama = LlamaConfig::new(config.base_url.clone(), config.model.clone());
            llama.effort = parse_effort(&config.effort)?;
            llama.api_key = Some(key);
            llama.session = Some(config.session.clone());
            llama.session_header = Some("x-opencode-session".to_owned());
            llama.affinity_headers = vec![
                "x-client-request-id".to_owned(),
                "x-session-affinity".to_owned(),
            ];
            llama.flavor = Flavor::OpenAi;
            let provider = LlamaProvider::new(llama).map_err(|error| error.to_string())?;
            let boxed: Box<dyn Provider> = Box::new(provider);
            Ok(boxed)
        }
        other => Err(format!("provider desconhecido: {other}")),
    }
}

/// Traduz o texto do esforço de raciocínio para o tipo do domínio.
fn parse_effort(value: &str) -> Result<ReasoningEffort, String> {
    match value {
        "low" => Ok(ReasoningEffort::Low),
        "medium" => Ok(ReasoningEffort::Medium),
        "high" => Ok(ReasoningEffort::High),
        "xhigh" => Ok(ReasoningEffort::Xhigh),
        other => Err(format!("esforco de raciocinio desconhecido: {other}")),
    }
}

/// Carrega a sessão, corre o turno e persiste o resultado.
async fn drive(root: &Path, prompt: &str, logger: &dyn Logger, config: &RunConfig) -> ExitCode {
    if let Err(message) = ensure_layout(root) {
        report_error(&message);
        return ExitCode::FAILURE;
    }
    let fs = StdFs;
    let clock = SystemClock::new();
    let sessions_dir = root.join(".ktu").join("sessions");
    let mut log = match load_session(&fs, &sessions_dir, config) {
        Ok(log) => log,
        Err(message) => {
            report_error(&message);
            return ExitCode::FAILURE;
        }
    };
    apply_session_start(&fs, &sessions_dir, config, &mut log);
    if config.plan && log.mode() != Mode::Plan {
        log.set_mode(Mode::Plan);
    }
    let cancel = CancelToken::new();
    let reason = match execute_turn(root, &mut log, prompt, config, logger, &cancel).await {
        Ok(reason) => reason,
        Err(message) => {
            report_error(&message);
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = save_session(&fs, &sessions_dir, config, &log) {
        report_error(&format!("nao foi possivel gravar a sessao: {error}"));
    }
    save_checkpoint(&fs, &sessions_dir, config, &log, &clock);
    emit_final(&log, reason)
}

/// Imprime a última resposta do modelo e mapeia a razão de término para o exit.
/// Extrai a resposta final **do turno corrente** (nunca de um turno anterior).
fn emit_final(log: &SessionLog, reason: StopReason) -> ExitCode {
    let events = log.events();
    let start = events
        .iter()
        .rposition(|event| matches!(event, Event::TurnStart { .. }))
        .map_or(0, |index| index.saturating_add(1));
    let final_text =
        events
            .get(start..)
            .unwrap_or(&[])
            .iter()
            .rev()
            .find_map(|event| match event {
                Event::Assistant {
                    content,
                    tool_calls,
                    ..
                } if tool_calls.is_empty() => Some(content.as_str()),
                _ => None,
            });
    if let Some(content) = final_text {
        let mut stdout = io::stdout().lock();
        let _ignored = writeln!(stdout, "{content}");
    }
    if reason == StopReason::Error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Escreve em `stderr` uma mensagem de erro da borda.
pub(crate) fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
