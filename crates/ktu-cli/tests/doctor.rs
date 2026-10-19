//! `ktu doctor` — diagnóstico da estrutura do projecto (o que o `ktu init` cria) e o
//! diagnóstico da memória com porta própria (`ktu memory doctor`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-doc-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    let _ignored = fs::create_dir_all(&root);
    root
}

/// Corre `ktu <args>` num projecto isolado.
fn ktu(root: &Path, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let child = Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    Ok(child.wait_with_output()?)
}

/// Texto de uma corrida (stdout + stderr).
fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn an_empty_project_points_to_init() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("vazio");
    let output = ktu(&root, &["doctor"])?;
    let text = output_text(&output);
    assert!(!output.status.success(), "{text}");
    assert!(text.contains("layout: FALTA"), "{text}");
    assert!(text.contains("memory_source: FALTA"), "{text}");
    assert!(text.contains("config: FALTA"), "{text}");
    assert!(text.contains("corre `ktu init`"), "{text}");
    // As outras áreas têm porta própria e são apontadas.
    assert!(text.contains("ktu memory doctor"), "{text}");
    assert!(text.contains("ktu config doctor"), "{text}");
    Ok(())
}

#[test]
fn after_init_the_project_is_healthy() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("init");
    let init = ktu(&root, &["init", "--no-prompt"])?;
    assert!(init.status.success(), "{}", output_text(&init));

    let output = ktu(&root, &["doctor", "--json"])?;
    let text = output_text(&output);
    assert!(output.status.success(), "{text}");
    assert!(text.contains("\"healthy\":true"), "{text}");
    assert!(text.contains("\"name\":\"layout\""), "{text}");
    assert!(text.contains("\"name\":\"config\""), "{text}");
    assert!(text.contains("\"ok\":true"), "{text}");
    Ok(())
}

#[test]
fn memory_doctor_has_its_own_door() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("memoria");
    assert!(ktu(&root, &["init", "--no-prompt"])?.status.success());
    let output = ktu(&root, &["memory", "doctor", "--json"])?;
    let text = output_text(&output);
    assert!(output.status.success(), "{text}");
    assert!(text.contains("\"command\":\"memory doctor\""), "{text}");
    assert!(text.contains("\"healthy\":true"), "{text}");
    Ok(())
}
