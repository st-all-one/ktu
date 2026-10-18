---
id: fact_0075kyt8
type: fact
statement: "H3 concluida: ktu run efemero por omissao (SessionTarget) + lock de turno por sessao (stale 30s, recusa)"
created_at: 2026-10-17T23:58:05.004Z
body_hash: e2c6441f
schema_version: 2
anchors: [crates/ktu-cli/src/lock.rs, crates/ktu-cli/src/session_mode.rs, crates/ktu-cli/src/run.rs]
---
H3 (task_017rrpwn) concluida. Efemero por omissao: novo enum SessionTarget { Ephemeral, Persistent(String) } em session_mode.rs; RunConfig.session passou de String para SessionTarget. ktu run sem --session e efemero (nao carrega nem grava log/checkpoint; ensure_layout deixou de criar .ktu/sessions); --session <id> persiste; --resume/--compact implicam default (resolve_session). session_store guarda load/save/checkpoint em config.session.id(). turn_runtime usa config.session.label() (plano/subagentes); subagentes e hub/actor criam .ktu/sessions on demand. Lock de turno (lock.rs): .ktu/sessions/<id>.lock com OpenOptions create_new (atomico), stale 30s (STALE_MS) lido via porta Clock, recusa se vivo (nunca apaga lock alheio); guarda RAII (Drop remove); so em sessoes persistentes. Testes: lock.rs (3 unit: recusa, stale reclamado, ids distintos) + tests/lock.rs (3 integracao via binario: lock vivo recusa, lock largado passa e grava, efemero nao persiste). Comportamento documentado no PROTOCOL (SESSAO:) e no --session help. Gate: make check + typos + machete verdes; ktu-cli 39+3+2+1 testes.