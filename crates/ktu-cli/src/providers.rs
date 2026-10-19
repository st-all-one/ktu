//! `ktu providers doctor` — imprime o diagnóstico do provider (stdout = dados).

use std::path::Path;
use std::process::ExitCode;

use ktu_provider::{DoctorReport, Probe, diagnose};
use tokio::runtime::Builder;

use crate::cli::ProvidersAction;
use crate::output::{self, Format};

/// Executa a acção de `ktu providers`.
pub(crate) fn run(action: ProvidersAction, format: Format) -> ExitCode {
    match action {
        ProvidersAction::Doctor {
            base_url,
            gguf,
            probe,
        } => {
            let choice = if probe { Probe::Run } else { Probe::Skip };
            doctor(&base_url, gguf.as_deref(), choice, format)
        }
    }
}

/// Corre o diagnóstico e imprime o relatório em `stdout`.
pub(crate) fn doctor(
    base_url: &str,
    gguf: Option<&Path>,
    probe: Probe,
    format: Format,
) -> ExitCode {
    let runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            output::emit_error(
                format,
                "providers doctor",
                "io",
                &format!("runtime: {error}"),
            );
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(diagnose(base_url, gguf, probe)) {
        Ok(report) => {
            output::emit(
                format,
                "providers doctor",
                &report_text(&report, gguf),
                &report_data(&report, gguf),
            );
            if report.healthy() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            output::emit_error(format, "providers doctor", "io", &message);
            ExitCode::FAILURE
        }
    }
}

/// Relatório em texto humano (uma linha por campo).
fn report_text(report: &DoctorReport, gguf: Option<&Path>) -> String {
    let mut lines = vec![
        format!("base_url: {}", report.base_url),
        format!("reachable: {}", report.reachable),
        format!("models: {}", report.models.join(", ")),
        format!(
            "model_path: {}",
            report.model_path.as_deref().unwrap_or("-")
        ),
        format!(
            "n_ctx: {}",
            report
                .n_ctx
                .map_or_else(|| "-".to_owned(), |n| n.to_string())
        ),
    ];
    if let Some(path) = gguf {
        lines.push(format!("gguf: {}", path.display()));
    }
    if let Some(meta) = &report.gguf {
        lines.push(format!("gguf.version: {}", meta.version));
        lines.push(format!(
            "gguf.architecture: {}",
            meta.architecture.as_deref().unwrap_or("-")
        ));
        lines.push(format!(
            "gguf.file_type: {}",
            meta.file_type
                .map_or_else(|| "-".to_owned(), |t| t.to_string())
        ));
    }
    if report.ternary_pack {
        lines.push("nota: pack ternary — exige o fork PrismML-Eng/llama.cpp".to_owned());
    }
    if let Some(ok) = report.probe_ok {
        lines.push(format!("probe_ok: {ok}"));
    }
    if report.healthy() {
        lines.push("estado: ok".to_owned());
    } else {
        for problem in &report.problems {
            lines.push(format!("problema: {problem}"));
        }
        lines.push("estado: problemas".to_owned());
    }
    lines.join("\n")
}

/// Relatório como dado (envelope JSON).
fn report_data(report: &DoctorReport, gguf: Option<&Path>) -> serde_json::Value {
    serde_json::json!({
        "base_url": report.base_url,
        "reachable": report.reachable,
        "models": report.models,
        "model_path": report.model_path,
        "n_ctx": report.n_ctx,
        "probe_ok": report.probe_ok,
        "ternary_pack": report.ternary_pack,
        "healthy": report.healthy(),
        "problems": report.problems,
        "gguf": gguf.map(|path| path.display().to_string()),
    })
}
