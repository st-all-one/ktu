//! Testes do modo plano: transição (I15), acções e artefacto durável.

use std::path::Path;

use crate::plan::{
    self, Approval, FsPlanHost, Mode, Plan, PlanAction, PlanState, apply, load, parse_action, save,
};
use crate::ports::{Fs, MemFs};

#[test]
fn plan_to_build_requires_approval() {
    let approved = Approval::Approved;
    let pending = Approval::Pending;
    assert_eq!(
        Mode::transition(Mode::Plan, Mode::Plan, pending),
        Ok(Mode::Plan)
    );
    assert_eq!(
        Mode::transition(Mode::Build, Mode::Plan, pending),
        Ok(Mode::Plan)
    );
    assert_eq!(
        Mode::transition(Mode::Plan, Mode::Build, approved),
        Ok(Mode::Build)
    );
    let refused = Mode::transition(Mode::Plan, Mode::Build, pending);
    assert!(refused.is_err());
    assert_eq!(refused.unwrap_or(Mode::Build), Mode::Build);
}

#[test]
fn parses_and_applies_a_plan() {
    let mut plan = Plan::new("s1");
    {
        let mut run = |json: &str| {
            let parsed = parse_action(json);
            assert!(parsed.is_ok(), "acao invalida: {json}");
            if let Ok(action) = parsed {
                let _ignored = apply(&mut plan, action);
            }
        };
        run(r#"{"action":"stage_add","id":"F1","title":"Fundacao"}"#);
        run(
            r#"{"action":"task_add","stage":"F1","id":"T1","objective":"loop","depends_on":["T0"],"done_when":"verde"}"#,
        );
        run(r#"{"action":"task_state","id":"T1","state":"doing"}"#);
    }
    let rendered = plan::render(&plan);
    assert!(rendered.contains("T1 (doing) loop"));
    assert_eq!(plan.stages.len(), 1);
    let task_count = plan.stages.first().map(|stage| stage.tasks.len());
    assert_eq!(task_count, Some(1));
    let first = plan.stages.first().and_then(|stage| stage.tasks.first());
    assert_eq!(first.map(|task| task.state), Some(PlanState::Doing));
}

#[test]
fn rejects_unknown_stage_and_duplicate_ids() {
    let mut plan = Plan::new("s1");
    let missing = parse_action(r#"{"action":"task_add","stage":"X","id":"T","objective":"o"}"#);
    assert!(missing.is_ok());
    if let Ok(action) = missing {
        assert!(apply(&mut plan, action).is_err());
    }
    let stage = parse_action(r#"{"action":"stage_add","id":"F1","title":""}"#);
    assert!(stage.is_ok());
    if let Ok(action) = stage {
        assert!(apply(&mut plan, action).is_ok());
    }
    let duplicate = parse_action(r#"{"action":"stage_add","id":"F1","title":""}"#);
    assert!(duplicate.is_ok());
    if let Ok(action) = duplicate {
        assert!(apply(&mut plan, action).is_err());
    }
    assert!(parse_action("nao json").is_err());
    assert!(parse_action(r#"{"action":"desconhecida"}"#).is_err());
}

#[test]
fn rejects_unknown_task_state() {
    let action = parse_action(r#"{"action":"task_state","id":"T1","state":"nope"}"#);
    assert!(action.is_err());
}

#[test]
fn artifact_round_trips_atomically() {
    let fs = MemFs::new();
    let dir = Path::new("/r/.ktu/plan");
    let mut plan = Plan::new("s1");
    let mut action = |json: &str| {
        if let Ok(parsed) = parse_action(json) {
            let _ignored = apply(&mut plan, parsed);
        }
    };
    action(r#"{"action":"stage_add","id":"F1","title":"Fundacao"}"#);
    action(
        r#"{"action":"task_add","stage":"F1","id":"T1","objective":"loop","done_when":"verde"}"#,
    );

    assert!(save(&fs, dir, &plan).is_ok());
    let loaded = load(&fs, dir, "s1");
    assert!(loaded.is_ok());
    assert_eq!(loaded.unwrap_or(None), Some(plan));
    // O temporário foi renomeado (não fica lixo).
    assert!(!fs.exists(&dir.join("s1.json.tmp")));
    assert!(fs.exists(&dir.join("s1.json")));
}

#[test]
fn fs_plan_host_applies_and_persists() {
    let fs = MemFs::new();
    let dir = Path::new("/r/.ktu/plan");
    let host = FsPlanHost::new(&fs, dir, "s1");

    let stage = host.execute(r#"{"action":"stage_add","id":"F1","title":"Fase"}"#);
    assert!(!stage.is_error);
    let task = host.execute(
        r#"{"action":"task_add","stage":"F1","id":"T1","objective":"fazer","done_when":"ok"}"#,
    );
    assert!(!task.is_error, "{}", task.content);
    let bad = host.execute(r#"{"action":"task_state","id":"nope","state":"done"}"#);
    assert!(bad.is_error);

    let loaded = load(&fs, dir, "s1").unwrap_or(None);
    assert_eq!(loaded.map(|plan| plan.stages.len()), Some(1));
}

#[test]
fn plan_action_is_debug_and_eq() {
    let action = PlanAction::List;
    assert!(format!("{action:?}").contains("List"));
    assert_eq!(action, PlanAction::List);
    assert_eq!(Mode::parse("plan"), Some(Mode::Plan));
    assert_eq!(Mode::parse("build"), Some(Mode::Build));
    assert_eq!(Mode::parse("x"), None);
}
