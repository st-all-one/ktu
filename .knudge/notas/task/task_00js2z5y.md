---
id: task_00js2z5y
type: task
statement: Toolchain Rust 1.99: rust-toolchain.toml + MSRV 1.99 + rustfmt style_edition
created_at: 2026-10-17T00:42:24.788Z
body_hash: 03f6f934
schema_version: 2
tags: [toolchain]
replaces: [fact_00dh57mm]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-17T01:06:55.246Z
    notes: "rust-version/msrv=1.99; rust-toolchain.toml (1.99, clippy/rustfmt); rustfmt style_edition=2024 + hex_literal_case; rustdoc unused_footnote_definition. make check verde."
anchors: [rust-toolchain.toml]
status: closed
scope: task
---
Subir rust-version para 1.99 e msrv para 1.99; adicionar rust-toolchain.toml (channel 1.99, components clippy/rustfmt, profile minimal); rustfmt.toml com style_edition=2024 e hex_literal_case=Lower, alinhado com o REF/sniff-css e a skill .agents/skills/rust.
<!-- knudge:parent task_015epqsj -->