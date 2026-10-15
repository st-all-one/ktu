//! `ktu run "<instrução>"` — turno do agente em modo *embedded* (sem socket).
//!
//! `stdout` recebe a resposta final; `stderr` recebe os eventos estruturados.
//! O provider é o `llama-server` real (llama.cpp); `--provider mock` usa o
//! eco determinístico (sem modelo) para exercitar o loop.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_core::agent::{Deps, LoopConfig, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::checkpoint::{self, Checkpoint};
use ktu_core::domain::StopReason;
use ktu_core::guard::StructuralGuard;
use ktu_core::ports::{Clock, Fs, Logger, NoopMetrics};
use ktu_core::prompt::{BASE, compose};
use ktu_core::provider::Provider;
use ktu_core::session::{Event, SessionLog};
use ktu_core::store;
use ktu_memory::JsonlMemory;
use ktu_provider::{Flavor, LlamaConfig, LlamaProvider, ReasoningEffort};
use ktu_tools::{StdFs, Toolbox, tool_names};
use tokio::runtime::Builder;

use crate::clock::SystemClock;
use crate::provider_fake::EchoProvider;
use crate::session_mode::{SessionMode, force_compaction};

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
fn build_provider(config: &RunConfig) -> Result<Box<dyn Provider>, String> {
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

/// Monta as dependências e corre o turno do agente.
async fn drive(root: &Path, prompt: &str, logger: &dyn Logger, config: &RunConfig) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let memory = JsonlMemory::new(&fs, &clock, root, root.join(".agents").join("memory"));
    let tools = Toolbox::new(root, &fs, &clock, Some(&memory));
    let guard = StructuralGuard::new(root, tool_names(), 1);
    let budget = BudgetGate::new(120_000);
    let metrics = NoopMetrics;
    let cancel = CancelToken::new();
    let provider = match build_provider(config) {
        Ok(provider) => provider,
        Err(message) => {
            report_error(&message);
            return ExitCode::FAILURE;
        }
    };
    let deps = Deps {
        provider: provider.as_ref(),
        tools: &tools,
        guard: &guard,
        budget: &budget,
        logger,
        metrics: &metrics,
        clock: &clock,
        cancel: &cancel,
        memory: Some(&memory),
    };
    let sessions_dir = root.join(".ktu").join("sessions");
    let trash_dir = root.join(".ktu").join("trash");
    let memory_dir = root.join(".agents").join("memory");
    for dir in [&sessions_dir, &trash_dir, &memory_dir] {
        if let Err(error) = std::fs::create_dir_all(dir) {
            report_error(&format!(
                "nao foi possivel criar {}: {error}",
                dir.display()
            ));
            return ExitCode::FAILURE;
        }
    }
    let mut log = match load_session(&fs, &sessions_dir, config) {
        Ok(log) => log,
        Err(message) => {
            report_error(&message);
            return ExitCode::FAILURE;
        }
    };
    apply_session_start(&fs, &sessions_dir, config, &mut log);
    let system = compose_system(&fs, root);
    let loop_config = LoopConfig {
        max_steps: config.max_steps,
        ..LoopConfig::default()
    };
    let reason = run_turn(&mut log, &deps, &system, prompt, &loop_config).await;
    if let Err(error) = save_session(&fs, &sessions_dir, config, &log) {
        report_error(&format!("nao foi possivel gravar a sessao: {error}"));
    }
    save_checkpoint(&fs, &sessions_dir, config, &log, &clock);
    emit_final(&log, reason)
}

/// Aplica o arranque da sessão: checkpoint (se retoma) e compactação forçada.
fn apply_session_start(fs: &dyn Fs, dir: &Path, config: &RunConfig, log: &mut SessionLog) {
    apply_checkpoint(fs, dir, config, log);
    if config.mode.compacts() {
        force_compaction(log);
    }
}

/// Aplica ao log o corte de compactação registado no checkpoint (retoma idêntica).
fn apply_checkpoint(fs: &dyn Fs, dir: &Path, config: &RunConfig, log: &mut SessionLog) {
    if !config.mode.resumes() {
        return;
    }
    let Ok(Some(checkpoint)) = checkpoint::load(fs, dir, &config.session) else {
        return;
    };
    if let Some(keep_from) = checkpoint.keep_from {
        log.set_compaction(usize::try_from(keep_from).unwrap_or(0));
    }
}

/// Grava o checkpoint tipado e durável (inclui o corte de compactação).
fn save_checkpoint(
    fs: &dyn Fs,
    dir: &Path,
    config: &RunConfig,
    log: &SessionLog,
    clock: &dyn Clock,
) {
    let checkpoint = Checkpoint {
        session: config.session.clone(),
        turns: count_turns(log),
        compacted: log.is_compacted(),
        keep_from: log
            .compaction_from()
            .and_then(|value| u32::try_from(value).ok()),
        updated_ms: clock.now().as_millis(),
    };
    if let Err(error) = checkpoint::save(fs, dir, &checkpoint) {
        report_error(&format!("nao foi possivel gravar o checkpoint: {error}"));
    }
}

/// Conta os turnos concluídos no log (`TurnEnd`).
fn count_turns(log: &SessionLog) -> u32 {
    let count = log
        .events()
        .iter()
        .filter(|event| matches!(event, Event::TurnEnd { .. }))
        .count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Carrega o log da sessão (ou começa um novo, se `--resume` estiver em falta).
fn load_session(fs: &dyn Fs, dir: &Path, config: &RunConfig) -> Result<SessionLog, String> {
    if !config.mode.resumes() {
        return Ok(SessionLog::new());
    }
    match store::load_recovering(fs, dir, &config.session) {
        Ok(Some(log)) => Ok(log),
        Ok(None) => Ok(SessionLog::new()),
        Err(error) => Err(format!(
            "nao foi possivel retomar {}: {error}",
            config.session
        )),
    }
}

/// Grava o log da sessão de forma durável.
fn save_session(
    fs: &dyn Fs,
    dir: &Path,
    config: &RunConfig,
    log: &SessionLog,
) -> ktu_core::Result<()> {
    store::save(fs, dir, &config.session, log)
}

/// Compõe o system prompt: base + `AGENTS.md` do projecto (condensado).
fn compose_system(fs: &dyn Fs, root: &Path) -> String {
    let agents_md = fs
        .read(&root.join("AGENTS.md"))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());
    compose(BASE, agents_md.as_deref())
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
fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
