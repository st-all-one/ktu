---
id: task_00ek2v0f
type: task
statement: Definir portas com injecção de dependência (Clock/Rng/Fs/Env) com fakes; domínio puro e sem unsafe/unwrap/expect/panic
created_at: 2026-10-13T00:50:48.046Z
body_hash: aa70f642
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:09:37.417Z
    notes: portas Clock/Rng/Fs/Env com fakes deterministas (FixedClock/SeqClock/SeqRng/MemFs/MapEnv); Error como valor (thiserror); make check verde (15 testes no core)
anchors: [crates/ktu-core/src/ports/mod.rs, crates/ktu-core/src/ports/fakes/mod.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00qm4eu6 -->