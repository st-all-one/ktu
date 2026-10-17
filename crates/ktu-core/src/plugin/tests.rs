//! Testes do catálogo de *hooks* e do *dispatcher* de plugins (§2.9).

use std::sync::Arc;

use super::{Dispatcher, HookEvent, HookPayload, Hooks, Mode, NoHooks, Plugin, PluginVerdict};

struct FakePlugin {
    name: &'static str,
    mode: Mode,
    events: Vec<HookEvent>,
    verdict: PluginVerdict,
}

impl FakePlugin {
    fn new(name: &'static str, mode: Mode, event: HookEvent, verdict: PluginVerdict) -> Self {
        Self {
            name,
            mode,
            events: vec![event],
            verdict,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for FakePlugin {
    fn name(&self) -> &str {
        self.name
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn subscribes(&self, event: HookEvent) -> bool {
        self.events.contains(&event)
    }

    async fn handle(&self, _payload: &HookPayload) -> PluginVerdict {
        self.verdict.clone()
    }
}

fn user_input(text: &str) -> HookPayload {
    HookPayload::UserInput {
        text: text.to_owned(),
    }
}

#[tokio::test]
async fn empty_hooks_allow_everything() {
    let mut payload = user_input("oi");
    let verdict = NoHooks.dispatch(&mut payload).await;
    assert!(verdict.allowed);
    assert_eq!(verdict.generation, 0);
    assert_eq!(payload, user_input("oi"));
}

#[tokio::test]
async fn transforms_apply_in_name_order() {
    let first = Arc::new(FakePlugin::new(
        "a-first",
        Mode::Transform,
        HookEvent::UserInput,
        PluginVerdict::Replace {
            text: "A".to_owned(),
        },
    ));
    let second = Arc::new(FakePlugin::new(
        "b-second",
        Mode::Transform,
        HookEvent::UserInput,
        PluginVerdict::Replace {
            text: "B".to_owned(),
        },
    ));
    // Passados fora de ordem: o *dispatcher* ordena por nome (determinismo).
    let dispatcher = Dispatcher::new(vec![second, first], 3);
    assert_eq!(dispatcher.names(), vec!["a-first", "b-second"]);
    let mut payload = user_input("base");
    let verdict = dispatcher.dispatch(&mut payload).await;
    assert!(verdict.allowed);
    assert_eq!(verdict.transforms, 2);
    assert_eq!(payload, user_input("B"));
}

#[tokio::test]
async fn gate_denies_and_stops_the_chain() {
    let gate = Arc::new(FakePlugin::new(
        "gate",
        Mode::Gate,
        HookEvent::ActionBefore,
        PluginVerdict::Deny {
            reason: "plugin_timeout".to_owned(),
        },
    ));
    let later = Arc::new(FakePlugin::new(
        "z-later",
        Mode::Transform,
        HookEvent::ActionBefore,
        PluginVerdict::Replace {
            text: "nunca".to_owned(),
        },
    ));
    let dispatcher = Dispatcher::new(vec![later, gate], 7);
    let mut payload = HookPayload::ActionBefore {
        tool: "bash".to_owned(),
        arguments: "{}".to_owned(),
    };
    let verdict = dispatcher.dispatch(&mut payload).await;
    assert!(!verdict.allowed);
    assert_eq!(verdict.reason.as_deref(), Some("plugin_timeout"));
    assert_eq!(verdict.generation, 7);
    assert_eq!(verdict.transforms, 0);
}

#[tokio::test]
async fn observe_verdict_never_transforms_nor_denies() {
    let observe = Arc::new(FakePlugin::new(
        "obs",
        Mode::Observe,
        HookEvent::UserInput,
        PluginVerdict::Deny {
            reason: "ignorado".to_owned(),
        },
    ));
    let dispatcher = Dispatcher::new(vec![observe], 1);
    let mut payload = user_input("texto");
    let verdict = dispatcher.dispatch(&mut payload).await;
    assert!(verdict.allowed);
    assert_eq!(payload, user_input("texto"));
}

#[test]
fn event_names_round_trip() {
    for event in HookEvent::ALL {
        assert_eq!(HookEvent::from_name(event.as_str()), Some(*event));
    }
    assert_eq!(HookEvent::from_name("desconhecido"), None);
}
