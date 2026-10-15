//! `ktu providers doctor` — imprime o diagnóstico do provider (stdout = dados).

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use ktu_provider::{Probe, diagnose};
use tokio::runtime::Builder;

/// Corre o diagnóstico e imprime o relatório em `stdout`.
pub(crate) fn doctor(base_url: &str, gguf: Option<&Path>, probe: Probe) -> ExitCode {
    let runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            report_error(&format!("runtime: {error}"));
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(diagnose(base_url, gguf, probe)) {
        Ok(report) => {
            print_report(&report, gguf);
            if report.healthy() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            report_error(&message);
            ExitCode::FAILURE
        }
    }
}

fn print_report(report: &ktu_provider::DoctorReport, gguf: Option<&Path>) {
    let mut stdout = io::stdout().lock();
    let _ignored = writeln!(stdout, "base_url: {}", report.base_url);
    let _ignored = writeln!(stdout, "reachable: {}", report.reachable);
    let _ignored = writeln!(stdout, "models: {}", report.models.join(", "));
    let _ignored = writeln!(
        stdout,
        "model_path: {}",
        report.model_path.as_deref().unwrap_or("-")
    );
    let _ignored = writeln!(
        stdout,
        "n_ctx: {}",
        report
            .n_ctx
            .map_or_else(|| "-".to_owned(), |n| n.to_string())
    );
    if let Some(path) = gguf {
        let _ignored = writeln!(stdout, "gguf: {}", path.display());
    }
    if let Some(meta) = &report.gguf {
        let _ignored = writeln!(stdout, "gguf.version: {}", meta.version);
        let _ignored = writeln!(
            stdout,
            "gguf.architecture: {}",
            meta.architecture.as_deref().unwrap_or("-")
        );
        let _ignored = writeln!(
            stdout,
            "gguf.file_type: {}",
            meta.file_type
                .map_or_else(|| "-".to_owned(), |t| t.to_string())
        );
    }
    if report.ternary_pack {
        let _ignored = writeln!(
            stdout,
            "nota: pack ternary — exige o fork PrismML-Eng/llama.cpp"
        );
    }
    if let Some(ok) = report.probe_ok {
        let _ignored = writeln!(stdout, "probe_ok: {ok}");
    }
    if report.healthy() {
        let _ignored = writeln!(stdout, "estado: ok");
    } else {
        for problem in &report.problems {
            let _ignored = writeln!(stdout, "problema: {problem}");
        }
        let _ignored = writeln!(stdout, "estado: problemas");
    }
}

fn report_error(message: &str) {
    let mut stderr = io::stderr().lock();
    let _ignored = writeln!(stderr, "ktu: {message}");
}
