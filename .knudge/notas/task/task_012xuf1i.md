---
id: task_012xuf1i
type: task
statement: "stdin/heredoc (ktu -, pipe) + envelope JSON + codigos de saida documentados"
created_at: 2026-10-17T22:54:06.561Z
body_hash: 6e31db72
schema_version: 2
tags: [fase2]
revision: 6
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:27:12.755Z
    notes: "stdin/heredoc (run -); envelope --json em todos os verbos de dados (prime, self, memory list/export/doctor, guardrails list, plugins list/reload/doctor, trash list/restore, providers doctor); codigos de saida documentados no prime e no rodape (0/1/2); make check verde"
anchors: [crates/ktu-cli/src/output.rs, crates/ktu-cli/src/memory_cmd.rs, crates/ktu-cli/src/guardrails_cmd.rs, crates/ktu-cli/src/plugins_cmd.rs, crates/ktu-cli/src/trash_cmd.rs, crates/ktu-cli/src/providers.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00qn6s14 -->