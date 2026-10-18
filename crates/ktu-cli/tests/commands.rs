//! Integração dos comandos do projecto: `ktu commands ls` e `/nome` na sessão.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Raiz temporária única por teste (o processo é partilhado).
fn fresh_root(tag: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ktu-commands-{tag}-{}-{id}", std::process::id()));
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

/// Projecto com um comando válido, um sem frontmatter e um malformado.
fn project(tag: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let root = fresh_root(tag);
    fs::create_dir_all(root.join(".agents/commands"))?;
    fs::write(
        root.join(".agents/commands/revisao.md"),
        "---\nname: revisao\ndescription: Reve o diff\n---\n\nReve $ARGUMENTS com cuidado.\n",
    )?;
    fs::write(root.join(".agents/commands/plano.md"), "Faz um plano.\n")?;
    fs::write(
        root.join(".agents/commands/mau.md"),
        "---\nname: com espaco\n---\n\nx\n",
    )?;
    Ok(root)
}

#[test]
fn commands_ls_lists_the_project_commands() -> Result<(), Box<dyn std::error::Error>> {
    let root = project("ls")?;
    let listed = ktu(&root, &["commands", "ls"], "")?;
    assert!(listed.status.success());
    let text = stdout(&listed);
    assert!(text.contains("revisao\tReve o diff"), "{text}");
    assert!(text.contains("plano"), "{text}");
    // O ficheiro malformado é ignorado e reportado em `stderr`.
    assert!(stderr(&listed).contains("mau.md"), "{}", stderr(&listed));

    let json = ktu(&root, &["commands", "ls", "--json"], "")?;
    assert!(json.status.success());
    assert!(
        stdout(&json).contains("\"name\":\"revisao\""),
        "{}",
        stdout(&json)
    );

    let empty_root = fresh_root("vazio");
    fs::create_dir_all(&empty_root)?;
    let empty = ktu(&empty_root, &["commands", "ls"], "")?;
    assert!(empty.status.success());
    assert_eq!(stdout(&empty).trim(), "sem comandos");

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn a_project_command_runs_as_a_turn_with_rendered_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let root = project("run")?;
    let run = ktu(
        &root,
        &["session", "c", "--provider", "mock", "--max-steps", "1"],
        "/revisao src/main.rs\n/quit\n",
    )?;
    assert!(run.status.success(), "{}", stderr(&run));
    let log = fs::read_to_string(root.join(".ktu/sessions/c.jsonl"))?;
    assert!(log.contains("Reve src/main.rs com cuidado."), "{log}");
    // O comando sem frontmatter usa o corpo todo e acrescenta os argumentos.
    let run = ktu(
        &root,
        &["session", "d", "--provider", "mock", "--max-steps", "1"],
        "/plano com urgencia\n/quit\n",
    )?;
    assert!(run.status.success(), "{}", stderr(&run));
    let log = fs::read_to_string(root.join(".ktu/sessions/d.jsonl"))?;
    assert!(log.contains("Faz um plano.\\n\\ncom urgencia"), "{log}");

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn help_lists_the_project_commands_and_unknown_ones_are_reported()
-> Result<(), Box<dyn std::error::Error>> {
    let root = project("help")?;
    let help = ktu(
        &root,
        &["session", "h", "--provider", "mock", "--max-steps", "1"],
        "/help\n/quit\n",
    )?;
    assert!(help.status.success());
    assert!(stderr(&help).contains("/revisao"), "{}", stderr(&help));
    assert!(stderr(&help).contains("/quit"), "{}", stderr(&help));

    let unknown = ktu(
        &root,
        &["session", "u", "--provider", "mock", "--max-steps", "1"],
        "/nao-existe\n/quit\n",
    )?;
    assert!(unknown.status.success());
    assert!(
        stderr(&unknown).contains("comando desconhecido: /nao-existe"),
        "{}",
        stderr(&unknown)
    );

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn prompt_shows_the_effective_system_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("prompt");
    fs::create_dir_all(root.join(".agents"))?;
    fs::write(root.join(".agents/SYSTEM.md"), "persona do projeto\n")?;
    fs::write(root.join(".agents/APPEND_SYSTEM.md"), "extra do projeto\n")?;
    fs::write(root.join("AGENTS.md"), "regra da raiz\n")?;
    fs::create_dir_all(root.join("sub"))?;
    fs::write(root.join("sub/AGENTS.md"), "regra local\n")?;
    // Directriz do utilizador: o projecto ganha (homónimos não se combinam).
    fs::create_dir_all(root.join("xdg/ktu"))?;
    fs::write(root.join("xdg/ktu/SYSTEM.md"), "persona do utilizador\n")?;

    let shown = ktu(&root, &["prompt"], "")?;
    assert!(shown.status.success(), "{}", stderr(&shown));
    let text = stdout(&shown);
    assert!(text.starts_with("persona do projeto"), "{text}");
    assert!(!text.contains("persona do utilizador"), "{text}");
    assert!(text.contains("## sub/AGENTS.md"), "{text}");
    assert!(text.ends_with("extra do projeto\n"), "{text}");

    let json = ktu(&root, &["prompt", "--json"], "")?;
    assert!(json.status.success());
    assert!(stdout(&json).contains("\"system\""), "{}", stdout(&json));

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn reading_a_file_attaches_the_nearest_agents_md() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("locais");
    fs::create_dir_all(root.join("sub"))?;
    fs::write(root.join("AGENTS.md"), "regra da raiz\n")?;
    fs::write(root.join("sub/AGENTS.md"), "regra local do modulo\n")?;
    fs::write(root.join("sub/a.md"), "conteudo\n")?;

    let run = ktu(
        &root,
        &[
            "run",
            "le sub/a.md",
            "--provider",
            "mock",
            "--max-steps",
            "2",
            "--session",
            "i",
        ],
        "",
    )?;
    assert!(run.status.success(), "{}", stderr(&run));
    let log = fs::read_to_string(root.join(".ktu/sessions/i.jsonl"))?;
    assert!(
        log.contains("[instrucoes locais: sub/AGENTS.md]\\nregra local do modulo"),
        "{log}"
    );
    assert!(
        !log.contains("regra da raiz"),
        "o mais proximo ganha: {log}"
    );

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn run_command_applies_settings_and_guarded_shell() -> Result<(), Box<dyn std::error::Error>> {
    let root = fresh_root("recipe");
    fs::create_dir_all(root.join(".agents/commands"))?;
    fs::write(
        root.join(".agents/commands/revisao.md"),
        "---\nname: revisao\ndescription: Reve\nprovider: mock\nmax_steps: 1\nshell: true\n---\n\n\
         Reve $1 com cuidado.\n!echo \"tudo limpo\"\n",
    )?;

    let run = ktu(
        &root,
        &[
            "run",
            "--command",
            "revisao",
            "src/main.rs",
            "--session",
            "r",
        ],
        "",
    )?;
    assert!(run.status.success(), "{}", stderr(&run));
    let log = fs::read_to_string(root.join(".ktu/sessions/r.jsonl"))?;
    assert!(log.contains("Reve src/main.rs com cuidado."), "{log}");
    assert!(log.contains("[shell: echo \\\"tudo limpo\\\"]"), "{log}");
    assert!(log.contains("tudo limpo"), "{log}");
    // `max_steps: 1` da receita foi aplicado (o turno fecha no limite).
    assert!(log.contains("\"reason\":\"max_steps\""), "{log}");

    // Fail-closed: com `bash` negado, o comando **não** corre.
    let denied = ktu(&root, &["permission", "set", "bash", "never"], "")?;
    assert!(denied.status.success());
    let refused = ktu(
        &root,
        &["run", "--command", "revisao", "x", "--session", "d"],
        "",
    )?;
    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains("shell negado"),
        "{}",
        stderr(&refused)
    );
    assert!(!root.join(".ktu/sessions/d.jsonl").exists());

    // Comando desconhecido: erro accionável, sem turno.
    let unknown = ktu(
        &root,
        &["run", "--command", "nao-existe", "--provider", "mock"],
        "",
    )?;
    assert!(!unknown.status.success());
    assert!(
        stderr(&unknown).contains("comando desconhecido: nao-existe"),
        "{}",
        stderr(&unknown)
    );

    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
