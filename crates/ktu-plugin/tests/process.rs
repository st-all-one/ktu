//! Integração do runtime de plugins externos (§2.9): JSON-lines, gate e geração.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use ktu_core::Origin;
use ktu_core::plugin::{HookPayload, Hooks, Plugin, PluginVerdict};
use ktu_plugin::{Manifest, PluginHost, ProcessPlugin};

const PASS_SCRIPT: &str = "#!/bin/sh\nread -r line\nprintf '{\"verdict\":\"pass\"}\\n'\n";
const GATE_SCRIPT: &str = "#!/bin/sh\nread -r line\ncase \"$line\" in *bash*) printf '{\"verdict\":\"deny\",\"reason\":\"plugin_negou\"}\\n';; *) printf '{\"verdict\":\"pass\"}\\n';; esac\n";

fn temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("ktu-plugin-{}-{name}", std::process::id()));
    dir
}

/// Escreve `<root>/<name>/{plugin.toml,run.sh}` e devolve o directório.
fn write_plugin(
    root: &Path,
    name: &str,
    mode: &str,
    timeout: u64,
    script: &str,
) -> Result<PathBuf, String> {
    let dir = root.join(name);
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let manifest = format!(
        "name = \"{name}\"\nentrypoint = \"./run.sh\"\nevents = [\"user.input\", \"action.before\"]\nmode = \"{mode}\"\ntimeout_ms = {timeout}\n"
    );
    fs::write(dir.join("plugin.toml"), manifest).map_err(|error| error.to_string())?;
    let script_path = dir.join("run.sh");
    fs::write(&script_path, script).map_err(|error| error.to_string())?;
    let mut perms = fs::metadata(&script_path)
        .map_err(|error| error.to_string())?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn input(text: &str) -> HookPayload {
    HookPayload::UserInput {
        text: text.to_owned(),
    }
}

fn action(tool: &str) -> HookPayload {
    HookPayload::ActionBefore {
        tool: tool.to_owned(),
        arguments: "{}".to_owned(),
    }
}

#[tokio::test]
async fn a_script_plugin_speaks_json_lines() -> Result<(), String> {
    let root = temp_dir("basic");
    let _ignored = fs::remove_dir_all(&root);
    let dir = write_plugin(&root, "pass", "observe", 2000, PASS_SCRIPT)?;
    let plugin = ProcessPlugin::new(&Manifest::load(&dir)?, dir.clone())?;
    assert_eq!(plugin.handle(&input("oi")).await, PluginVerdict::Pass);
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test]
async fn a_gate_denies_when_the_script_says_so() -> Result<(), String> {
    let root = temp_dir("gate");
    let _ignored = fs::remove_dir_all(&root);
    let dir = write_plugin(&root, "gate", "gate", 2000, GATE_SCRIPT)?;
    let plugin = ProcessPlugin::new(&Manifest::load(&dir)?, dir.clone())?;
    assert_eq!(
        plugin.handle(&action("bash")).await,
        PluginVerdict::Deny {
            reason: "plugin_negou".to_owned(),
        }
    );
    assert_eq!(
        plugin.handle(&action("read_file")).await,
        PluginVerdict::Pass
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test]
async fn a_gate_timeout_is_fail_closed() -> Result<(), String> {
    let root = temp_dir("timeout");
    let _ignored = fs::remove_dir_all(&root);
    let slow = "#!/bin/sh\nsleep 5\nprintf '{\"verdict\":\"pass\"}\\n'\n";
    let dir = write_plugin(&root, "slow", "gate", 150, slow)?;
    let plugin = ProcessPlugin::new(&Manifest::load(&dir)?, dir.clone())?;
    assert_eq!(
        plugin.handle(&action("bash")).await,
        PluginVerdict::Deny {
            reason: "plugin_timeout".to_owned(),
        }
    );
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test]
async fn reload_increments_the_generation_without_dropping_the_session() -> Result<(), String> {
    let root = temp_dir("reload");
    let _ignored = fs::remove_dir_all(&root);
    let _dir = write_plugin(&root, "pass", "observe", 2000, PASS_SCRIPT)?;
    let host = PluginHost::load(root.clone());
    assert_eq!(host.generation(), 1);
    assert_eq!(host.names(), vec!["pass".to_owned()]);
    let mut payload = input("oi");
    assert!(host.dispatch(&mut payload).await.allowed);
    let errors = host.reload();
    assert!(errors.is_empty(), "esperava zero erros");
    assert_eq!(host.generation(), 2);
    let mut payload = input("oi outra vez");
    assert!(host.dispatch(&mut payload).await.allowed);
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test]
async fn a_broken_manifest_disables_the_plugin_but_not_the_session() -> Result<(), String> {
    let root = temp_dir("broken");
    let _ignored = fs::remove_dir_all(&root);
    let dir = root.join("bad");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    fs::write(dir.join("plugin.toml"), "isto nao e toml =").map_err(|error| error.to_string())?;
    let host = PluginHost::load(root.clone());
    assert!(host.names().is_empty(), "esperava zero plugins");
    assert_eq!(host.errors().len(), 1);
    let mut payload = input("oi");
    assert!(host.dispatch(&mut payload).await.allowed);
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}

#[tokio::test]
async fn the_repo_examples_transform_and_gate() -> Result<(), String> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins");
    let transform_dir = base.join("input-note");
    let transform = ProcessPlugin::new(&Manifest::load(&transform_dir)?, transform_dir)?;
    match transform.handle(&input("ola")).await {
        PluginVerdict::Replace { text } => assert_eq!(text, "[exemplo] ola"),
        other => return Err(format!("esperava replace, veio {other:?}")),
    }
    let gate_dir = base.join("guard-bash");
    let gate = ProcessPlugin::new(&Manifest::load(&gate_dir)?, gate_dir)?;
    let destructive = HookPayload::ActionBefore {
        tool: "bash".to_owned(),
        arguments: r#"{"command":"rm -rf /"}"#.to_owned(),
    };
    assert_eq!(
        gate.handle(&destructive).await,
        PluginVerdict::Deny {
            reason: "bash_destrutivo".to_owned(),
        }
    );
    let safe = HookPayload::ActionBefore {
        tool: "bash".to_owned(),
        arguments: r#"{"command":"ls"}"#.to_owned(),
    };
    assert_eq!(gate.handle(&safe).await, PluginVerdict::Pass);
    Ok(())
}

/// Os dois níveis são **aditivos** (M8): o mesmo nome nos dois corre duas vezes.
#[tokio::test]
async fn global_and_project_plugins_are_additive() -> Result<(), String> {
    let root = temp_dir("layered");
    let _ignored = fs::remove_dir_all(&root);
    let global = root.join("global");
    let project = root.join("project");
    let _g1 = write_plugin(&global, "g1", "observe", 2000, PASS_SCRIPT)?;
    let _g2 = write_plugin(&global, "g2", "observe", 2000, PASS_SCRIPT)?;
    let _l1 = write_plugin(&project, "g1", "observe", 2000, PASS_SCRIPT)?;

    let host = PluginHost::layered(vec![(Origin::Global, global), (Origin::Local, project)]);
    assert!(host.errors().is_empty(), "{:?}", host.errors());
    // Global primeiro, por ordem de nome; o projecto depois (não desliga o global).
    assert_eq!(
        host.entries(),
        vec![
            ("g1".to_owned(), Origin::Global),
            ("g2".to_owned(), Origin::Global),
            ("g1".to_owned(), Origin::Local),
        ]
    );
    // Os três despacham (aditivo, não substituição).
    let mut payload = input("oi");
    assert!(host.dispatch(&mut payload).await.allowed);
    assert_eq!(host.names(), vec!["g1", "g2", "g1"]);
    let _ignored = fs::remove_dir_all(&root);
    Ok(())
}
