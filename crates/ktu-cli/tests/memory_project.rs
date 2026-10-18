//! Integração de `ktu memory project`: o log de sessão vira **notas derivadas** (H7).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-project-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Corre `ktu <args>` no `root`, isolado da config do utilizador.
fn ktu(root: &Path, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?)
}

/// Cria uma sessão persistente com um turno (mock determinístico).
fn session(root: &Path, id: &str, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out = ktu(
        root,
        &[
            "run",
            prompt,
            "--provider",
            "mock",
            "--max-steps",
            "4",
            "--session",
            id,
        ],
    )?;
    assert!(out.status.success(), "run falhou: {out:?}");
    Ok(())
}

/// Linhas do `session-log.jsonl` derivado.
fn derived(root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(
        root.join(".agents/memory/session-log.jsonl"),
    )?)
}

#[test]
fn project_writes_derived_notes_with_provenance() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("notes");
    fs::create_dir_all(&root)?;
    session(&root, "a", "onde vive a memoria do agente")?;

    let out = ktu(&root, &["memory", "project", "--root", "."])?;
    assert!(out.status.success(), "{out:?}");
    let text = derived(&root)?;
    assert_eq!(text.lines().count(), 1);
    assert!(text.contains("\"type\":\"meta\""), "{text}");
    assert!(text.contains("\"activity\":\"derive\""), "{text}");
    assert!(text.contains("\"agent\":\"ktu\""), "{text}");
    assert!(text.contains("\"tags\":[\"session-log\"]"), "{text}");
    assert!(
        text.contains("\"statement\":\"[a] onde vive a memoria do agente\""),
        "{text}"
    );
    assert!(text.contains("\"anchors\":[\"AGENTS.md\"]"), "{text}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn project_is_idempotent_and_uses_the_json_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("idem");
    fs::create_dir_all(&root)?;
    session(&root, "a", "primeiro turno")?;

    let first = ktu(&root, &["memory", "project", "--root", ".", "--json"])?;
    assert!(first.status.success(), "{first:?}");
    let data: serde_json::Value = serde_json::from_slice(&first.stdout)?;
    assert_eq!(
        data.get("data").and_then(|v| v.get("written")),
        Some(&serde_json::json!(1))
    );

    let second = ktu(&root, &["memory", "project", "--root", ".", "--json"])?;
    let data: serde_json::Value = serde_json::from_slice(&second.stdout)?;
    assert_eq!(
        data.get("data").and_then(|v| v.get("written")),
        Some(&serde_json::json!(0))
    );
    assert_eq!(
        derived(&root)?.lines().count(),
        1,
        "re-projectar nao duplica"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn project_covers_all_sessions_and_filters_by_session() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("all");
    fs::create_dir_all(&root)?;
    session(&root, "a", "primeira sessao")?;
    session(&root, "b", "segunda sessao")?;

    let all = ktu(&root, &["memory", "project", "--root", "."])?;
    assert!(all.status.success(), "{all:?}");
    assert_eq!(derived(&root)?.lines().count(), 2);

    let filtered = ktu(
        &root,
        &[
            "memory",
            "project",
            "--root",
            ".",
            "--session",
            "a",
            "--out",
            "so-a.jsonl",
        ],
    )?;
    assert!(filtered.status.success(), "{filtered:?}");
    let text = fs::read_to_string(root.join("so-a.jsonl"))?;
    assert_eq!(text.lines().count(), 1);
    assert!(text.contains("[a] primeira sessao"), "{text}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn project_missing_session_fails() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("missing");
    fs::create_dir_all(&root)?;
    let out = ktu(
        &root,
        &["memory", "project", "--root", ".", "--session", "nope"],
    )?;
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("nope"),
        "{out:?}"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
