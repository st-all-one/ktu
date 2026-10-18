//! Testes do modulo (extraidos para manter o ficheiro de producao abaixo do teto).

use super::{Action, DrainMode, worker_args};
use crate::actions::ServiceArgs;

fn service_flags() -> ServiceArgs {
    ServiceArgs {
        install: false,
        subscribe: false,
        unsubscribe: false,
        status: false,
        uninstall: false,
        reconcile: false,
        keep_model: false,
        remove_model: false,
        dry_run: false,
        every: "1h".to_owned(),
        port: 8889,
        model: None,
        no_deps: false,
    }
}

#[test]
fn default_action_is_status_and_is_not_scoped() {
    let flags = service_flags();
    let action = Action::of(&flags);
    assert_eq!(action, Action::Status);
    assert_eq!(
        worker_args(action, std::path::Path::new("/p"), &flags),
        vec!["status"]
    );
    assert_eq!(DrainMode::Status, DrainMode::Status);
    assert_ne!(DrainMode::Status, DrainMode::Force);
}

#[test]
fn install_passes_every_port_and_project() {
    let mut flags = service_flags();
    flags.install = true;
    let argv = worker_args(Action::of(&flags), std::path::Path::new("/proj"), &flags);
    assert_eq!(
        argv,
        vec![
            "install",
            "--project",
            "/proj",
            "--every",
            "1h",
            "--port",
            "8889"
        ]
    );
}

#[test]
fn uninstall_carries_the_model_policy_and_dry_run() {
    let mut flags = service_flags();
    flags.uninstall = true;
    flags.remove_model = true;
    flags.dry_run = true;
    let argv = worker_args(Action::of(&flags), std::path::Path::new("/proj"), &flags);
    assert_eq!(
        argv,
        vec!["uninstall", "--remove-model", "--dry-run"],
        "a desinstalacao e global (nao e scoped)"
    );
}
