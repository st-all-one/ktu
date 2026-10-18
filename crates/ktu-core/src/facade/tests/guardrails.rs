//! Testes dos guardrails declarativos na fachada (I16 e o piso aditivo M5).

use std::path::Path;

use super::{Fixture, ROOT, call, text};
use crate::domain::Completion;
use crate::session::{Event, SessionLog};
use crate::testing::FakeProvider;

#[tokio::test]
async fn a_global_guardrail_cannot_be_disabled_by_the_project() {
    // M5: guardrail global é **piso aditivo**. O projecto pode acrescentar regras,
    // nunca desligar as globais — a cadeia só consulta a regra seguinte se a
    // anterior deixar passar.
    let deny = "---\nid: deny-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: deny\nreason: segredos\n---\n";
    let allow = "---\nid: allow-env\nversion: 1\nfact: path\nmatch: '*.env'\ndecision: allow\nreason: quero\n---\n";
    let fixture = Fixture::new(vec![text("x")]);
    let fixture = Fixture {
        fs: fixture
            .fs
            .with_file(Path::new("/utilizador/guardrails/deny-env.md"), deny)
            .with_dir(Path::new(ROOT).join(".agents").join("guardrails"))
            .with_file(
                Path::new(ROOT)
                    .join(".agents")
                    .join("guardrails")
                    .join("allow-env.md"),
                allow,
            ),
        ..fixture
    };
    let mut log = SessionLog::new();
    let script = vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![call("c1", "read_file", "{\"path\":\"a.env\"}")],
        },
        text("fim"),
    ];
    let fixture = Fixture {
        provider: FakeProvider::new(script),
        ..fixture
    };
    let Ok(ktu) = fixture.build_with_user(Path::new("/utilizador")) else {
        return;
    };
    let _ignored = ktu.turn(&mut log, "le a.env").await;
    let denied = log
        .events()
        .iter()
        .any(|event| matches!(event, Event::GuardDecision { allowed, .. } if !allowed));
    assert!(denied, "o guardrail global nao foi aplicado");
}

#[test]
fn broken_permissions_stop_the_build() {
    let fixture = Fixture::new(vec![text("x")]);
    let fixture = Fixture {
        fs: fixture.fs.with_file(
            Path::new(ROOT).join(".agents").join("permissions.toml"),
            "[permissions]\nbash = \"talvez\"\n",
        ),
        ..fixture
    };
    let error = fixture.build();
    let message = match error {
        Ok(_) => String::new(),
        Err(error) => error.to_string(),
    };
    assert!(message.contains("permissions.toml"), "{message}");
}
