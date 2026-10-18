//! `ktu memory drain` e `ktu memory drain service` — a fila derivada de embeddings (H8).
//!
//! O dreno é **off-path**: compara o `body_hash` das notas com o índice vetorial,
//! embeda só o que falta, grava uma vez por lote e **nunca descarta nota**. Sem
//! provedor, o recall é lexical e o canal semântico desliga sem aviso.
//!
//! O `service` gere o worker (agendador + servidor de embeddings) delegando no
//! script embutido ([`crate::script`]).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ktu_core::config::global_dir;
use ktu_core::memory_config::Embeddings;
use ktu_memory::drain::{DrainOutcome, DrainStatus, digest, status};
use ktu_memory::embed::index_dir;
use ktu_memory::{JsonlMemory, Note, Rebuild};
use ktu_tools::StdFs;

use crate::actions::ServiceArgs;
use crate::clock::SystemClock;
use crate::embeddings;
use crate::output::{self, Format};
use crate::script;

/// Modo de `ktu memory drain`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrainMode {
    /// Reporta a fila sem tocar no índice (default).
    Status,
    /// Digere a fila em lotes.
    Digest,
    /// Ignora o índice e reindexa tudo (último recurso).
    Force,
}

/// Acção do worker de auto-drain (mutuamente exclusiva; `--status` é o default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Install,
    Subscribe,
    Unsubscribe,
    Status,
    Uninstall,
    Reconcile,
}

impl Action {
    /// Resolve a acção a partir das flags (o `clap` garante no máximo uma).
    fn of(args: &ServiceArgs) -> Self {
        if args.install {
            Self::Install
        } else if args.subscribe {
            Self::Subscribe
        } else if args.unsubscribe {
            Self::Unsubscribe
        } else if args.uninstall {
            Self::Uninstall
        } else if args.reconcile {
            Self::Reconcile
        } else {
            Self::Status
        }
    }

    /// Nome estável (contrato com o worker).
    fn name(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Subscribe => "subscribe",
            Self::Unsubscribe => "unsubscribe",
            Self::Status => "status",
            Self::Uninstall => "uninstall",
            Self::Reconcile => "reconcile",
        }
    }

    /// Acções que operam sobre o projecto actual.
    fn scoped(self) -> bool {
        matches!(
            self,
            Self::Install | Self::Subscribe | Self::Unsubscribe | Self::Reconcile
        )
    }
}

/// `ktu memory drain [--status|--digest|--force]`.
pub(crate) fn queue(root: &Path, mode: DrainMode, format: Format) -> ExitCode {
    let fs = StdFs;
    let clock = SystemClock::new();
    let config = match embeddings::load_config(&fs, root) {
        Ok(config) => config,
        Err(message) => {
            output::emit_error(format, "memory drain", "config", &message);
            return ExitCode::FAILURE;
        }
    };
    let memory = JsonlMemory::new(&fs, &clock, root, root.join(".agents").join("memory"));
    let notes = match memory.load_notes() {
        Ok(notes) => notes,
        Err(error) => {
            output::emit_error(
                format,
                "memory drain",
                "io",
                &format!("nao foi possivel ler a memoria: {error}"),
            );
            return ExitCode::FAILURE;
        }
    };
    let dir = memory.memory_dir();
    match mode {
        DrainMode::Status => report(fs, dir, &config.embeddings, &notes, format),
        DrainMode::Digest | DrainMode::Force => {
            run_digest(fs, dir, &config.embeddings, &notes, mode, format)
        }
    }
}

/// `ktu memory drain service <acção>` — delega no worker embutido.
pub(crate) fn service(root: &Path, flags: &ServiceArgs, format: Format) -> ExitCode {
    let action = Action::of(flags);
    let argv = worker_args(action, root, flags);
    let run = if matches!(action, Action::Install) {
        match config_dir() {
            Some(dir) => match script::materialize(&dir) {
                Ok(path) => script::run_path(&path, &argv),
                Err(message) => Err(message),
            },
            None => Err("nao foi possivel determinar o directorio de configuracao".to_owned()),
        }
    } else {
        script::run_embedded(&argv)
    };
    match run {
        Ok(run) => {
            let text = if run.stdout.trim().is_empty() {
                format!("drain service {}: ok", action.name())
            } else {
                run.stdout.trim().to_owned()
            };
            let data = serde_json::json!({
                "action": action.name(),
                "done": run.success,
                "worker": script::WORKER_FILE,
            });
            output::emit(format, "memory drain service", &text, &data);
            if run.success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            output::emit_error(format, "memory drain service", "io", &message);
            ExitCode::FAILURE
        }
    }
}

/// Argumentos repassados ao worker (contrato `ktu-idle <acção> [flags]`).
fn worker_args(action: Action, root: &Path, flags: &ServiceArgs) -> Vec<String> {
    let mut argv = vec![action.name().to_owned()];
    if action.scoped() {
        argv.push("--project".to_owned());
        argv.push(root.display().to_string());
    }
    if matches!(action, Action::Install) {
        argv.push("--every".to_owned());
        argv.push(flags.every.clone());
        argv.push("--port".to_owned());
        argv.push(flags.port.to_string());
        if let Some(model) = &flags.model {
            argv.push("--model".to_owned());
            argv.push(model.clone());
        }
        if flags.no_deps {
            argv.push("--no-deps".to_owned());
        }
    }
    if matches!(action, Action::Uninstall) {
        if flags.keep_model {
            argv.push("--keep-model".to_owned());
        }
        if flags.remove_model {
            argv.push("--remove-model".to_owned());
        }
    }
    if flags.dry_run {
        argv.push("--dry-run".to_owned());
    }
    argv
}

/// Directório de configuração global do ktu (`$XDG_CONFIG_HOME/ktu` ou `~/.config/ktu`).
fn config_dir() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_CONFIG_HOME").and_then(|value| value.into_string().ok());
    let home = std::env::var_os("HOME").and_then(|value| value.into_string().ok());
    global_dir(xdg.as_deref(), home.as_deref())
}

/// Reporta a fila (read-only).
fn report(fs: StdFs, dir: &Path, config: &Embeddings, notes: &[Note], format: Format) -> ExitCode {
    match status(&fs, dir, config, notes) {
        Ok(report) => {
            emit_status(format, &report);
            ExitCode::SUCCESS
        }
        Err(error) => {
            output::emit_error(format, "memory drain", "config", &error.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Digere a fila (ou explica porque não há nada a fazer).
#[allow(
    clippy::too_many_arguments,
    reason = "comando explicito: portas + configuracao + notas + modo; agrupar esconderia a DI"
)]
fn run_digest(
    fs: StdFs,
    dir: &Path,
    config: &Embeddings,
    notes: &[Note],
    mode: DrainMode,
    format: Format,
) -> ExitCode {
    let embedder = match embeddings::build(config) {
        Ok(Some(embedder)) => embedder,
        Ok(None) => {
            output::emit(
                format,
                "memory drain",
                DISABLED,
                &serde_json::json!({ "provider": "none", "indexed": 0, "pending": 0, "stale": 0 }),
            );
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            output::emit_error(format, "memory drain", "config", &message);
            return ExitCode::FAILURE;
        }
    };
    let index_dir = index_dir(dir);
    if let Err(error) = std::fs::create_dir_all(&index_dir) {
        output::emit_error(
            format,
            "memory drain",
            "io",
            &format!("nao foi possivel criar {}: {error}", index_dir.display()),
        );
        return ExitCode::FAILURE;
    }
    let rebuild = if mode == DrainMode::Force {
        Rebuild::Wipe
    } else {
        Rebuild::Keep
    };
    match digest(&fs, dir, config, notes, &embedder, rebuild) {
        Ok(outcome) => emit_outcome(format, &outcome),
        Err(error) => {
            output::emit_error(format, "memory drain", "io", &error.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Mensagem do canal semântico desligado.
const DISABLED: &str = "memory.embeddings.channel: lexical\nmemory.embeddings.next: canal semantico \
desligado (`embeddings.provider = none`); rode `ktu memory drain service --install` para ligar";

/// Emite o estado da fila (texto + envelope JSON).
fn emit_status(format: Format, report: &DrainStatus) {
    let channel = if report.channel.is_semantic() {
        "semantic"
    } else {
        "lexical"
    };
    let index = if report.index.is_present() {
        "present"
    } else {
        "absent"
    };
    let lines = [
        format!("memory.embeddings.provider: {}", report.provider),
        format!("memory.embeddings.model: {}", report.model),
        format!("memory.embeddings.dimensions: {}", report.dimensions),
        format!("memory.embeddings.channel: {channel}"),
        format!("memory.embeddings.index: {index}"),
        format!("memory.embeddings.indexed: {}", report.indexed),
        format!("memory.embeddings.pending: {}", report.pending),
        format!("memory.embeddings.stale: {}", report.stale),
        format!("memory.embeddings.next: {}", next_step(report)),
    ];
    let data = serde_json::json!({
        "provider": report.provider,
        "model": report.model,
        "dimensions": report.dimensions,
        "channel": channel,
        "index": index,
        "indexed": report.indexed,
        "pending": report.pending,
        "stale": report.stale,
    });
    output::emit(format, "memory drain", &lines.join("\n"), &data);
}

/// Passo seguinte sugerido pelo estado.
fn next_step(report: &DrainStatus) -> String {
    if !report.channel.is_semantic() {
        return "canal semantico desligado; `ktu memory drain service --install` liga-o".to_owned();
    }
    if !report.index.is_present() {
        return "nada digerido ainda; rode `ktu memory drain --digest`".to_owned();
    }
    let backlog = report.pending.saturating_add(report.stale);
    if backlog > 0 {
        return format!("rode `ktu memory drain --digest` para drenar {backlog} nota(s)");
    }
    "fila em dia".to_owned()
}

/// Emite o resultado do dreno; falha do provedor ⇒ `exit 1` (a fila fica pendente).
fn emit_outcome(format: Format, outcome: &DrainOutcome) -> ExitCode {
    if let Some(message) = &outcome.failure {
        output::emit_error(
            format,
            "memory drain",
            "unavailable",
            &format!(
                "provedor de embeddings indisponivel: {message} (indexados {}, pendentes {})",
                outcome.indexed, outcome.pending
            ),
        );
        return ExitCode::FAILURE;
    }
    let text = format!(
        "memory.embeddings.indexed: {}\nmemory.embeddings.pending: {}\nmemory.embeddings.stale: {}\nmemory.embeddings.batches: {}",
        outcome.indexed, outcome.pending, outcome.stale, outcome.batches
    );
    let data = serde_json::json!({
        "indexed": outcome.indexed,
        "pending": outcome.pending,
        "stale": outcome.stale,
        "batches": outcome.batches,
    });
    output::emit(format, "memory drain", &text, &data);
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests;
