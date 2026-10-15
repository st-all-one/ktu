---
id: task_01yiyq91
type: task
statement: Fixar lints do repo (clippy.toml/rustfmt.toml) e Makefile check = fmt + clippy -D warnings + test
created_at: 2026-10-13T00:50:47.992Z
body_hash: bde340fb
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:01:44.967Z
    notes: "lints do REF/knudge adaptados em [workspace.lints] (deny unwrap/expect/panic/etc); Makefile com check=fmt+clippy -D warnings+test+layers+file-length; make check verde"
anchors: [Cargo.toml, clippy.toml, Makefile]
status: closed
scope: task
---
<!-- knudge:parent task_00qm4eu6 -->