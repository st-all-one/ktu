//! Integração do REPL `ktu session`: **M4** (dois caminhos, uma sessão) e gestão.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-session-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Corre `ktu <args>` no `root`, com `input` em `stdin`, isolado da config do utilizador.
fn ktu(root: &PathBuf, args: &[&str], input: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ktu"))
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = child.stdin.take().ok_or("sem stdin")?;
        stdin.write_all(input.as_bytes())?;
    }
    Ok(child.wait_with_output()?)
}

/// `stdout` de uma corrida, como texto.
fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn run_and_repl_produce_the_same_log() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("m4");
    fs::create_dir_all(&root)?;
    let first = ktu(
        &root,
        &[
            "run",
            "ola",
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            "a",
        ],
        "",
    )?;
    assert!(first.status.success());
    let second = ktu(
        &root,
        &["session", "b", "--provider", "mock", "--max-steps", "1"],
        "ola\n/quit\n",
    )?;
    assert!(second.status.success());
    let a = fs::read_to_string(root.join(".ktu/sessions/a.jsonl"))?;
    let b = fs::read_to_string(root.join(".ktu/sessions/b.jsonl"))?;
    assert_eq!(a, b, "M4: run e REPL devem produzir o mesmo log");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn ls_fork_and_rm_manage_sessions() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("manage");
    fs::create_dir_all(&root)?;
    let first = ktu(
        &root,
        &[
            "run",
            "ola",
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            "a",
        ],
        "",
    )?;
    assert!(first.status.success());

    let listed = ktu(&root, &["session", "ls"], "")?;
    assert!(listed.status.success());
    assert!(stdout(&listed).contains('a'), "ls: {}", stdout(&listed));

    let forked = ktu(&root, &["session", "fork", "a", "c"], "")?;
    assert!(forked.status.success());
    assert!(root.join(".ktu/sessions/c.jsonl").is_file());

    let removed = ktu(&root, &["session", "rm", "c"], "")?;
    assert!(removed.status.success());
    assert!(!root.join(".ktu/sessions/c.jsonl").exists());

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn slash_plan_changes_the_mode() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("plan");
    fs::create_dir_all(&root)?;
    let opened = ktu(
        &root,
        &["session", "p", "--provider", "mock", "--max-steps", "1"],
        "/plan\n/quit\n",
    )?;
    assert!(opened.status.success());
    let listed = ktu(&root, &["session", "ls"], "")?;
    assert!(listed.status.success());
    assert!(
        stdout(&listed).contains("Plan"),
        "modo: {}",
        stdout(&listed)
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_line_during_a_turn_is_admitted_and_delivered_next_time()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("steer");
    fs::create_dir_all(&root)?;
    let opened = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "4"],
        "ola\nfaz antes X\n/quit\n",
    )?;
    assert!(opened.status.success());
    let log = fs::read_to_string(root.join(".ktu/sessions/s.jsonl"))?;
    // M7: a linha é **admitida** (durável no log) com o id da borda.
    assert!(log.contains("\"event\":\"steer_queued\""), "{log}");
    assert!(log.contains("repl-1"), "{log}");
    assert!(
        !log.contains("\"event\":\"steer\""),
        "a entrega e no turno seguinte: {log}"
    );

    // Segundo arranque na **mesma** sessão: a admissão pendente entra na primeira fronteira.
    let reopened = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "4"],
        "continua\n/quit\n",
    )?;
    assert!(reopened.status.success());
    let log = fs::read_to_string(root.join(".ktu/sessions/s.jsonl"))?;
    assert!(log.contains("\"event\":\"steer\""), "{log}");
    assert!(log.contains("faz antes X"), "{log}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn ls_shows_the_title_derived_from_the_first_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("titulo");
    fs::create_dir_all(&root)?;
    let prompt = "arranja o bug do parser de frontmatter";
    let first = ktu(
        &root,
        &[
            "run",
            prompt,
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            "t",
        ],
        "",
    )?;
    assert!(first.status.success());

    // O checkpoint guarda o título (índice derivado do log).
    let checkpoint = fs::read_to_string(root.join(".ktu/sessions/t.checkpoint.json"))?;
    assert!(checkpoint.contains("arranja o bug do parser de frontmatter"));

    let listed = ktu(&root, &["session", "ls"], "")?;
    assert!(listed.status.success());
    assert!(
        stdout(&listed).contains("arranja o bug do parser de frontmatter"),
        "ls: {}",
        stdout(&listed)
    );

    let listed_json = ktu(&root, &["session", "ls", "--json"], "")?;
    assert!(listed_json.status.success());
    assert!(
        stdout(&listed_json).contains("\"title\":\"arranja o bug do parser de frontmatter\""),
        "ls --json: {}",
        stdout(&listed_json)
    );

    // O título é determinístico: retomar a sessão não o muda.
    let resumed = ktu(
        &root,
        &[
            "run",
            "outro pedido completamente diferente",
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            "t",
            "--resume",
        ],
        "",
    )?;
    assert!(resumed.status.success());
    let checkpoint = fs::read_to_string(root.join(".ktu/sessions/t.checkpoint.json"))?;
    assert!(checkpoint.contains("arranja o bug do parser de frontmatter"));

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
