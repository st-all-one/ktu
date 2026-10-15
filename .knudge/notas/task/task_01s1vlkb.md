---
id: task_01s1vlkb
type: task
statement: "ktu memory export --md (vista derivada) + ktu doctor --memory (âncoras quebradas, saúde do índice)"
created_at: 2026-10-13T00:50:49.250Z
body_hash: de6bc8fb
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T03:02:23.667Z
    notes: "ktu memory export/list e ktu doctor --memory implementados (ktu-cli/src/memory_cmd.rs). export imprime a vista Markdown derivada (export_markdown); list imprime id/tipo/afirmacao; doctor --memory reporta memory.notes/malformed/bytes/broken_anchors e sai FAILURE se houver anomalias. Testes de binario (crates/ktu-cli/tests/memory_cli.rs) correm o binario e validam export, list e doctor verdes. make check verde (92 testes)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01iz1xh1 -->