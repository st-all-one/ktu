//! Integração da fiabilidade de edição (F4): âncora errada sugere as linhas próximas
//! e o remédio da regra acompanha o estrutural.
//!
//! O caminho é o **real**: `ktu run` com um guião determinístico (o modelo pede o
//! `edit_file`); o resultado da tool fica no log da sessão.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-edit-{tag}-{}-{id}", std::process::id()));
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

/// Escreve um ficheiro dentro do `root` (criando os directórios).
fn write(root: &Path, name: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full = root.join(name);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(full, text)?;
    Ok(())
}

/// Guião de um passo: o modelo pede `edit_file` com os argumentos dados.
fn edit_script(root: &Path, path: &str, old: &str, new: &str) -> Result<String, String> {
    let arguments = serde_json::json!({
        "path": path,
        "edits": [{ "old": old, "new": new }],
    });
    let line = serde_json::json!({
        "text": "",
        "tool_calls": [{ "id": "c1", "name": "edit_file", "arguments": arguments.to_string() }],
    });
    let file = root.join("guiao.jsonl");
    fs::write(&file, format!("{line}\n{{\"text\":\"fim\"}}\n"))
        .map_err(|error| error.to_string())?;
    Ok(file.to_string_lossy().into_owned())
}

#[test]
fn a_wrong_anchor_suggests_the_closest_lines() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("ancora");
    write(
        &root,
        "alvo.txt",
        "# Titulo\n\nfn soma(a: u32) -> u32 {\n    a + 1\n}\n",
    )?;
    let script = edit_script(&root, "alvo.txt", "fn somma(a: u32) -> u32 {", "x")?;
    let output = ktu(
        &root,
        &[
            "run",
            "edita",
            "--provider",
            "scripted",
            "--script",
            &script,
            "--session",
            "s",
            "--max-steps",
            "2",
        ],
    )?;
    assert!(output.status.success());
    let log = fs::read_to_string(root.join(".ktu/sessions/s.jsonl"))?;
    assert!(log.contains("texto nao encontrado"), "{log}");
    // A sugestão traz a linha certa (a 3) com o texto.
    assert!(log.contains("mais proximas"), "{log}");
    assert!(log.contains("3: fn soma(a: u32) -> u32 {"), "{log}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_missing_anchor_leaves_the_file_untouched() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("intacto");
    let original = "# Titulo\n\nfn soma(a: u32) -> u32 {\n    a + 1\n}\n";
    write(&root, "alvo.txt", original)?;
    let script = edit_script(&root, "alvo.txt", "nao existe de todo", "x")?;
    let output = ktu(
        &root,
        &[
            "run",
            "edita",
            "--provider",
            "scripted",
            "--script",
            &script,
            "--session",
            "s",
            "--max-steps",
            "2",
        ],
    )?;
    assert!(output.status.success());
    let after = fs::read_to_string(root.join("alvo.txt"))?;
    assert_eq!(after, original, "a edicao falhada alterou o ficheiro");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
