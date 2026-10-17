---
id: fact_00ml2kgy
type: fact
statement: SessionHub: um actor por sessao (dono do SessionLog); cancel out-of-band; log persistido
created_at: 2026-10-17T02:06:37.933Z
body_hash: dab022c6
schema_version: 2
anchors: [crates/ktu-cli/src/hub.rs]
---
crates/ktu-cli/src/hub.rs + hub/actor.rs: um actor por sessao possui o SessionLog; o turno corre em execute_turn e publica os eventos novos (ServerMsg::Log) + TurnFinished. O cancel e fora-de-banda (CancelToken partilhado, nao espera o actor). SetMode e logado e persistido; resume no arranque.