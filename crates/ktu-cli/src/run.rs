//! `ktu run "<instrução>"` — turno do agente em modo *embedded* (sem socket).
//!
//! `stdout` recebe a resposta final; `stderr` recebe os eventos estruturados.
//! O provider é o `llama-server` real (llama.cpp); `--provider mock` usa o
//! eco determinístico (sem modelo) para exercitar o loop.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ktu_core::cancel::CancelToken;
use ktu_core::domain::StopReason;
use ktu_core::facade::final_text;
use ktu_core::plan::Mode;
use ktu_core::ports::Logger;
use ktu_core::provider::Provider;
use ktu_provider::{Flavor, LlamaConfig, LlamaProvider, ReasoningEffort};
use ktu_tools::{FileMutationQueue, StdFs};
use std::sync::Arc;
use tokio::runtime::Builder;

use crate::clock::SystemClock;
use crate::lock::TurnLock;
use crate::session_mode::{SessionMode, SessionTarget};
use crate::session_store::{apply_session_start, load_session, save_checkpoint, save_session};
use crate::turn_runtime::{ensure_layout, ensure_sessions, execute_turn};
use ktu_core::testing::EchoProvider;

use crate::provider_script::ScriptedProvider;

/// Configuração do turno vinda da linha de comandos.
#[derive(Debug, Clone)]
pub(crate) struct RunConfig {
    /// Provider: `llama` (real), `mock` (eco) ou `scripted` (guião, sem modelo).
    pub provider: String,
    /// Guião do `--provider scripted` (JSONL: uma `Completion` por linha).
    pub script: Option<PathBuf>,
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
    /// Tools **diferidas** (`.agents/config.toml`): fora do catálogo até `tool_search`.
    pub deferred_tools: Vec<String>,
    /// Identificador da sessão persistida (efémera se `Ephemeral`).
    pub session: SessionTarget,
    /// Modo de arranque da sessão (nova, retoma, retoma+compacta).
    pub mode: SessionMode,
    /// Arranca em modo plano (read-only + tool `plan`) — §2.13.
    pub plan: bool,
}

/// Resolve a instrução: `-` ou vazio com *pipe* lê de `stdin` (heredoc); caso
/// contrário, junta as palavras.
///
/// # Errors
/// Devolve mensagem legível se a leitura de `stdin` falhar.
pub(crate) fn resolve_prompt(words: &[String]) -> Result<String, String> {
    use std::io::{IsTerminal, Read};
    let from_stdin = words == ["-"] || (words.is_empty() && !io::stdin().is_terminal());
    if from_stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("nao foi possivel ler stdin: {error}"))?;
        Ok(buffer.trim_end().to_owned())
    } else {
        Ok(words.join(" "))
    }
}

/// Corre um turno e imprime a resposta final em `stdout`.
pub(crate) fn run(root: &Path, prompt: &str, logger: &dyn Logger, config: &RunConfig) -> ExitCode {
    let mutations = Arc::new(FileMutationQueue::new());
    let (ok, text) = run_capture(root, prompt, logger, config, &mutations);
    if let Some(content) = text {
        let mut stdout = io::stdout().lock();
        let _ignored = writeln!(stdout, "{content}");
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Corre um turno e devolve `(sucesso, resposta final)` **sem** imprimir.
///
/// Usado pelo `--batch`, que recolhe os resultados para os ordenar.
pub(crate) fn run_capture(
    root: &Path,
    prompt: &str,
    logger: &dyn Logger,
    config: &RunConfig,
    mutations: &Arc<FileMutationQueue>,
) -> (bool, Option<String>) {
    let runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            report_error(&format!("nao foi possivel iniciar o runtime: {error}"));
            return (false, None);
        }
    };
    runtime.block_on(drive(root, prompt, logger, config, mutations))
}

/// Constrói o provider pedido (real ou de demonstração).
///
/// # Errors
/// Devolve mensagem legível se o provider for desconhecido ou a configuração for inválida.
pub(crate) fn build_provider(config: &RunConfig) -> Result<Box<dyn Provider>, String> {
    match config.provider.as_str() {
        "mock" => Ok(Box::new(EchoProvider)),
        "scripted" => {
            let path = config
                .script
                .as_deref()
                .ok_or_else(|| "--provider scripted exige --script <ficheiro>".to_owned())?;
            Ok(Box::new(ScriptedProvider::load(path)?))
        }
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
            llama.session = Some(config.session.label().to_owned());
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
///
/// Devolve `(exit, resposta final)` — **sem** imprimir (o `run` imprime; o
/// `--batch` recolhe para ordenar).
async fn drive(
    root: &Path,
    prompt: &str,
    logger: &dyn Logger,
    config: &RunConfig,
    mutations: &Arc<FileMutationQueue>,
) -> (bool, Option<String>) {
    if let Err(message) = ensure_layout(root) {
        report_error(&message);
        return (false, None);
    }
    if config.session.is_persistent()
        && let Err(message) = ensure_sessions(root)
    {
        report_error(&message);
        return (false, None);
    }
    let fs = StdFs;
    let clock = SystemClock::new();
    let sessions_dir = root.join(".ktu").join("sessions");
    let _lock = match config.session.id() {
        Some(id) => match TurnLock::acquire(&sessions_dir, id, &clock) {
            Ok(lock) => Some(lock),
            Err(message) => {
                report_error(&message);
                return (false, None);
            }
        },
        None => None,
    };
    let mut log = match load_session(&fs, &sessions_dir, config) {
        Ok(log) => log,
        Err(message) => {
            report_error(&message);
            return (false, None);
        }
    };
    apply_session_start(&fs, &sessions_dir, config, &mut log);
    if config.plan && log.mode() != Mode::Plan {
        log.set_mode(Mode::Plan);
    }
    let cancel = CancelToken::new();
    let reason = match execute_turn(
        root, &mut log, prompt, config, logger, &cancel, None, mutations,
    )
    .await
    {
        Ok(reason) => reason,
        Err(message) => {
            report_error(&message);
            return (false, None);
        }
    };
    if let Err(error) = save_session(&fs, &sessions_dir, config, &log) {
        report_error(&format!("nao foi possivel gravar a sessao: {error}"));
    }
    save_checkpoint(&fs, &sessions_dir, config, &log, &clock);
    let ok = reason != StopReason::Error;
    (ok, final_text(&log))
}

/// Escreve em `stderr` uma mensagem de erro da borda.
pub(crate) fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}

#[cfg(test)]
mod tests {
    use super::resolve_prompt;

    #[test]
    fn words_are_joined_when_not_piped() {
        let words = vec!["faz".to_owned(), "isto".to_owned()];
        assert_eq!(resolve_prompt(&words).unwrap_or_default(), "faz isto");
    }

    #[test]
    fn empty_words_without_pipe_are_empty() {
        // Sem pipe (TTY nos testes) o resultado é vazio, não um erro.
        assert_eq!(resolve_prompt(&[]).unwrap_or_default(), "");
    }
}
