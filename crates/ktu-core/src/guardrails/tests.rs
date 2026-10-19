//! Testes dos guardrails declarativos (§2.8, I16).

use std::path::Path;

use super::{ChainGuard, DeclarativeGuard, FactKind, glob_match, load, load_layered, parse};
use crate::guard::{AgentId, Decision, DenyReason, Fact, Guard, StructuralGuard};
use crate::origin::Origin;
use crate::ports::MemFs;

const VALID: &str = "---\nid: deny-dotenv\nversion: 1\napplies_to: explorer\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n\n# razao humana\n";

fn fact(agent: AgentId, tool: &str, path: Option<&str>) -> Fact {
    Fact {
        agent,
        tool: tool.to_owned(),
        requested_path: path.map(str::to_owned),
        remaining_steps: 5,
        depth: 0,
    }
}

#[test]
fn parses_a_valid_deny_rule() -> Result<(), String> {
    let Some(rule) = parse("a.md", VALID)? else {
        return Err("esperava uma regra".to_owned());
    };
    assert_eq!(rule.id, "deny-dotenv");
    assert_eq!(rule.fact, FactKind::Path);
    assert_eq!(rule.applies_to, Some(AgentId::Explorer));
    assert_eq!(rule.matcher, "*.env");
    Ok(())
}

#[test]
fn rejects_a_broken_file_naming_it() -> Result<(), String> {
    let broken = "---\nid: x\nversion: 1\n---\n";
    let Err(message) = parse("quebrado.md", broken) else {
        return Err("devia falhar".to_owned());
    };
    assert!(message.contains("quebrado.md"));
    Ok(())
}

#[test]
fn rejects_an_unsupported_schema_version() -> Result<(), String> {
    let text = "---\nid: x\nversion: 9\nfact: tool\nmatch: bash\ndecision: deny\nreason: r\n---\n";
    let Err(message) = parse("v9.md", text) else {
        return Err("devia falhar".to_owned());
    };
    assert!(message.contains("versao"));
    Ok(())
}

#[test]
fn a_rule_that_tries_to_allow_is_ignored() -> Result<(), String> {
    let text = "---\nid: allow-write\nversion: 1\napplies_to: explorer\nfact: tool\nmatch: write_file\ndecision: allow\nreason: oops\n---\n";
    assert_eq!(parse("allow.md", text)?, None);
    Ok(())
}

#[test]
fn load_reads_md_files_and_ignores_others() -> Result<(), String> {
    let fs = MemFs::new()
        .with_file("/g/a.md", VALID)
        .with_file("/g/notas.txt", "ignorado");
    let rules = load(&fs, Path::new("/g"))?;
    assert_eq!(rules.len(), 1);
    assert_eq!(
        rules.first().map(|rule| rule.id.as_str()),
        Some("deny-dotenv")
    );
    Ok(())
}

#[test]
fn load_fails_closed_naming_the_broken_file() -> Result<(), String> {
    let fs = MemFs::new().with_file("/g/quebrado.md", "---\nid: x\n---\n");
    let Err(message) = load(&fs, Path::new("/g")) else {
        return Err("devia falhar".to_owned());
    };
    assert!(message.contains("quebrado.md"));
    Ok(())
}

#[test]
fn load_of_a_missing_directory_is_empty() -> Result<(), String> {
    let fs = MemFs::new();
    let rules = load(&fs, Path::new("/nao-existe"))?;
    assert!(rules.is_empty(), "esperava zero regras");
    Ok(())
}

#[test]
fn declarative_guard_denies_matching_facts() -> Result<(), String> {
    let Some(rule) = parse("a.md", VALID)? else {
        return Err("esperava uma regra".to_owned());
    };
    let guard = DeclarativeGuard::new(vec![rule]);
    let explorer = fact(AgentId::Explorer, "read_file", Some(".env"));
    assert_eq!(
        guard.evaluate(&explorer),
        Decision::Deny(DenyReason::Guardrail)
    );
    // `applies_to: explorer` — o agente primário não é afectado.
    let primary = fact(AgentId::Primary, "read_file", Some(".env"));
    assert_eq!(guard.evaluate(&primary), Decision::Allow);
    Ok(())
}

#[test]
fn chain_guard_does_not_free_a_structural_deny() {
    let structural = StructuralGuard::new("/r", ["read_file".to_owned()], 1);
    let permissive = DeclarativeGuard::new(Vec::new());
    let chain = ChainGuard::new(&structural, &permissive);
    let write = fact(AgentId::Primary, "write_file", None);
    assert_eq!(
        chain.evaluate(&write),
        Decision::Deny(DenyReason::Capability)
    );
}

#[test]
fn glob_matches_star() {
    assert!(glob_match("*.env", ".env"));
    assert!(glob_match("*.env", "a/b/.env"));
    assert!(glob_match("a/*/c", "a/b/c"));
    assert!(glob_match("x", "x"));
    assert!(!glob_match("*.env", "env"));
    assert!(!glob_match("a/*/c", "a/c"));
}

#[test]
fn layered_load_keeps_the_global_floor_and_names_the_origin() -> Result<(), String> {
    let global = "---\nid: deny-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n";
    let local = "---\nid: deny-bash\nversion: 1\nfact: tool\nmatch: bash\ndecision: deny\nreason: sem shell\n---\n";
    let fs = MemFs::new()
        .with_file("/u/guardrails/deny-env.md", global)
        .with_file("/p/.agents/guardrails/deny-bash.md", local);
    let rules = load_layered(&fs, Some(Path::new("/u")), Path::new("/p"))?;
    // Global primeiro: é o piso que o projecto não pode desligar (M5).
    assert_eq!(
        rules
            .iter()
            .map(|(origin, rule)| (*origin, rule.id.as_str()))
            .collect::<Vec<_>>(),
        vec![(Origin::Global, "deny-env"), (Origin::Local, "deny-bash")]
    );
    Ok(())
}

#[test]
fn layered_load_without_a_user_dir_reads_only_the_project() -> Result<(), String> {
    let local = "---\nid: deny-bash\nversion: 1\nfact: tool\nmatch: bash\ndecision: deny\nreason: sem shell\n---\n";
    let fs = MemFs::new().with_file("/p/.agents/guardrails/deny-bash.md", local);
    let rules = load_layered(&fs, None, Path::new("/p"))?;
    assert_eq!(rules.len(), 1);
    assert_eq!(
        rules.first().map(|(origin, _rule)| *origin),
        Some(Origin::Local)
    );
    Ok(())
}
