---
id: task_00kzk1n0
type: task
statement: "kd ai [memory|session-log] como area fina sobre a Source"
created_at: 2026-10-17T22:54:32.841Z
body_hash: 4bf0da81
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:21:22.113Z
    notes: "kd ai [memory|session-log] [QUERY...] [--source PATH] [--limit N] (D218): area posicional (default memory), default consulta (sem query lista id|tipo|statement; com query faz recall id|statement|score, [no_results] se vazio). Convencoes .agents/memory/notes.jsonl e .agents/memory/session-log.jsonl. Read-only (sem --add). Area invalida -> exit 2 (D212); fonte ausente -> vazio, exit 0; linha ma -> aviso. Testes em crates/knudge-cli/tests/ai.rs (7)."
anchors: [wiki/backlog/kd-ai.md]
status: closed
scope: task
---
<!-- knudge:parent task_00104ugh -->