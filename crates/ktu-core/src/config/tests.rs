//! Testes do schema de config e das migrações (F5).

use std::path::PathBuf;

use toml::Value;

use super::migrate::{Obsolete, Step, apply_steps, migrate, read_migrate, schema_of, table};
use super::{
    Auth, Config, SCHEMA, global_auth_path, global_config_path, global_dir, local_config_path,
};

/// Documento v0 com a forma plana de `deferred_tools` e uma chave que o schema
/// já não conhece (a "chave plana antiga" do contrato de leitura).
fn legacy_document() -> toml::Table {
    table("model = \"m\"\ndeferred_tools = \"bash, write_file\"\nmax_tokens = 2048\n")
        .unwrap_or_default()
}

#[test]
fn parse_rejects_unknown_keys() {
    assert!(Config::parse("provider = \"mock\"\n").is_ok());
    assert!(Config::parse("nope = 1\n").is_err());
}

#[test]
fn local_wins_key_by_key_over_global() {
    let global = Config {
        provider: Some("llama".to_owned()),
        model: Some("global-model".to_owned()),
        ..Config::default()
    };
    let local = Config {
        model: Some("local-model".to_owned()),
        effort: Some("high".to_owned()),
        ..Config::default()
    };
    let effective = Config::merge(local, global);
    assert_eq!(effective.provider.as_deref(), Some("llama"));
    assert_eq!(effective.model.as_deref(), Some("local-model"));
    assert_eq!(effective.effort.as_deref(), Some("high"));
}

#[test]
fn set_unset_get_roundtrip() {
    let mut config = Config::default();
    assert!(config.set("provider", "mock").is_ok());
    assert!(config.set("max_steps", "3").is_ok());
    assert_eq!(config.get("provider").as_deref(), Some("mock"));
    assert_eq!(config.get("max_steps").as_deref(), Some("3"));
    assert!(config.set("max_steps", "muitos").is_err());
    assert!(config.unset("provider").is_ok());
    assert_eq!(config.get("provider"), None);
    assert!(config.set("desconhecida", "x").is_err());
}

#[test]
fn render_omits_undefined_keys() {
    let mut config = Config::default();
    assert!(config.set("model", "m").is_ok());
    let text = config.render().unwrap_or_default();
    assert!(text.contains("model = \"m\""));
    assert!(!text.contains("provider"));
}

#[test]
fn global_dir_prefers_xdg_then_home() {
    assert_eq!(
        global_dir(Some("/xdg"), Some("/home/u")),
        Some(PathBuf::from("/xdg/ktu"))
    );
    assert_eq!(
        global_dir(None, Some("/home/u")),
        Some(PathBuf::from("/home/u/.config/ktu"))
    );
    assert_eq!(global_dir(Some(""), Some("")), None);
    assert_eq!(global_config_path(None, None), None);
    assert_eq!(
        global_auth_path(Some("/xdg"), None),
        Some(PathBuf::from("/xdg/ktu/auth.toml"))
    );
}

#[test]
fn local_path_lives_in_agents() {
    assert_eq!(
        local_config_path(&PathBuf::from("/proj")),
        PathBuf::from("/proj/.agents/config.toml")
    );
}

#[test]
fn auth_reads_provider_key() {
    let auth = Auth::parse("[providers]\nopencode-go = \"sk-x\"\n").unwrap_or_default();
    assert_eq!(auth.api_key("opencode-go"), Some("sk-x"));
    assert_eq!(auth.api_key("llama"), None);
}

#[test]
fn auth_debug_never_leaks_the_key() {
    let auth = Auth::parse("[providers]\nopencode-go = \"sk-secret\"\n").unwrap_or_default();
    let rendered = format!("{auth:?}");
    assert!(rendered.contains("opencode-go"));
    assert!(!rendered.contains("sk-secret"), "a chave nao pode aparecer");
}

#[test]
fn schema_of_reads_the_declared_version() {
    assert_eq!(schema_of(&table("model = \"m\"\n").unwrap_or_default()), 0);
    assert_eq!(
        schema_of(&table("schema = 1\n").unwrap_or_default()),
        SCHEMA
    );
    // Declarada mas inválida ⇒ v0 (não se inventa uma versão).
    assert_eq!(
        schema_of(&table("schema = \"um\"\n").unwrap_or_default()),
        0
    );
}

#[test]
fn read_migrate_stamps_the_version_and_keeps_every_key() {
    let migrated = read_migrate(legacy_document()).unwrap_or_default();
    assert_eq!(schema_of(&migrated), SCHEMA);
    // A chave plana antiga **não** é retirada (outro chamador pode consultá-la)…
    assert!(migrated.contains_key("max_tokens"), "{migrated:?}");
    // …e o valor antigo é normalizado (string CSV → lista). A tabela migrada
    // serve os dois: quem tiver uma chave desconhecida ainda a vê.
    let tools = migrated.get("deferred_tools").and_then(Value::as_array);
    assert_eq!(
        tools.map(|values| values.iter().filter_map(Value::as_str).collect::<Vec<_>>()),
        Some(vec!["bash", "write_file"])
    );
    // Sem a chave desconhecida, o documento migrado tipa (v0 → v1 legível).
    let typed = Config::from_table(read_migrate(legacy_document()).unwrap_or_default());
    assert!(typed.is_err(), "a chave desconhecida continua a ser erro");
}

#[test]
fn migrate_is_idempotent_and_drops_only_what_a_step_declares() {
    let (first, changed) = migrate(legacy_document()).unwrap_or_default();
    assert!(changed, "o v0 tem de migrar");
    assert_eq!(schema_of(&first), SCHEMA);
    let (second, again) = migrate(first.clone()).unwrap_or_default();
    assert!(!again, "migrar duas vezes nao muda nada");
    assert_eq!(first, second);
    // A lista de obsoletas do v1 está vazia: a chave desconhecida fica (é o
    // passo que decide o que sai, não a migração por si).
    assert!(second.contains_key("max_tokens"));
    // Com um passo que a declare, a **escrita** retira-a e a **leitura** não.
    let step = Step {
        to: SCHEMA,
        deprecated: &["max_tokens"],
    };
    let mut dropped = table("max_tokens = 1\n").unwrap_or_default();
    let _changed = apply_steps(&mut dropped, Obsolete::Drop, &[step]);
    assert!(!dropped.contains_key("max_tokens"));
    let mut kept = table("max_tokens = 1\n").unwrap_or_default();
    let _changed = apply_steps(&mut kept, Obsolete::Keep, &[step]);
    assert!(kept.contains_key("max_tokens"));
}

#[test]
fn a_future_version_is_refused_without_touching_anything() {
    let document = table("schema = 3\nmodel = \"m\"\n").unwrap_or_default();
    let read = read_migrate(document.clone());
    let write = migrate(document);
    for error in [read.err(), write.err()] {
        let message = error.unwrap_or_default();
        assert!(message.contains("schema v3"), "{message}");
        assert!(
            message.contains("nenhum ficheiro foi alterado"),
            "{message}"
        );
    }
}
