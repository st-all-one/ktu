---
id: task_003n4ebj
type: task
statement: Reorganizar o clap em areas (CONVERSA/CONHECIMENTO/PROJECTO/SISTEMA/PORTA) + flags globais
created_at: 2026-10-17T22:54:05.931Z
body_hash: 99cfe16a
schema_version: 2
tags: [fase2]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:16:07.138Z
    notes: "areas: ordem por area no --help + rodape AFTER_HELP; flags globais --json/--quiet/--log-level (wired ao logging); reorder do enum Command; envelope JSON (output.rs, Format enum); make check verde"
anchors: [crates/ktu-cli/src/cli.rs, crates/ktu-cli/src/output.rs, wiki/backlog/cli-superficie.md]
status: closed
scope: task
---
<!-- knudge:parent task_00qn6s14 -->