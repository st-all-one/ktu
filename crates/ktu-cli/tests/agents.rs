//! Integração dos agentes declarativos (`.agents/agents/*.md`).

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-agents-{tag}-{}-{id}", std::process::id()));
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

/// `stderr` de uma corrida, como texto.
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn agents_ls_lists_the_declarative_agents() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("ls");
    fs::create_dir_all(root.join(".agents/agents"))?;
    fs::write(
        root.join(".agents/agents/revisor.md"),
        "---\nname: revisor\nmode: subagent\ntools: {\"*\": false, \"read_*\": true}\n---\n",
    )?;
    fs::write(
        root.join(".agents/agents/chefe.md"),
        "---\nname: chefe\nmode: primary\ntools: read_file, bash\n---\n",
    )?;

    let listed = ktu(&root, &["agents", "ls"], "")?;
    assert!(listed.status.success(), "{}", stderr(&listed));
    let text = stdout(&listed);
    assert!(text.contains("revisor\tsubagent\tread_file"), "{text}");
    assert!(text.contains("chefe\tprimary\tbash,read_file"), "{text}");

    let json = ktu(&root, &["agents", "ls", "--json"], "")?;
    assert!(json.status.success());
    assert!(
        stdout(&json).contains("\"name\":\"revisor\""),
        "{}",
        stdout(&json)
    );

    // Fail-closed: um agente inválido impede o arranque e nomeia o ficheiro.
    fs::write(
        root.join(".agents/agents/mau.md"),
        "---\nname: mau\nmode: chefe\ntools: read_file\n---\n",
    )?;
    let broken = ktu(&root, &["agents", "ls"], "")?;
    assert!(!broken.status.success());
    assert!(stderr(&broken).contains("mau.md"), "{}", stderr(&broken));
    let run = ktu(
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
    assert!(!run.status.success(), "um agente invalido impede o turno");
    assert!(
        stderr(&run).contains("agentes invalidos"),
        "{}",
        stderr(&run)
    );

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_declarative_primary_restricts_tools_structurally() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("estrutural");
    fs::create_dir_all(root.join(".agents/agents"))?;
    // Agente primário declarativo **sem** `read_file`: a tool é negada pelo guard.
    fs::write(
        root.join(".agents/agents/sem-leitura.md"),
        "---\nname: sem-leitura\nmode: primary\ntools: {\"*\": false, \"list_dir\": true}\n---\n",
    )?;
    let run = ktu(
        &root,
        &[
            "run",
            "le AGENTS.md",
            "--provider",
            "mock",
            "--max-steps",
            "2",
            "--session",
            "p",
        ],
        "",
    )?;
    assert!(run.status.success(), "{}", stderr(&run));
    let log = fs::read_to_string(root.join(".ktu/sessions/p.jsonl"))?;
    assert!(log.contains("negado: capability"), "{log}");
    assert!(log.contains("\"is_error\":true"), "{log}");

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
