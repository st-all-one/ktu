//! Exemplo de **incorporação** do ktu noutro projecto Rust (uso #1).
//!
//! Mostra o caminho completo de quem embute o kernel:
//!
//! 1. **adaptadores** — o `ktu-core` é puro, por isso a borda constrói as portas
//!    reais: `StdFs` (ktu-tools), um `Clock` e um `Logger` próprios (30 linhas cada),
//!    `NoopMetrics` e o provider (`LlamaProvider`, ou o `EchoProvider` do core
//!    para correr sem modelo — o mesmo do `ktu run --provider mock`);
//! 2. **thread de tools (T2)** — `ToolThread` + `Toolbox`, que serve o executor injectado;
//! 3. **fachada** — `Ktu::builder()…build()`, que resolve a parte pura do projecto
//!    (guardrails, permissões, system prompt, orçamento) e corre o turno;
//! 4. **sessão** — o log é a sessão (I3); grava-se pela porta `Fs`.
//!
//! ```text
//! cargo run -p ktu-embed -- --backend scripted "explica o que fazes"
//! cargo run -p ktu-embed -- --base-url http://127.0.0.1:8080 --model local "olá"
//! ```
//!
//! Guia completo: `wiki/integration/` (índice em `wiki/integration/README.md`).

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::StopReason;
use ktu_core::facade::Ktu;
use ktu_core::plan::Mode;
use ktu_core::ports::{Clock, Level, LogRecord, Logger, NoopMetrics, Timestamp};
use ktu_core::provider::Provider;
use ktu_core::testing::EchoProvider;
use ktu_plugin::PluginHost;
use ktu_provider::{LlamaConfig, LlamaProvider};
use ktu_tools::{StdFs, ToolThread, Toolbox, default_registry, plan_mode_spec};
use tokio::runtime::Builder;

/// Relógio real do SO — o `ktu-core` nunca lê o relógio; a borda injecta-o.
#[derive(Debug)]
struct SystemClock {
    start: Instant,
}

impl SystemClock {
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de borda: base monótona real"
    )]
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de borda: relógio de parede real"
    )]
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        Timestamp::from_millis(i64::try_from(millis).unwrap_or(i64::MAX))
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "adaptador de borda: relógio monótono real"
    )]
    fn monotonic_micros(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
}

/// Logger de borda: uma linha por evento em `stderr` (**stdout é dado**).
///
/// O `event` é um id estável ([`ktu_core::events`]) — nunca texto livre. Numa
/// integração a sério, emita JSON (como o `ktu-cli` faz com `tracing`).
struct StderrLogger;

impl Logger for StderrLogger {
    fn log(&self, record: &LogRecord<'_>) {
        let level = match record.level {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
            Level::Trace => "trace",
        };
        let mut line = format!("[{level}] {}: {}", record.event, record.message);
        for (key, value) in record.fields {
            line.push(' ');
            line.push_str(key);
            line.push('=');
            line.push_str(value);
        }
        let mut stderr = io::stderr().lock();
        let _ignored = writeln!(stderr, "{line}");
    }
}

/// Backend do modelo: real (`llama-server`) ou scripted (sem rede).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Backend {
    Llama,
    Scripted,
}

/// Argumentos da linha de comandos (sem dependências: o exemplo é copiável).
struct Args {
    root: PathBuf,
    session: String,
    backend: Backend,
    mode: Mode,
    base_url: String,
    model: String,
    prompt: String,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut args = Self {
            root: PathBuf::from("."),
            session: "embed".to_owned(),
            backend: Backend::Scripted,
            mode: Mode::Build,
            base_url: ktu_provider::DEFAULT_BASE_URL.to_owned(),
            model: "local".to_owned(),
            prompt: String::new(),
        };
        let mut words: Vec<String> = Vec::new();
        let mut raw = std::env::args().skip(1);
        while let Some(arg) = raw.next() {
            match arg.as_str() {
                "--root" => args.root = PathBuf::from(next(&mut raw, "--root")?),
                "--session" => args.session = next(&mut raw, "--session")?,
                "--base-url" => args.base_url = next(&mut raw, "--base-url")?,
                "--model" => args.model = next(&mut raw, "--model")?,
                "--backend" => {
                    let value = next(&mut raw, "--backend")?;
                    args.backend = match value.as_str() {
                        "llama" => Backend::Llama,
                        "scripted" => Backend::Scripted,
                        other => return Err(format!("backend desconhecido: {other}")),
                    };
                }
                "--mode" => {
                    let value = next(&mut raw, "--mode")?;
                    args.mode =
                        Mode::parse(&value).ok_or_else(|| format!("modo desconhecido: {value}"))?;
                }
                other => words.push(other.to_owned()),
            }
        }
        args.prompt = words.join(" ");
        if args.prompt.is_empty() {
            return Err("uso: ktu-embed [--root DIR] [--session ID] [--backend llama|scripted] [--mode build|plan] [--base-url URL] [--model NOME] PROMPT".to_owned());
        }
        Ok(args)
    }
}

/// Lê o valor do argumento seguinte.
fn next(raw: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    raw.next().ok_or_else(|| format!("{flag} exige um valor"))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let mut stderr = io::stderr().lock();
            let _ignored = writeln!(stderr, "ktu-embed: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Cria os directórios de runtime do projecto.
///
/// A porta `Fs` do kernel **não** cria directórios: quem embute é que prepara o
/// terreno (a CLI faz o mesmo em `ensure_layout`).
fn ensure_layout(root: &Path) -> Result<(), String> {
    for dir in [
        root.join(".ktu").join("sessions"),
        root.join(".ktu").join("trash"),
        root.join(".ktu").join("plan"),
        root.join(".agents").join("memory"),
    ] {
        std::fs::create_dir_all(&dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    }
    Ok(())
}

/// Compõe tudo e corre **um** turno (o `main` só trata de `ExitCode`).
fn run() -> Result<(), String> {
    let args = Args::parse()?;
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("runtime: {error}"))?;
    runtime.block_on(turn(&args))
}

async fn turn(args: &Args) -> Result<(), String> {
    let root = args
        .root
        .canonicalize()
        .unwrap_or_else(|_| args.root.clone());
    ensure_layout(&root)?;
    let fs = StdFs;
    let clock = SystemClock::new();
    let logger = StderrLogger;
    let metrics = NoopMetrics;

    // ── T2: a thread de tools constrói os seus adaptadores dentro da thread ──
    let plan_mode = args.mode == Mode::Plan;
    let session = args.session.clone();
    let tool_thread = ToolThread::spawn({
        let root = root.clone();
        move |ctx| {
            let fs = StdFs;
            let clock = SystemClock::new();
            let mut toolbox = Toolbox::new(&root, &fs, &clock, None);
            if plan_mode {
                toolbox = toolbox.with_plan(ktu_core::plan::FsPlanHost::new(
                    &fs,
                    root.join(".ktu").join("plan"),
                    session,
                ));
            }
            ctx.serve(&toolbox);
        }
    })
    .map_err(|error| format!("thread de tools: {error}"))?;

    // ── T1: o provider e as portas de controlo ──
    // Sem modelo, o exemplo usa o **mesmo** provider de demonstração do
    // `ktu run --provider mock`: é o que torna a prova de M4 (mesmo prompt por
    // caminhos diferentes ⇒ log byte-idêntico) verificável entre CLI e crate.
    let scripted = EchoProvider::new();
    let llama = match args.backend {
        Backend::Llama => Some(
            LlamaProvider::new(LlamaConfig::new(args.base_url.clone(), args.model.clone()))
                .map_err(|error| format!("provider: {error}"))?,
        ),
        Backend::Scripted => None,
    };
    let provider: &dyn Provider = match &llama {
        Some(provider) => provider,
        None => &scripted,
    };
    // Hooks (§2.9): o exemplo liga o **mesmo** host de plugins do CLI (um manifesto
    // inválido desactiva o plugin, não a sessão) — sem ele o log não seria idêntico.
    let plugins = PluginHost::load(root.join(".ktu").join("plugins"));
    let registry = default_registry();
    let plan_spec = plan_mode_spec();
    let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
    let cancel = CancelToken::new();

    let ktu = Ktu::builder()
        .root(&root)
        .fs(&fs)
        .clock(&clock)
        .logger(&logger)
        .metrics(&metrics)
        .provider(provider)
        .tools(&tool_thread)
        .budget(&budget)
        .cancel(&cancel)
        .registry(&registry)
        .plan_spec(&plan_spec)
        .hooks(&plugins)
        .build()
        .map_err(|error| error.to_string())?;

    // ── A sessão é o log (I3): retoma-se se existir, grava-se no fim ──
    let mut log = ktu
        .session(&args.session)
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    // `build → plan` é sempre permitido; `plan → build` exige aprovação (I15),
    // por isso o exemplo só restringe (nunca escala o modo sozinho).
    if args.mode == Mode::Plan && log.mode() != Mode::Plan {
        log.set_mode(Mode::Plan);
    }
    let outcome = ktu
        .turn(&mut log, &args.prompt)
        .await
        .map_err(|error| error.to_string())?;
    ktu.save_session(&args.session, &log)
        .map_err(|error| error.to_string())?;

    // stdout = dado (a resposta final); stderr = log.
    let mut stdout = io::stdout().lock();
    let _ignored = writeln!(stdout, "{}", outcome.text);
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(
        stderr,
        "turno: {:?} ({} eventos, sessao em {})",
        outcome.reason,
        log.events().len(),
        ktu.sessions_dir()
            .join(format!("{}.jsonl", args.session))
            .display()
    );
    if outcome.reason == StopReason::Error {
        return Err("o turno terminou com erro (veja os eventos acima)".to_owned());
    }
    Ok(())
}
