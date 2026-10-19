//! Integração dos comandos `/` do REPL e do Ctrl-C (F2 — REPL interactivo).
//!
//! Cada comando **delega** num verbo que já existe (`ktu session fork`, `ktu memory list`,
//! `--compact`, a porta `Host`); o Ctrl-C chama a porta `cancel` em vez de matar o processo.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-repl-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Comando `ktu` isolado (config do utilizador fora do projecto).
fn command(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ktu"));
    command
        .args(args)
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .env("HOME", root.join("home"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

/// Corre `ktu <args>` no `root`, com `input` em `stdin`.
fn ktu(root: &Path, args: &[&str], input: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = command(root, args).spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes())?;
    }
    drop(child.stdin.take());
    Ok(child.wait_with_output()?)
}

/// `stderr` de uma corrida, como texto.
fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Escreve um ficheiro do projecto (criando os directórios).
fn write(root: &Path, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full = root.join(path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(full, content)?;
    Ok(())
}

/// Um turno numa sessão persistente (cria o log que os comandos de gestão precisam).
fn seed_turn(root: &Path, session: &str, prompt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args = if root.join(format!(".ktu/sessions/{session}.jsonl")).exists() {
        vec![
            "run",
            prompt,
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            session,
            "--resume",
        ]
    } else {
        vec![
            "run",
            prompt,
            "--provider",
            "mock",
            "--max-steps",
            "1",
            "--session",
            session,
        ]
    };
    let output = ktu(root, &args, "")?;
    assert!(output.status.success(), "{}", stderr(&output));
    Ok(())
}

#[test]
fn help_lists_the_new_commands() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("help");
    fs::create_dir_all(&root)?;
    let output = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "1"],
        "/help\n/quit\n",
    )?;
    assert!(output.status.success());
    let text = stderr(&output);
    for command in ["/compact", "/fork", "/memory", "/resume"] {
        assert!(text.contains(command), "{command} em falta:\n{text}");
    }
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn memory_lists_the_project_notes() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("memory");
    fs::create_dir_all(&root)?;
    write(
        &root,
        ".agents/memory/notes.jsonl",
        "{\"body\":\"corpo\",\"frontmatter\":{\"body_hash\":\"abc\",\"created_at\":\"2026-10-18T00:00:00.000Z\",\"id\":\"fact_abcabcab\",\"provenance\":{\"activity\":\"write\",\"agent\":\"ktu\"},\"schema_version\":2,\"statement\":\"a memoria funciona\",\"tags\":[\"teste\"],\"type\":\"fact\"}}\n",
    )?;
    let output = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "1"],
        "/memory\n/quit\n",
    )?;
    assert!(output.status.success());
    let text = stderr(&output);
    assert!(text.contains("fact_abcabcab"), "{text}");
    assert!(text.contains("a memoria funciona"), "{text}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn fork_copies_the_session_and_resume_switches() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("fork");
    fs::create_dir_all(&root)?;
    seed_turn(&root, "s", "ola")?;
    let output = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "1"],
        "/fork copia\n/resume\n/quit\n",
    )?;
    assert!(output.status.success());
    let text = stderr(&output);
    assert!(text.contains("sessao copiada: s -> copia"), "{text}");
    // A lista sai ordenada por id (`session_cmd::ids`), não pela ordem de criação.
    let listing = text
        .lines()
        .find(|line| line.starts_with("sessoes: "))
        .unwrap_or_default();
    assert!(listing.contains("copia") && listing.contains('s'), "{text}");
    assert!(root.join(".ktu/sessions/copia.jsonl").is_file());

    // `/resume copia` muda de sessão: o turno seguinte escreve no log da sessão nova.
    let output = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "1"],
        "/resume copia\nola de novo\n/quit\n",
    )?;
    assert!(output.status.success());
    let text = stderr(&output);
    assert!(text.contains("sessao: copia"), "{text}");
    let copied = fs::read_to_string(root.join(".ktu/sessions/copia.jsonl"))?;
    assert!(copied.contains("ola de novo"), "{copied}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn compact_writes_the_cut_to_the_checkpoint() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("compact");
    fs::create_dir_all(&root)?;
    seed_turn(&root, "s", "um")?;
    seed_turn(&root, "s", "dois")?;
    seed_turn(&root, "s", "tres")?;

    let output = ktu(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "1"],
        "/compact\n/quit\n",
    )?;
    assert!(output.status.success());
    let checkpoint = fs::read_to_string(root.join(".ktu/sessions/s.checkpoint.json"))?;
    assert!(checkpoint.contains("\"compacted\": true"), "{checkpoint}");
    assert!(!checkpoint.contains("\"keep_from\": null"), "{checkpoint}");
    // O log cru fica intacto (a compactação é uma projeção).
    let log = fs::read_to_string(root.join(".ktu/sessions/s.jsonl"))?;
    assert_eq!(log.matches("\"turn_start\"").count(), 3, "{log}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

/// Ctrl-C durante um turno: o processo **não** morre e o turno fecha `Cancelled`.
///
/// O turno é tornado lento por um plugin `observe` (2 s) no `model.request` — sem
/// inferência e sem depender do *timing* do provider.
#[test]
fn ctrl_c_cancels_the_turn_without_killing_the_process() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("ctrl-c");
    fs::create_dir_all(&root)?;
    write(
        &root,
        ".ktu/plugins/slow/plugin.toml",
        "name = \"slow\"\nentrypoint = \"./run.sh\"\nevents = [\"model.request\"]\nmode = \"observe\"\ntimeout_ms = 6000\n",
    )?;
    write(
        &root,
        ".ktu/plugins/slow/run.sh",
        "#!/bin/sh\nsleep 2\necho '{\"verdict\":\"pass\"}'\n",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            root.join(".ktu/plugins/slow/run.sh"),
            fs::Permissions::from_mode(0o755),
        )?;
    }

    let mut child: Child = command(
        &root,
        &["session", "s", "--provider", "mock", "--max-steps", "4"],
    )
    .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(b"ola\n")?;
    }
    std::thread::sleep(Duration::from_millis(1000));
    let killed = Command::new("kill")
        .arg("-INT")
        .arg(child.id().to_string())
        .status()?;
    assert!(killed.success(), "nao foi possivel enviar SIGINT");
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(b"/quit\n")?;
    }
    drop(child.stdin.take());
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "o processo morreu com Ctrl-C: {}",
        stderr(&output)
    );
    let log = fs::read_to_string(root.join(".ktu/sessions/s.jsonl"))?;
    assert!(log.contains("\"reason\":\"cancelled\""), "{log}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
