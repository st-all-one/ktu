---
id: task_01i0wipl
type: task
statement: Criar workspace Rust 2024 com crates ktu-core/tools/provider/port/cli/memory e firewall de camadas (xtask check-layers)
created_at: 2026-10-13T00:50:47.943Z
body_hash: d5ad214a
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:01:44.922Z
    notes: "workspace Rust 2024 (resolver 3, rust-version 1.97) com crates core/tools/provider/port/memory/cli + xtask; make check verde; xtask check-layers apanha violacao nao-ciclica (ktu-port→ktu-memory) e reverte limpo"
anchors: [Cargo.toml, xtask/src/main.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00qm4eu6 -->