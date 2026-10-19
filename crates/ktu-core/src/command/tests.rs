//! Testes dos comandos do projecto (parse, render, carga e *shell*).

use std::path::Path;

use super::{ARGUMENTS, Command, CommandSettings, ShellAccess, find, load, load_layered, parse};
use crate::origin::Origin;
use crate::ports::MemFs;

const DIR: &str = "/proj/.agents/commands";

fn fixture() -> MemFs {
    MemFs::new()
        .with_file(
            Path::new(DIR).join("revisao.md"),
            "---\nname: revisao\ndescription: Revê o diff\n---\n\nRevê $ARGUMENTS com cuidado.\n",
        )
        .with_file(Path::new(DIR).join("sem-frontmatter.md"), "Faz isto.\n")
        .with_file(Path::new(DIR).join("vazio.md"), "---\nname: vazio\n---\n\n")
        .with_file(
            Path::new(DIR).join("mau.md"),
            "---\nname: com espaco\n---\n\nx\n",
        )
        .with_file(Path::new(DIR).join("notas.txt"), "ignorado")
}

#[test]
fn parses_frontmatter_and_body() {
    let command = parse(
        "revisao.md",
        "---\nname: revisao\ndescription: 'Reve'\n---\n\nfaz $ARGUMENTS\n",
    )
    .ok()
    .flatten();
    assert_eq!(command.as_ref().map(Command::name), Some("revisao"));
    assert_eq!(command.as_ref().map(Command::description), Some("Reve"));
    assert_eq!(
        command.as_ref().map(Command::prompt),
        Some("faz $ARGUMENTS")
    );
}

#[test]
fn renders_arguments_with_and_without_the_marker() {
    let command = parse("x.md", "---\nname: x\n---\n\nCorre $ARGUMENTS agora")
        .ok()
        .flatten();
    assert_eq!(
        command
            .as_ref()
            .map(|command| command.render("a b"))
            .as_deref(),
        Some("Corre a b agora")
    );

    let plain = parse("y.md", "Faz a tarefa").ok().flatten();
    assert_eq!(
        plain
            .as_ref()
            .map(|command| command.render("extra"))
            .as_deref(),
        Some("Faz a tarefa\n\nextra")
    );
    assert_eq!(
        plain
            .as_ref()
            .map(|command| command.render("  "))
            .as_deref(),
        Some("Faz a tarefa")
    );
}

#[test]
fn load_is_deterministic_and_reports_problems() {
    let (commands, problems) = load(&fixture(), Path::new(DIR));
    let names: Vec<&str> = commands.iter().map(Command::name).collect();
    assert_eq!(names, ["revisao", "sem-frontmatter"]);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems.iter().any(|item| item.contains("vazio.md")));
    assert!(problems.iter().any(|item| item.contains("mau.md")));
    assert!(find(&commands, "revisao").is_some());
    assert!(find(&commands, "nao-existe").is_none());
}

#[test]
fn a_missing_directory_is_not_an_error() {
    let (commands, problems) = load(&MemFs::new(), Path::new(DIR));
    assert_eq!(commands, Vec::new());
    assert_eq!(problems, Vec::<String>::new());
}

#[test]
fn arguments_marker_is_stable() {
    assert_eq!(ARGUMENTS, "$ARGUMENTS");
}

#[test]
fn positional_arguments_are_substituted() {
    let command = parse("x.md", "---\nname: x\n---\n\nReve $1 e $2 ($1 de novo)")
        .ok()
        .flatten();
    assert_eq!(
        command
            .as_ref()
            .map(|command| command.render("src/main.rs \"dois ficheiros.rs\""))
            .as_deref(),
        Some("Reve src/main.rs e dois ficheiros.rs (src/main.rs de novo)")
    );
    // Sem marcadores posicionais nem `$ARGUMENTS`, os argumentos vão no fim.
    let plain = parse("y.md", "Faz isto").ok().flatten();
    assert_eq!(
        plain
            .as_ref()
            .map(|command| command.render("a b"))
            .as_deref(),
        Some("Faz isto\n\na b")
    );
}

#[test]
fn settings_are_parsed_and_validated() {
    let command = parse(
        "x.md",
        "---\nname: x\nprovider: mock\nmodel: eco\nbase_url: http://x\nmax_steps: 3\n---\n\ncorpo",
    )
    .ok()
    .flatten();
    let settings = command.as_ref().map(|command| command.settings().clone());
    let settings = settings.unwrap_or_default();
    assert_eq!(settings.provider.as_deref(), Some("mock"));
    assert_eq!(settings.model.as_deref(), Some("eco"));
    assert_eq!(settings.base_url.as_deref(), Some("http://x"));
    assert_eq!(settings.max_steps, Some(3));
    assert!(!settings.is_empty());
    assert!(CommandSettings::default().is_empty());
    assert!(parse("x.md", "---\nname: x\nmax_steps: muitos\n---\n\ncorpo").is_err());
}

#[test]
fn shell_is_opt_in_and_segments_are_listed() {
    let prompt = "---\nname: x\nshell: true\n---\n\nFaz isto:\n!echo oi\n!ls -la\n";
    let command = parse("x.md", prompt).ok().flatten();
    assert_eq!(
        command.as_ref().map(Command::shell),
        Some(ShellAccess::Guarded)
    );
    assert_eq!(
        command
            .as_ref()
            .map(Command::shell_segments)
            .unwrap_or_default(),
        ["echo oi", "ls -la"]
    );

    // Sem `shell: true`, `!…` é texto normal — nada executa por surpresa.
    let off = parse("y.md", "---\nname: y\n---\n\nFaz isto:\n!echo oi")
        .ok()
        .flatten();
    assert_eq!(
        off.as_ref().map(Command::shell),
        Some(ShellAccess::Disabled)
    );
    assert_eq!(
        off.as_ref()
            .map(Command::shell_segments)
            .unwrap_or_default(),
        Vec::<String>::new()
    );
    assert!(parse("z.md", "---\nname: z\nshell: talvez\n---\n\ncorpo").is_err());
}

#[test]
fn layered_load_gives_local_precedence_by_name() {
    let fs = MemFs::new()
        .with_file(
            "/u/recipes/revisao.md",
            "---
description: global
---

global
",
        )
        .with_file(
            "/u/recipes/so-global.md",
            "---
description: g
---

so global
",
        )
        .with_file(
            Path::new(DIR).join("revisao.md"),
            "---
description: local
---

local
",
        );
    let (commands, problems) = load_layered(&fs, Some(Path::new("/u")), Path::new("/proj"));
    assert!(problems.is_empty(), "{problems:?}");
    let names: Vec<&str> = commands
        .iter()
        .map(|(_origin, command)| command.name())
        .collect();
    assert_eq!(names, vec!["revisao", "so-global"]);
    // O local ganha: uma só `revisao`, com o corpo local.
    assert_eq!(
        commands
            .first()
            .map(|(origin, command)| (*origin, command.prompt())),
        Some((Origin::Local, "local"))
    );
    assert_eq!(
        commands.get(1).map(|(origin, _command)| *origin),
        Some(Origin::Global)
    );
}
