//! I15: no modo plano, a escrita é negada estruturalmente e a tool `plan` persiste
//! o artefacto em `.ktu/plan/<sessão>.json`.

use std::path::PathBuf;

use ktu_core::domain::{ToolCall, ToolCallId};
use ktu_core::plan::{self, FsPlanHost};
use ktu_core::ports::{FixedClock, Fs, MemFs, Timestamp};
use ktu_core::tools::{ScopedTools, ToolExecutor};
use ktu_tools::{Toolbox, plan_mode_spec};

fn call(name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: ToolCallId::new("1"),
        name: name.to_owned(),
        arguments: arguments.to_owned(),
    }
}

#[tokio::test]
async fn plan_mode_denies_writes_and_persists_the_plan() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let root = PathBuf::from("/r");
    let plan_dir = root.join(".ktu").join("plan");
    let host = FsPlanHost::new(&fs, &plan_dir, "s1");
    let toolbox = Toolbox::new(&root, &fs, &clock, None).with_plan(host);
    let spec = plan_mode_spec();
    let scoped = ScopedTools::new(&toolbox, spec.tool_names());

    // O modelo só vê leitura + `plan` (nunca `write_file`).
    let names: Vec<String> = scoped.specs().into_iter().map(|spec| spec.name).collect();
    assert!(names.contains(&"plan".to_owned()));
    assert!(!names.contains(&"write_file".to_owned()));

    // Escrita recusada por capacidade (I9/I15) — nunca chega a executar.
    let denied = scoped
        .execute(&call("write_file", r#"{"path":"a","content":"x"}"#))
        .await;
    assert!(denied.is_error);

    // A tool `plan` funciona e grava o artefacto de forma durável.
    let added = scoped
        .execute(&call(
            "plan",
            r#"{"action":"stage_add","id":"F1","title":"Fundacao"}"#,
        ))
        .await;
    assert!(!added.is_error, "{}", added.content);
    assert!(fs.exists(&plan_dir.join("s1.json")));

    let loaded = plan::load(&fs, &plan_dir, "s1");
    assert_eq!(
        loaded.unwrap_or(None).map(|plan| plan.stages.len()),
        Some(1)
    );
}

#[tokio::test]
async fn deferred_tools_leave_the_catalog_but_stay_executable() {
    let fs = MemFs::new();
    let clock = FixedClock::new(Timestamp::from_millis(0));
    let root = PathBuf::from("/r");
    let toolbox = Toolbox::new(&root, &fs, &clock, None)
        .with_deferred(["glob".to_owned(), "grep".to_owned()]);

    let declared: Vec<String> = toolbox.specs().into_iter().map(|spec| spec.name).collect();
    let deferred: Vec<String> = toolbox
        .deferred()
        .into_iter()
        .map(|spec| spec.name)
        .collect();
    assert!(!declared.contains(&"glob".to_owned()), "{declared:?}");
    assert!(!declared.contains(&"grep".to_owned()), "{declared:?}");
    assert!(declared.contains(&"read_file".to_owned()), "{declared:?}");
    assert_eq!(deferred, ["glob", "grep"]);

    // Diferida é só exposição: a execução continua a funcionar.
    let outcome = toolbox
        .execute(&call("glob", r#"{"pattern":"*.md"}"#))
        .await;
    assert!(!outcome.is_error, "{}", outcome.content);

    // O `ScopedTools` cruza a exposição com o **conjunto do agente**: uma diferida
    // fora do conjunto nunca é oferecida (nem por `tool_search`).
    let scoped = ScopedTools::new(&toolbox, ["read_file".to_owned(), "glob".to_owned()]);
    let names: Vec<String> = scoped
        .deferred()
        .into_iter()
        .map(|spec| spec.name)
        .collect();
    assert_eq!(names, ["glob"]);
    let scoped = ScopedTools::new(&toolbox, ["read_file".to_owned()]);
    assert!(
        scoped.deferred().is_empty(),
        "glob nao esta no conjunto do agente"
    );
}
