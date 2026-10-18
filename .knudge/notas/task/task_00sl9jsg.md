---
id: task_00sl9jsg
type: task
statement: Lotes: batch <JSONL> (paralelo default com --concurrency; --sequential numa sessao)
created_at: 2026-10-17T22:54:26.875Z
body_hash: 2e8b8245
schema_version: 2
tags: [fase2]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:23:00.869Z
    notes: "batch.rs: JSONL de params; paralelo default (--concurrency, sessoes batch-<i>) ou --sequential (mesma sessao); resultados ordenados pelo indice; run_capture extraido; make check verde"
anchors: [crates/ktu-cli/src/batch.rs, crates/ktu-cli/src/run.rs, crates/ktu-cli/src/main.rs, wiki/backlog/cli-superficie.md]
status: closed
scope: task
---
<!-- knudge:parent task_00qn6s14 -->