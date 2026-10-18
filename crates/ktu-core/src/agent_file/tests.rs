//! Testes dos agentes declarativos (política de tools, carga e aplicação ao registo).

use std::collections::BTreeMap;
use std::path::Path;

use super::{AgentFile, AgentMode, ToolAccess, ToolPolicy, apply, load, parse};
use crate::agent::{AgentRegistry, AgentSpec};
use crate::guard::AgentId;
use crate::ports::MemFs;

const DIR: &str = "/proj/.agents/agents";

fn catalog() -> Vec<String> {
    ["read_file", "write_file", "glob", "bash", "subagent"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect()
}

/// Política a partir de uma especificação (nega tudo em caso de erro).
fn policy(spec: &str) -> ToolPolicy {
    ToolPolicy::parse(spec).unwrap_or_else(|_| ToolPolicy::new(ToolAccess::Deny, BTreeMap::new()))
}

/// Agente a partir do texto (vazio em caso de erro).
fn agent(file: &str, text: &str) -> Option<AgentFile> {
    parse(file, text).ok()
}

/// Registo default do teste (primário com o catálogo + `explorer`).
fn registry() -> AgentRegistry {
    AgentRegistry::new(
        AgentSpec::new(AgentId::Primary, catalog(), 1),
        Some(AgentSpec::new(
            AgentId::Explorer,
            vec!["read_file".to_owned()],
            0,
        )),
    )
}

#[test]
fn parses_the_map_syntax_and_the_allow_list() {
    let map = policy("{\"*\": false, \"read_*\": true, \"bash\": true}");
    assert!(map.allows("read_file"), "padrao com * casa");
    assert!(map.allows("bash"));
    assert!(!map.allows("write_file"));
    assert!(!map.allows("subagent"));

    let list = policy("read_file, glob");
    assert_eq!(
        list.resolve(&catalog()),
        vec!["read_file".to_owned(), "glob".to_owned()]
    );
    assert!(ToolPolicy::parse("").is_err());
    assert!(ToolPolicy::parse("{\"*\": talvez}").is_err());
}

#[test]
fn exact_rules_win_over_wildcards() {
    let mixed = policy("{\"*\": true, \"read_*\": false, \"read_file\": true}");
    assert!(mixed.allows("read_file"), "exacto ganha ao padrao");
    assert!(!mixed.allows("read_dir"));
    assert!(mixed.allows("bash"));
    assert_eq!(mixed.unknown(&catalog()), Vec::<String>::new());

    let typo = policy("{\"*\": false, \"read_fil\": true}");
    assert_eq!(typo.unknown(&catalog()), vec!["read_fil".to_owned()]);
}

#[test]
fn parses_agents_and_rejects_bad_ones() {
    let good = agent(
        "rev.md",
        "---\nname: rev\nmode: subagent\ntools: {\"*\": false, \"read_file\": true}\n---\n",
    );
    assert_eq!(good.as_ref().map(AgentFile::name), Some("rev"));
    assert_eq!(
        good.as_ref().map(AgentFile::mode),
        Some(AgentMode::Subagent)
    );

    assert!(agent("x.md", "sem frontmatter").is_none());
    assert!(agent("x.md", "---\nname: ktu\ntools: read_file\n---\n").is_none());
    assert!(agent("x.md", "---\nname: a b\ntools: read_file\n---\n").is_none());
    assert!(agent("x.md", "---\nname: a\nmode: chefe\ntools: read_file\n---\n").is_none());
    assert!(
        agent("x.md", "---\nname: a\n---\n").is_none(),
        "tools em falta"
    );
}

#[test]
fn load_is_fail_closed_and_deterministic() {
    let fs = MemFs::new()
        .with_file(
            Path::new(DIR).join("rev.md"),
            "---\nname: rev\ntools: {\"*\": false, \"read_file\": true}\n---\n",
        )
        .with_file(
            Path::new(DIR).join("chefe.md"),
            "---\nname: chefe\nmode: primary\ntools: read_file, bash, subagent\n---\n",
        )
        .with_file(Path::new(DIR).join("notas.txt"), "ignorado");
    let agents = load(&fs, Path::new(DIR)).unwrap_or_default();
    let names: Vec<&str> = agents.iter().map(AgentFile::name).collect();
    assert_eq!(names, ["chefe", "rev"]);

    let broken = MemFs::new().with_file(
        Path::new(DIR).join("mau.md"),
        "---\nname: mau\nmode: errado\ntools: read_file\n---\n",
    );
    assert!(
        load(&broken, Path::new(DIR))
            .err()
            .is_some_and(|error| error.contains("mau.md"))
    );
    assert_eq!(
        load(&MemFs::new(), Path::new(DIR)).unwrap_or_default(),
        Vec::new()
    );
}

#[test]
fn apply_replaces_the_primary_and_registers_subagents() {
    let fs = MemFs::new()
        .with_file(
            Path::new(DIR).join("rev.md"),
            "---\nname: rev\ntools: {\"*\": false, \"read_*\": true}\n---\n",
        )
        .with_file(
            Path::new(DIR).join("chefe.md"),
            "---\nname: chefe\nmode: primary\ntools: read_file, bash\n---\n",
        );
    let agents = load(&fs, Path::new(DIR)).unwrap_or_default();
    let applied = apply(&agents, registry(), &catalog());
    assert!(applied.is_ok());
    let Ok(registry) = applied else {
        return;
    };
    assert_eq!(registry.primary().tool_names(), ["bash", "read_file"]);
    assert!(!registry.primary().allows("write_file"));
    assert_eq!(registry.subagent_names(), ["explorer", "rev"]);
    let rev = registry.get_by_name("rev");
    assert_eq!(
        rev.map(AgentSpec::tool_names),
        Some(vec!["read_file".to_owned()])
    );
    assert_eq!(rev.map(AgentSpec::max_depth), Some(0));
    assert_eq!(rev.map(|spec| spec.id().as_str()), Some("rev"));
}

#[test]
fn apply_rejects_unknown_tools_two_primaries_and_empty_agents() {
    let unknown = agent(
        "x.md",
        "---\nname: x\ntools: {\"*\": false, \"read_fil\": true}\n---\n",
    );
    let empty = agent("y.md", "---\nname: y\ntools: {\"*\": false}\n---\n");
    let one = agent(
        "a.md",
        "---\nname: a\nmode: primary\ntools: read_file\n---\n",
    );
    let two = agent(
        "b.md",
        "---\nname: b\nmode: primary\ntools: read_file\n---\n",
    );
    let (Some(unknown), Some(empty), Some(one), Some(two)) = (unknown, empty, one, two) else {
        return;
    };
    assert!(
        apply(&[unknown], registry(), &catalog())
            .err()
            .is_some_and(|error| error.contains("read_fil"))
    );
    assert!(
        apply(&[empty], registry(), &catalog())
            .err()
            .is_some_and(|error| error.contains("sem tools"))
    );
    assert!(
        apply(&[one, two], registry(), &catalog())
            .err()
            .is_some_and(|error| error.contains("primary"))
    );
}
