//! `ktu-cli` — o binário `ktu`: front-end de linha de comandos (modo embedded).
//!
//! `stdout` é **dado** (a resposta final); `stderr` é **log** estruturado. Nunca
//! se misturam (§1.3 do `epic.md`).
#![forbid(unsafe_code)]

mod clock;
mod init;
pub(crate) mod logging;
mod memory_cmd;
mod provider_fake;
mod providers;
mod run;
mod trash_cmd;

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// `ktu` — agente mínimo: sessão, contexto e cache atrás de uma porta única.
#[derive(Debug, Parser)]
#[command(name = "ktu", version, about, long_about = None)]
struct Cli {
    /// Subcomando a executar (por omissão, mostra a ajuda).
    #[command(subcommand)]
    command: Option<Command>,
}

/// Subcomandos do `ktu`.
#[derive(Debug, Subcommand)]
#[allow(
    clippy::large_enum_variant,
    reason = "variante Run agrega os argumentos da linha de comandos; trivial e unica"
)]
enum Command {
    /// Inicializa o layout `.agents/` (conhecimento) e `.ktu/` (runtime).
    Init,
    /// Corre um turno do agente contra o `llama-server` (ou `--provider mock`).
    Run {
        /// Instrução do utilizador (palavras).
        prompt: Vec<String>,
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Provider: `llama` (real) ou `mock` (eco, sem modelo).
        #[arg(long, env = "KTU_PROVIDER", default_value = "llama")]
        provider: String,
        /// *Base URL* do `llama-server`.
        #[arg(long, env = "KTU_BASE_URL", default_value = "http://127.0.0.1:8080")]
        base_url: String,
        /// Nome/alias do modelo.
        #[arg(long, env = "KTU_MODEL", default_value = "ornith")]
        model: String,
        /// Esforço de raciocínio (`low|medium|high|xhigh`).
        #[arg(long, env = "KTU_EFFORT", default_value = "xhigh")]
        effort: String,
        /// Tecto de passos do turno.
        #[arg(long, env = "KTU_MAX_STEPS", default_value_t = 24)]
        max_steps: u32,
        /// Identificador da sessão persistida (`.ktu/sessions/<id>.jsonl`).
        #[arg(long, env = "KTU_SESSION", default_value = "default")]
        session: String,
        /// Retoma a sessão (carrega o log) antes deste turno.
        #[arg(long)]
        resume: bool,
    },
    /// Diagnostica o provider `llama.cpp` (`llama-server`).
    Providers {
        /// Acção de diagnóstico.
        #[command(subcommand)]
        action: ProvidersAction,
    },
    /// Gere o *trash* (ficheiros movidos em vez de apagados).
    Trash {
        /// Acção sobre o *trash*.
        #[command(subcommand)]
        action: TrashAction,
    },
    /// Inspecciona e mantém a memória persistente do projecto.
    Memory {
        /// Acção sobre a memória.
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Diagnostica o layout e o conhecimento do projecto.
    Doctor {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Diagnostica a memória (âncoras quebradas, saúde do índice).
        #[arg(long)]
        memory: bool,
    },
}

/// Acções de `ktu memory`.
#[derive(Debug, Subcommand)]
enum MemoryAction {
    /// Imprime a vista humana (Markdown) derivada de `notes.jsonl`.
    Export {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Lista uma nota por linha (`id`, tipo e afirmação).
    List {
        /// *Root* do projecto (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

/// Acções de `ktu providers`.
#[derive(Debug, Subcommand)]
enum ProvidersAction {
    /// Verifica ligação, modelo e (opcionalmente) o ficheiro GGUF.
    Doctor {
        /// *Base URL* do `llama-server`.
        #[arg(long, env = "KTU_BASE_URL", default_value = "http://127.0.0.1:8080")]
        base_url: String,
        /// Caminho do GGUF a inspeccionar (magic + metadados).
        #[arg(long)]
        gguf: Option<PathBuf>,
        /// Corre uma sonda funcional com tool-calling.
        #[arg(long)]
        probe: bool,
    },
}

/// Acções de `ktu trash`.
#[derive(Debug, Subcommand)]
enum TrashAction {
    /// Lista as entradas do *trash*.
    List {
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Restaura uma entrada para o caminho original.
    Restore {
        /// Identificador da entrada.
        id: String,
        /// *Root* da sessão (por omissão, o directório actual).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let logger = logging::init();
    let cli = Cli::parse();
    dispatch(cli.command, &logger)
}

/// Despacha o subcomando escolhido para a borda correspondente.
fn dispatch(command: Option<Command>, logger: &logging::TracingLogger) -> ExitCode {
    match command {
        Some(Command::Init) => run_init(),
        Some(Command::Run {
            prompt,
            root,
            provider,
            base_url,
            model,
            effort,
            max_steps,
            session,
            resume,
        }) => {
            let config = run::RunConfig {
                provider,
                base_url,
                model,
                effort,
                max_steps,
                session,
                resume,
            };
            run_command(root, &prompt, &config, logger)
        }
        Some(Command::Providers { action }) => match action {
            ProvidersAction::Doctor {
                base_url,
                gguf,
                probe,
            } => {
                let choice = if probe {
                    ktu_provider::Probe::Run
                } else {
                    ktu_provider::Probe::Skip
                };
                providers::doctor(&base_url, gguf.as_deref(), choice)
            }
        },
        Some(Command::Trash { action }) => match action {
            TrashAction::List { root } => trash_cmd::list(&root_or_cwd(root)),
            TrashAction::Restore { id, root } => trash_cmd::restore(&root_or_cwd(root), &id),
        },
        Some(Command::Memory { action }) => match action {
            MemoryAction::Export { root } => memory_cmd::export(&root_or_cwd(root)),
            MemoryAction::List { root } => memory_cmd::list(&root_or_cwd(root)),
        },
        Some(Command::Doctor { root, memory }) => {
            let root = root_or_cwd(root);
            if memory {
                memory_cmd::doctor(&root)
            } else {
                providers::doctor("http://127.0.0.1:8080", None, ktu_provider::Probe::Skip)
            }
        }
        None => ExitCode::SUCCESS,
    }
}

/// Resolve o *root* pedido, caindo para o directório actual.
fn root_or_cwd(root: Option<PathBuf>) -> PathBuf {
    root.unwrap_or_else(|| env::current_dir().unwrap_or_default())
}

/// Corre um turno do agente (`ktu run`).
fn run_command(
    root: Option<PathBuf>,
    prompt: &[String],
    config: &run::RunConfig,
    logger: &logging::TracingLogger,
) -> ExitCode {
    run::run(&root_or_cwd(root), &prompt.join(" "), logger, config)
}

/// Executa `ktu init` no directório actual e reporta o que foi criado em `stdout`.
fn run_init() -> ExitCode {
    let Ok(root) = env::current_dir() else {
        report_error("nao foi possivel determinar o directório actual");
        return ExitCode::FAILURE;
    };
    match init::run(&root) {
        Ok(created) => {
            report_created(&created, &root);
            ExitCode::SUCCESS
        }
        Err(error) => {
            report_error(&error.to_string());
            ExitCode::FAILURE
        }
    }
}

/// Escreve em `stdout` (dado) os caminhos criados.
fn report_created(created: &[&str], root: &Path) {
    let mut stdout = io::stdout().lock();
    if created.is_empty() {
        let _ignored = writeln!(stdout, "layout ja existe em {}", root.display());
    } else {
        for path in created {
            let _ignored = writeln!(stdout, "criado {path}");
        }
    }
}

/// Escreve em `stderr` (log) uma mensagem de erro da borda.
fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
