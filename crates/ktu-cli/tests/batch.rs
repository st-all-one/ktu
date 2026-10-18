//! Integração do `--batch`: contexto acumulado em `--sequential` (G10/G13) e
//! isolamento por item no modo paralelo (uso #2).

use std::fs;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> std::path::PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-batch-{tag}-{}-{id}", std::process::id()));
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

/// Escreve um ficheiro dentro do `root` (criando o `root`).
fn write(root: &Path, name: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root)?;
    fs::write(root.join(name), text)?;
    Ok(())
}

#[test]
fn sequential_batch_accumulates_context_in_one_session() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("seq");
    write(&root, "a.txt", "conteudo\n")?;
    write(
        &root,
        "batch.jsonl",
        "{\"prompt\":\"le a.txt\"}\n{\"prompt\":\"le a.txt\"}\n{\"prompt\":\"le a.txt\"}\n",
    )?;
    let output = ktu(
        &root,
        &[
            "run",
            "--batch",
            "batch.jsonl",
            "--sequential",
            "--session",
            "g",
            "--provider",
            "mock",
            "--max-steps",
            "2",
        ],
    )?;
    assert!(output.status.success(), "batch falhou");
    let log = fs::read_to_string(root.join(".ktu/sessions/g.jsonl"))?;
    // "Uma sessão, N turnos": os três turnos ficam no **mesmo** log, por ordem.
    assert_eq!(
        log.matches("\"event\":\"turn_start\"").count(),
        3,
        "os turnos nao se acumularam: {log}"
    );
    let first = log.find("\"prompt\":\"le a.txt\"").unwrap_or(usize::MAX);
    let last = log.rfind("\"prompt\":\"le a.txt\"").unwrap_or(0);
    assert!(first < last, "ordem dos turnos perdida");
    // E o contexto acumula: o 2.º turno vê o resultado de tool do 1.º.
    assert!(
        log.matches("\"event\":\"tool_result\"").count() >= 3,
        "cada turno deve trazer o seu resultado de tool"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn parallel_batch_isolates_each_item_and_orders_the_output()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("par");
    write(&root, "a.txt", "conteudo\n")?;
    write(
        &root,
        "batch.jsonl",
        "{\"prompt\":\"le a.txt\"}\n{\"prompt\":\"le a.txt\"}\n{\"prompt\":\"le a.txt\"}\n",
    )?;
    let output = ktu(
        &root,
        &[
            "run",
            "--batch",
            "batch.jsonl",
            "--provider",
            "mock",
            "--max-steps",
            "2",
        ],
    )?;
    assert!(output.status.success(), "batch paralelo falhou");
    // `stdout` = dados, ordenados pelo índice de entrada (determinismo).
    let stdout = String::from_utf8_lossy(&output.stdout);
    let indexes: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split(']').next())
        .collect();
    assert_eq!(indexes, ["[0", "[1", "[2"], "saida fora de ordem: {stdout}");
    // Cada item corre na sua própria sessão (sem corrupção do log).
    for index in 0..3 {
        let path = root.join(format!(".ktu/sessions/batch-{index}.jsonl"));
        let log = fs::read_to_string(&path)?;
        assert_eq!(log.matches("\"event\":\"turn_start\"").count(), 1);
    }
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
