//! Testes das skills declarativas (dois níveis, M8).

use std::path::Path;

use super::{Skill, load, section};
use crate::origin::Origin;
use crate::ports::MemFs;

/// Um `SKILL.md` com o frontmatter **escalar** do ktu.
fn scalar(name: &str, body: &str) -> String {
    format!("---\nname: {name}\ndescription: protocolo {name}\n---\n\n{body}\n")
}

/// Um `SKILL.md` com o frontmatter do pi/knudge (valores dobrados e listas).
const FOLDED: &str = "---\nname: rust\ndescription: >\n  Rust 1.99.1: ownership,\n  async/Tokio, macros.\ncategory: languages\ntags: [rust, ownership]\n---\n\n# Rust\n\n## Use When\n- escrever Rust\n";

fn names(skills: &[Skill]) -> Vec<String> {
    skills
        .iter()
        .map(|skill| skill.name().to_owned())
        .collect::<Vec<_>>()
}

#[test]
fn loads_both_levels_with_local_winning_by_name() {
    let fs = MemFs::new()
        .with_file("/u/skills/rust/SKILL.md", scalar("rust", "global"))
        .with_file("/u/skills/kd/SKILL.md", scalar("kd", "global kd"))
        .with_file("/p/.agents/skills/rust/SKILL.md", scalar("rust", "local"));
    let (skills, problems) = load(&fs, Some(Path::new("/u")), Path::new("/p"));
    assert_eq!(names(&skills), vec!["kd".to_owned(), "rust".to_owned()]);
    assert!(problems.is_empty(), "{problems:?}");
    // O local ganha: um só `rust`, com o corpo local e a origem local.
    let rust = skills.iter().find(|skill| skill.name() == "rust");
    assert_eq!(rust.map(Skill::body), Some("local"));
    assert_eq!(rust.map(Skill::origin), Some(Origin::Local));
    let kd = skills.iter().find(|skill| skill.name() == "kd");
    assert_eq!(kd.map(Skill::origin), Some(Origin::Global));
}

#[test]
fn without_a_user_dir_only_the_project_counts() {
    let fs = MemFs::new().with_file("/p/.agents/skills/rust/SKILL.md", scalar("rust", "local"));
    let (skills, _problems) = load(&fs, None, Path::new("/p"));
    assert_eq!(names(&skills), vec!["rust".to_owned()]);
    assert_eq!(skills.first().map(Skill::origin), Some(Origin::Local));
}

#[test]
fn reads_the_folded_frontmatter_of_a_pi_skill() -> Result<(), String> {
    let fs = MemFs::new().with_file("/p/.agents/skills/rust/SKILL.md", FOLDED);
    let (skills, problems) = load(&fs, None, Path::new("/p"));
    assert!(problems.is_empty(), "{problems:?}");
    let skill = skills.first().ok_or("esperava uma skill")?;
    assert_eq!(
        skill.description(),
        "Rust 1.99.1: ownership, async/Tokio, macros."
    );
    // O frontmatter não entra no corpo.
    assert!(skill.body().starts_with("# Rust"), "{}", skill.body());
    assert!(!skill.body().contains("description"));
    Ok(())
}

#[test]
fn a_directory_without_the_file_is_a_problem_not_a_failure() {
    let fs = MemFs::new()
        .with_dir("/p/.agents/skills/vazia")
        .with_file("/p/.agents/skills/rust/SKILL.md", scalar("rust", "ok"));
    let (skills, problems) = load(&fs, None, Path::new("/p"));
    assert_eq!(names(&skills), vec!["rust".to_owned()]);
    assert_eq!(problems.len(), 1);
    assert!(problems.first().is_some_and(|item| item.contains("vazia")));
}

#[test]
fn a_file_above_the_cap_is_ignored_with_a_warning() {
    let big = format!("---\nname: grande\n---\n\n{}", "x".repeat(9 * 1024));
    let fs = MemFs::new()
        .with_file("/p/.agents/skills/grande/SKILL.md", big)
        .with_file("/p/.agents/skills/rust/SKILL.md", scalar("rust", "ok"));
    let (skills, problems) = load(&fs, None, Path::new("/p"));
    assert_eq!(names(&skills), vec!["rust".to_owned()]);
    assert!(problems.first().is_some_and(|item| item.contains("tecto")));
}

#[test]
fn the_section_is_none_without_skills_and_lists_them_by_name() -> Result<(), String> {
    assert_eq!(section(&[]), None);
    let fs = MemFs::new()
        .with_file("/p/.agents/skills/b/SKILL.md", scalar("b", "corpo b"))
        .with_file("/p/.agents/skills/a/SKILL.md", scalar("a", "corpo a"));
    let (skills, _problems) = load(&fs, None, Path::new("/p"));
    let text = section(&skills).ok_or("esperava uma seccao")?;
    assert!(text.starts_with("# Skills"));
    assert!(text.contains("## a — protocolo a\n\ncorpo a"));
    let (Some(first), Some(second)) = (text.find("## a"), text.find("## b")) else {
        return Err("esperava as duas skills".to_owned());
    };
    assert!(first < second, "ordem instavel:\n{text}");
    Ok(())
}
