//! Benchmark determinístico do núcleo (sem modelo real).
//!
//! Mede três caminhos quentes — a projeção (`derive_messages`), a construção do
//! log e um turno completo com provider/tools falsos — e escreve um artefacto
//! JSON com os números e a máquina. Corre com:
//!
//! ```text
//! cargo bench -p ktu-core --bench core
//! ```
//!
//! Saída: `wiki/benchmarks/ktu-bench.json` (um argumento opcional sobrepõe). Cada número
//! declara a **base de evidência** (`basis: measured`); ver `wiki/metodo-medicao.md`.

#![allow(
    clippy::disallowed_methods,
    reason = "benchmark: medir tempo real é o objectivo; a lógica medida (núcleo) é determinística"
)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ktu_core::agent::{Capabilities, Deps, LoopConfig, Ports, Telemetry, run_turn};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Completion, ToolCall, ToolCallId, ToolSpec};
use ktu_core::guard::StructuralGuard;
use ktu_core::ports::{FixedClock, NoopMetrics, RecordingLogger, Timestamp};
use ktu_core::session::{Event, SessionLog};
use ktu_core::testing::{FakeProvider, FakeTools};
use ktu_core::tools::ToolOutcome;
use tokio::runtime::{Builder, Runtime};

/// Amostras de duração (microssegundos), por nome, em ordem determinística.
#[derive(Default)]
struct Bench {
    samples: BTreeMap<&'static str, Vec<u64>>,
}

impl Bench {
    /// Regista uma observação.
    fn record(&mut self, name: &'static str, elapsed: Duration) {
        let micros = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        self.samples.entry(name).or_default().push(micros);
    }

    /// Vista serializável das amostras.
    fn json(&self) -> serde_json::Value {
        let mut samples = serde_json::Map::new();
        for (name, values) in &self.samples {
            samples.insert(
                (*name).to_owned(),
                serde_json::json!({
                    "basis": "measured",
                    "iterations": values.len(),
                    "min_micros": values.iter().copied().min().unwrap_or(0),
                    "median_micros": median(values),
                    "max_micros": values.iter().copied().max().unwrap_or(0),
                }),
            );
        }
        serde_json::Value::Object(samples)
    }
}

/// Mediana de uma amostra (determinística).
fn median(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted
        .get(sorted.len().saturating_div(2))
        .copied()
        .unwrap_or(0)
}

/// Um `ToolSpec` mínimo.
fn spec(name: &str) -> ToolSpec {
    ToolSpec {
        name: name.to_owned(),
        description: String::new(),
        parameters: "{}".to_owned(),
    }
}

/// Log sintético com `events` eventos (turno + pares `assistant`/`tool_result`).
fn synthetic_log(events: usize) -> SessionLog {
    let mut log = SessionLog::new();
    log.push(Event::TurnStart {
        prompt: "p".to_owned(),
    });
    let mut index: u64 = 0;
    while log.events().len() < events {
        let id = index.to_string();
        let call = ToolCall {
            id: ToolCallId::new(&id),
            name: "read_file".to_owned(),
            arguments: "{\"path\":\"a.txt\"}".to_owned(),
        };
        log.push(Event::Assistant {
            content: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call],
        });
        log.push(Event::ToolResult {
            tool_call_id: ToolCallId::new(&id),
            name: "read_file".to_owned(),
            content: "x".repeat(64),
            is_error: false,
        });
        index = index.saturating_add(1);
    }
    log
}

/// Mede a projeção `derive_messages` sobre um log de `events` eventos.
fn bench_projection(bench: &mut Bench, name: &'static str, events: usize, iterations: u32) {
    let log = synthetic_log(events);
    for _ in 0..iterations {
        let start = Instant::now();
        let messages = log.derive_messages();
        let elapsed = start.elapsed();
        std::hint::black_box(&messages);
        bench.record(name, elapsed);
    }
}

/// Mede a construção do log sintético de `events` eventos.
fn bench_log_build(bench: &mut Bench, name: &'static str, events: usize, iterations: u32) {
    for _ in 0..iterations {
        let start = Instant::now();
        let log = synthetic_log(events);
        let elapsed = start.elapsed();
        std::hint::black_box(&log);
        bench.record(name, elapsed);
    }
}

/// Mede um turno completo com `steps` chamadas de tool (provider/tools falsos).
fn bench_turn(rt: &Runtime, bench: &mut Bench, name: &'static str, steps: u32, iterations: u32) {
    for _ in 0..iterations {
        let mut script = Vec::new();
        for step in 0..steps {
            script.push(Completion {
                text: String::new(),
                reasoning: String::new(),
                tool_calls: vec![ToolCall {
                    id: ToolCallId::new(step.to_string()),
                    name: "read_file".to_owned(),
                    arguments: "{\"path\":\"a.txt\"}".to_owned(),
                }],
            });
        }
        script.push(Completion {
            text: "feito".to_owned(),
            reasoning: String::new(),
            tool_calls: Vec::new(),
        });

        let mut responses = Vec::new();
        for _ in 0..steps {
            responses.push(ToolOutcome::ok("conteudo"));
        }

        let provider = FakeProvider::new(script);
        let executor = FakeTools::new(vec![spec("read_file")], responses);
        let guard = StructuralGuard::new("/r", ["read_file".to_owned()], 1);
        let budget = BudgetGate::new(1_000_000);
        let logger = RecordingLogger::new();
        let metrics = NoopMetrics;
        let clock = FixedClock::new(Timestamp::from_millis(0));
        let cancel = CancelToken::new();
        let deps = Deps {
            ports: Ports {
                provider: &provider,
                tools: &executor,
                guard: &guard,
                budget: &budget,
                cancel: &cancel,
            },
            telemetry: Telemetry {
                logger: &logger,
                metrics: &metrics,
                clock: &clock,
            },
            capabilities: Capabilities {
                memory: None,
                subagents: None,
                plugins: None,
                steering: None,
                instructions: None,
            },
        };
        let config = LoopConfig {
            max_steps: steps.saturating_add(4),
            ..LoopConfig::default()
        };
        let mut log = SessionLog::new();

        let start = Instant::now();
        let reason = rt.block_on(run_turn(&mut log, &deps, "sys", "le a.txt", &config));
        let elapsed = start.elapsed();
        std::hint::black_box(&reason);
        bench.record(name, elapsed);
    }
}

/// Escreve texto em `stdout`, ignorando falhas (não é a API pública do binário).
fn emit(text: &str) {
    let mut stdout = std::io::stdout();
    stdout.write_all(text.as_bytes()).ok();
    stdout.flush().ok();
}

/// Tabela legível das amostras.
fn emit_table(bench: &Bench) {
    let mut text = String::new();
    writeln!(
        text,
        "{:<24} {:>6} {:>12} {:>12} {:>12}",
        "benchmark", "n", "min_us", "median_us", "max_us"
    )
    .ok();
    for (name, values) in &bench.samples {
        let min = values.iter().copied().min().unwrap_or(0);
        let max = values.iter().copied().max().unwrap_or(0);
        writeln!(
            text,
            "{name:<24} {:>6} {min:>12} {:>12} {max:>12}",
            values.len(),
            median(values)
        )
        .ok();
    }
    emit(&text);
}

/// Caminho do artefacto (1.º argumento livre ou `wiki/benchmarks/ktu-bench.json`).
fn artifact_path() -> PathBuf {
    // `cargo bench` passa `--bench`; ignora qualquer argumento com `-`.
    if let Some(path) = std::env::args().skip(1).find(|arg| !arg.starts_with('-')) {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../wiki/benchmarks/ktu-bench.json")
}

/// Modelo do CPU (melhor esforço, `/proc/cpuinfo`).
fn cpu_model() -> Option<String> {
    let info = fs::read_to_string("/proc/cpuinfo").ok()?;
    let line = info.lines().find(|line| line.starts_with("model name"))?;
    Some(line.split_once(':')?.1.trim().to_owned())
}

/// Versão do `rustc` (melhor esforço).
fn rustc_version() -> Option<String> {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|text| text.trim().to_owned())
}

/// Pico de RSS do processo (KiB; melhor esforço, Linux).
fn peak_rss_kib() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    line.strip_prefix("VmHWM:")?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// Tamanho do binário de release `ktu`, se já estiver construído (bytes).
fn binary_size() -> Option<u64> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/ktu");
    fs::metadata(path).ok().map(|meta| meta.len())
}

/// Escreve o artefacto JSON (cria o diretório se preciso).
fn write_artifact(value: &serde_json::Value) -> std::io::Result<()> {
    let path = artifact_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value).unwrap_or_default();
    fs::write(path, json)
}

fn main() {
    let Ok(rt) = Builder::new_current_thread().enable_all().build() else {
        emit("bench: não foi possível criar o runtime tokio\n");
        return;
    };

    let mut bench = Bench::default();
    bench_projection(&mut bench, "derive_messages_100", 100, 500);
    bench_projection(&mut bench, "derive_messages_1k", 1_000, 200);
    bench_projection(&mut bench, "derive_messages_10k", 10_000, 50);
    bench_log_build(&mut bench, "log_build_1k", 1_000, 200);
    bench_turn(&rt, &mut bench, "turn_1_step", 1, 200);
    bench_turn(&rt, &mut bench, "turn_8_steps", 8, 100);

    let artifact = serde_json::json!({
        "schema": 2,
        "tool": "cargo bench -p ktu-core --bench core",
        "host": {
            "arch": std::env::consts::ARCH,
            "os": std::env::consts::OS,
            "cpu": cpu_model(),
            "rustc": rustc_version(),
        },
        "samples": bench.json(),
        // `basis` (método de medição): tudo aqui é `measured` — o que não for, não entra.
        "memory": { "basis": "measured", "peak_rss_kib": peak_rss_kib() },
        "binary": { "basis": "measured", "ktu_release_bytes": binary_size() },
    });

    emit_table(&bench);
    match write_artifact(&artifact) {
        Ok(()) => emit(&format!(
            "bench: artefacto em {}\n",
            artifact_path().display()
        )),
        Err(err) => emit(&format!("bench: falha ao escrever o artefacto: {err}\n")),
    }
}
