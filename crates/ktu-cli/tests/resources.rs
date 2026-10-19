//! Recursos globais do utilizador e o **piso visível** (M8): `~/.config/ktu/`.
//!
//! Cada listagem diz de onde vem cada recurso efectivo; a precedência é declarada:
//! guardrails e plugins **somam**, skills e receitas **substituem por nome**.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-res-{tag}-{}-{id}", std::process::id()));
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

/// `stdout` de uma corrida bem sucedida, como texto.
fn ok_stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
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

/// Um `SKILL.md` mínimo.
fn skill(description: &str, body: &str) -> String {
    format!("---\nname: x\ndescription: {description}\n---\n\n{body}\n")
}

#[test]
fn skills_ls_shows_both_levels_with_local_winning() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("skills");
    fs::create_dir_all(&root)?;
    write(
        &root,
        "xdg/ktu/skills/rust/SKILL.md",
        &skill("rust global", "# global"),
    )?;
    write(
        &root,
        "xdg/ktu/skills/kd/SKILL.md",
        &skill("kd global", "# kd"),
    )?;
    write(
        &root,
        ".agents/skills/rust/SKILL.md",
        &skill("rust local", "# local"),
    )?;

    let text = ok_stdout(&ktu(&root, &["skills", "ls"])?);
    assert!(text.contains("kd\tkd global\tglobal"), "{text}");
    assert!(text.contains("rust\trust local\tlocal"), "{text}");
    assert!(
        !text.contains("rust global"),
        "o local devia ganhar:\n{text}"
    );

    let json = ok_stdout(&ktu(&root, &["skills", "ls", "--json"])?);
    assert!(json.contains("\"origin\":\"global\""), "{json}");
    assert!(json.contains("\"origin\":\"local\""), "{json}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn guardrails_list_shows_the_global_floor_with_origin() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("guardrails");
    fs::create_dir_all(&root)?;
    write(
        &root,
        "xdg/ktu/guardrails/deny-env.md",
        "---\nid: deny-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n",
    )?;
    write(
        &root,
        ".agents/guardrails/deny-bash.md",
        "---\nid: deny-bash\nversion: 1\nfact: tool\nmatch: bash\ndecision: deny\nreason: sem shell\n---\n",
    )?;

    let text = ok_stdout(&ktu(&root, &["guardrails", "list"])?);
    let global = text.find("deny-env\tglobal");
    let local = text.find("deny-bash\tlocal");
    assert!(global.is_some(), "{text}");
    assert!(local.is_some(), "{text}");
    // O piso global vem primeiro: é ele que o projecto não pode desligar (M5).
    assert!(global < local, "{text}");

    let json = ok_stdout(&ktu(&root, &["guardrails", "list", "--json"])?);
    assert!(
        json.contains("\"id\":\"deny-env\",\"origin\":\"global\""),
        "{json}"
    );
    assert!(
        json.contains("\"id\":\"deny-bash\",\"origin\":\"local\""),
        "{json}"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn commands_ls_shows_global_recipes_and_the_local_one_wins()
-> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("recipes");
    fs::create_dir_all(&root)?;
    write(
        &root,
        "xdg/ktu/recipes/revisao.md",
        "---\nname: revisao\ndescription: global\nmax_steps: 9\n---\n\nrevê global\n",
    )?;
    write(
        &root,
        "xdg/ktu/recipes/so-global.md",
        "---\nname: so-global\ndescription: so global\n---\n\ncorre\n",
    )?;
    // O local ganha **e** impõe a sua configuração (`max_steps: 2`).
    write(
        &root,
        ".agents/commands/revisao.md",
        "---\nname: revisao\ndescription: local\nmax_steps: 2\n---\n\nrevê local\n",
    )?;

    let text = ok_stdout(&ktu(&root, &["commands", "ls"])?);
    assert!(text.contains("revisao\tlocal\tlocal"), "{text}");
    assert!(text.contains("so-global\tso global\tglobal"), "{text}");

    // A receita local ganha também no **runtime**: o `max_steps` efectivo é o dela.
    let _run = ok_stdout(&ktu(
        &root,
        &["run", "/revisao", "--provider", "mock", "--session", "s"],
    )?);
    let log = fs::read_to_string(root.join(".ktu/sessions/s.jsonl"))?;
    assert_eq!(log.matches("\"event\":\"assistant\"").count(), 2, "{log}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn plugins_list_shows_both_levels_additively() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("plugins");
    fs::create_dir_all(&root)?;
    for (base, name) in [
        ("xdg/ktu/plugins", "g1"),
        ("xdg/ktu/plugins", "g2"),
        (".ktu/plugins", "g1"),
    ] {
        write(
            &root,
            &format!("{base}/{name}/plugin.toml"),
            &format!(
                "name = \"{name}\"\nentrypoint = \"./run.sh\"\nevents = [\"user.input\"]\nmode = \"observe\"\ntimeout_ms = 2000\n"
            ),
        )?;
        write(
            &root,
            &format!("{base}/{name}/run.sh"),
            "#!/bin/sh\nread -r line\nprintf '{\"verdict\":\"pass\"}\\n'\n",
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                root.join(format!("{base}/{name}/run.sh")),
                fs::Permissions::from_mode(0o755),
            )?;
        }
    }

    let text = ok_stdout(&ktu(&root, &["plugins", "list"])?);
    assert!(text.contains("g1\tglobal"), "{text}");
    assert!(text.contains("g2\tglobal"), "{text}");
    // O mesmo nome no projecto **não** desliga o global: aparecem os dois (aditivo).
    assert!(text.contains("g1\tlocal"), "{text}");
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn the_prompt_composes_the_effective_skills() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("prompt");
    fs::create_dir_all(&root)?;
    write(
        &root,
        "xdg/ktu/skills/rust/SKILL.md",
        &skill("global", "protocolo global"),
    )?;
    write(
        &root,
        ".agents/skills/rust/SKILL.md",
        &skill("local", "protocolo local"),
    )?;

    let prompt = ok_stdout(&ktu(&root, &["prompt"])?);
    assert!(prompt.contains("# Skills"), "{prompt}");
    assert!(prompt.contains("## rust — local"), "{prompt}");
    assert!(prompt.contains("protocolo local"), "{prompt}");
    assert!(
        !prompt.contains("protocolo global"),
        "o local devia ganhar:\n{prompt}"
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
