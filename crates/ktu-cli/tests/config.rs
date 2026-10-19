//! Schema da config e migrações (F5): a leitura **não** destrói, a escrita carimba a
//! versão e uma versão futura é **fail-closed** (erro legível, nenhuma escrita).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-cfg-{tag}-{}-{id}", std::process::id()));
    let _ignored = fs::remove_dir_all(&root);
    root
}

/// Corre `ktu <args>` num projecto isolado, com o config do utilizador **dentro** da raiz.
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

/// Escreve um ficheiro (criando os directórios).
fn write(root: &Path, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let full = root.join(path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(full, content)?;
    Ok(())
}

/// Conteúdo do `config.toml` local.
fn config(root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(root.join(".agents/config.toml"))?)
}

/// Documento **v0**: sem versão e com a forma plana (CSV) de `deferred_tools`.
const V0: &str = "model = \"m\"\ndeferred_tools = \"bash, write_file\"\n";

#[test]
fn a_v0_config_is_read_without_being_rewritten() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("v0-leitura");
    write(&root, ".agents/config.toml", V0)?;

    // A leitura normaliza **em memória** (a forma plana v0 é aceite)…
    let output = ktu(&root, &["config", "get", "deferred_tools"])?;
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "bash,write_file"
    );
    // …e o ficheiro fica **intacto** (a leitura não destrói).
    assert_eq!(config(&root)?, V0);
    Ok(())
}

#[test]
fn doctor_reports_the_version_without_writing() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("doctor");
    write(&root, ".agents/config.toml", V0)?;

    let output = ktu(&root, &["config", "doctor", "--json"])?;
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(output.status.success(), "{text}");
    assert!(text.contains("\"schema\":0"), "{text}");
    assert!(text.contains("\"migrates\":true"), "{text}");
    assert_eq!(config(&root)?, V0, "o doctor nao escreve");

    let human = ktu(&root, &["config", "doctor"])?;
    let human_text = String::from_utf8_lossy(&human.stdout).into_owned();
    assert!(human_text.contains("schema=0"), "{human_text}");
    assert!(human_text.contains("migraria=sim"), "{human_text}");
    Ok(())
}

#[test]
fn a_write_migrates_once_and_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("escrita");
    write(&root, ".agents/config.toml", V0)?;

    // A 1.ª escrita migra (aviso em `stderr`, nunca silencioso) e carimba a versão.
    let first = ktu(&root, &["config", "set", "--local", "effort", "high"])?;
    assert!(first.status.success());
    let stderr = String::from_utf8_lossy(&first.stderr).into_owned();
    assert!(stderr.contains("migrada do schema v0 para v1"), "{stderr}");
    let after_first = config(&root)?;
    assert!(after_first.contains("schema = 1"), "{after_first}");
    assert!(
        after_first.contains("[\"bash\", \"write_file\"]"),
        "{after_first}"
    );

    // A 2.ª escrita não migra nada (idempotência) e só acrescenta a chave.
    let second = ktu(&root, &["config", "set", "--local", "max_steps", "3"])?;
    assert!(second.status.success());
    let stderr = String::from_utf8_lossy(&second.stderr).into_owned();
    assert!(!stderr.contains("migrada"), "{stderr}");
    assert!(config(&root)?.contains("schema = 1"));
    Ok(())
}

#[test]
fn a_future_version_is_refused_without_writing() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("futuro");
    let future = "schema = 3\nmodel = \"m\"\n";
    write(&root, ".agents/config.toml", future)?;

    // Ler e escrever recusam, com mensagem legível, e o ficheiro não é tocado.
    for args in [
        vec!["config", "get", "model"],
        vec!["config", "set", "--local", "model", "outro"],
        vec!["config", "doctor"],
    ] {
        let output = ktu(&root, &args)?;
        assert!(!output.status.success(), "{args:?} devia falhar");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(text.contains("schema v3"), "{args:?}: {text}");
        assert!(
            text.contains("nenhum ficheiro foi alterado"),
            "{args:?}: {text}"
        );
    }
    assert_eq!(config(&root)?, future);
    Ok(())
}
