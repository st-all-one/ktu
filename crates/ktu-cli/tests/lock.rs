//! Integração do lock de turno e do modo efémero, via o binário `ktu`.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-lock-it-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Corre `ktu run` com o provider `mock`, isolado da config do utilizador.
fn run_mock(root: &PathBuf, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_ktu"))
        .arg("run")
        .arg("ola")
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .output()?;
    Ok(output)
}

#[test]
fn a_live_lock_refuses_a_second_process() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("refuse");
    let sessions = root.join(".ktu").join("sessions");
    fs::create_dir_all(&sessions)?;
    // Timestamp no futuro ⇒ o lock nunca expira (vivo).
    fs::write(sessions.join("s1.lock"), "9999999999999\n")?;

    let output = run_mock(
        &root,
        &["--provider", "mock", "--session", "s1", "--max-steps", "1"],
    )?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ocupada"), "stderr: {stderr}");

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_released_lock_allows_the_next_run() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("release");
    fs::create_dir_all(&root)?;

    let first = run_mock(
        &root,
        &["--provider", "mock", "--session", "s1", "--max-steps", "1"],
    )?;
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    // O lock foi largado: a segunda corrida passa e o log fica gravado.
    let second = run_mock(
        &root,
        &["--provider", "mock", "--session", "s1", "--max-steps", "1"],
    )?;
    assert!(second.status.success());
    assert!(root.join(".ktu/sessions/s1.jsonl").is_file());
    assert!(!root.join(".ktu/sessions/s1.lock").exists());

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn an_ephemeral_run_does_not_persist() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("ephemeral");
    fs::create_dir_all(&root)?;

    let output = run_mock(&root, &["--provider", "mock", "--max-steps", "1"])?;
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.join(".ktu/sessions").exists());

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
