---
id: task_01htyks4
type: task
statement: Source JSONL read-only no knudge-core (reusa Note/Frontmatter)
created_at: 2026-10-17T22:54:31.788Z
body_hash: 1b66e331
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:21:21.602Z
    notes: "Source JSONL read-only no knudge-core (D218): corpus::Source {NotesDir default | Jsonl(path)} + Loaded + parse_jsonl (parte pura, partilhavel por quem tem outra porta Fs) + codec Note::to_jsonl/from_jsonl (JSON canonico do knudge). Dedup-on-read por id (D26), linha malformada = skip + warning (D18), ordem deterministica por id. Testes: golden JSONL vs notas/ (mesmo indice e grafo), dedup, malformada, ausente."
anchors: [wiki/backlog/kd-ai.md]
status: closed
scope: task
---
<!-- knudge:parent task_00104ugh -->