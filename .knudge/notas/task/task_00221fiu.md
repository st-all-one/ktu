---
id: task_00221fiu
type: task
statement: ktu-memory consome a Source (elimina a reimplementacao da leitura)
created_at: 2026-10-17T22:54:33.180Z
body_hash: 4678d757
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:21:22.268Z
    notes: "ktu-memory consome as regras do knudge: eliminado src/record.rs (codec reimplementado); load_notes/load_report usam knudge_core::corpus::parse_jsonl e a escrita usa Note::to_jsonl. A porta Fs continua a ser a do kernel (nao se adapta porta alheia): o que se partilha e o codec, a dedup-on-read e a tolerancia, nao o plumbing. Passa a gravar proveniencia (agent=ktu, activity=write|derive)."
anchors: [wiki/backlog/knudge-store-write.md]
status: closed
scope: task
---
<!-- knudge:parent task_00104ugh -->