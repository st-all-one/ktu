//! Integração das permissões por tool (`.agents/permissions.toml`) e do veredicto
//! declarativo `escalate` — a política é **dado**, não obediência do modelo.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-perm-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Corre `ktu <args>` no `root`, isolado da config do utilizador.
fn ktu(root: &PathBuf, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .stdin(Stdio::null())
        .output()?;
    Ok(output)
}

/// `stdout` de uma corrida, como texto.
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Corre o mock (que pede `read_file`) com passos suficientes para o avaliar.
fn run_mock(root: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let output = ktu(
        root,
        &[
            "run",
            "le AGENTS.md",
            "--provider",
            "mock",
            "--max-steps",
            "3",
        ],
    )?;
    assert!(output.status.success());
    Ok(stdout(&output))
}

#[test]
fn permission_never_denies_and_ask_escalates() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("modes");
    fs::create_dir_all(&root)?;

    let set = ktu(&root, &["permission", "set", "read_file", "never"])?;
    assert!(set.status.success());
    let denied = run_mock(&root)?;
    assert!(denied.contains("negado: policy"), "{denied}");

    let _ask = ktu(&root, &["permission", "set", "read_file", "ask"])?;
    let escalated = run_mock(&root)?;
    assert!(escalated.contains("requer aprovacao"), "{escalated}");

    let _unset = ktu(&root, &["permission", "unset", "read_file"])?;
    let allowed = run_mock(&root)?;
    assert!(!allowed.contains("negado"), "{allowed}");

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_declarative_rule_escalates_with_its_own_remedy() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("escalate");
    let dir = root.join(".agents").join("guardrails");
    fs::create_dir_all(&dir)?;
    let rule = "---\nid: ask-read\nversion: 1\napplies_to: '*'\nfact: tool\nmatch: read_file\ndecision: escalate\nreason: leitura sensivel\nremedy: pede aprovacao ao utilizador\n---\n";
    fs::write(dir.join("ask-read.md"), rule)?;

    let escalated = run_mock(&root)?;
    assert!(
        escalated.contains("requer aprovacao: pede aprovacao ao utilizador"),
        "{escalated}"
    );

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn permissions_survive_resume() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("resume");
    fs::create_dir_all(&root)?;
    let _set = ktu(&root, &["permission", "set", "read_file", "never"])?;

    let first = ktu(
        &root,
        &[
            "run",
            "le AGENTS.md",
            "--provider",
            "mock",
            "--max-steps",
            "3",
            "--session",
            "s",
        ],
    )?;
    assert!(first.status.success());
    assert!(stdout(&first).contains("negado: policy"));

    let resumed = ktu(
        &root,
        &[
            "run",
            "le AGENTS.md",
            "--provider",
            "mock",
            "--max-steps",
            "3",
            "--session",
            "s",
            "--resume",
        ],
    )?;
    assert!(resumed.status.success());
    assert!(stdout(&resumed).contains("negado: policy"));

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
