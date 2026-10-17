---
id: fact_00wvds59
type: fact
statement: turn_runtime::execute_turn partilhado por ktu run e serve (uma so composicao de DI)
created_at: 2026-10-17T02:06:38.488Z
body_hash: db41c8a1
schema_version: 2
anchors: [crates/ktu-cli/src/turn_runtime.rs]
---
crates/ktu-cli/src/turn_runtime.rs: extrai a composicao de portas (provider, thread T2, memoria, guard, subagentes) de run.rs. O embedded e o servidor usam o mesmo caminho; o cancel e injectado pela borda. Evita DI duplicada.