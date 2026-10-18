---
id: fact_00s96tou
type: fact
statement: "H2 parcial: config dois niveis (schema fechado, local>global) + ktu config get/set/list/unset + auth redigida (M3)"
created_at: 2026-10-17T23:49:39.472Z
body_hash: 31260b30
schema_version: 2
anchors: [crates/ktu-core/src/config.rs, crates/ktu-cli/src/config_cmd.rs]
---
H2 (task_00tx0uvu) PARCIAL. Feito: ktu-core::config (puro) — Config (chaves fechadas: provider/base_url/model/effort/max_steps; deny_unknown_fields; KEYS estaveis), Auth (BTreeMap providers; Debug REDIGIDO, M3), Config::merge(local, global) (local ganha chave a chave, M6), parse/render TOML, set/unset/get/entries, caminhos global_dir (XDG_CONFIG_HOME/ktu ou HOME/.config/ktu), global_config_path/global_auth_path, local_config_path. ktu-cli: config_cmd.rs (ktu config list/get/set/unset com --local/--global; envelope --json; o auth nunca e listado) + Settings/load (config efectiva + auth) + integracao no dispatch_run e dispatch_serve (flag > --params > local > global > default; credenciais flag > env > auth.toml). Serve: provider/effort/max_steps passaram a Option para a config poder ganhar. toml adicionado ao ktu-core. Verificado: config set --local/--global; list efectivo mostra source; get/unset; set sem ambito -> erro; run usa provider=mock da config e falha com provider=nope; --provider/--params ganham. Gate: make check + typos + machete verdes. ADIADO por YAGNI: recursos globais (skills/guardrails/plugins — o runtime ainda nao os carrega) e migracoes (schema v1, sem versao anterior).