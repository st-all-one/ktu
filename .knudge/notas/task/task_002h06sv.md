---
id: task_002h06sv
type: task
statement: Efemero por omissao (ktu/ktu <prompt> sem persistir); --session persiste
created_at: 2026-10-17T22:54:09.768Z
body_hash: cc1521ae
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T23:58:04.409Z
    notes: "SessionTarget (Ephemeral/Persistent) no RunConfig; ktu run sem --session e efemero (nao carrega nem grava log/checkpoint, nao cria .ktu/sessions); --session <id> persiste; --resume/--compact implicam default; batch/serve/hub persistentes; subagentes criam o dir on demand"
anchors: [wiki/backlog/modo-efemero.md]
status: closed
scope: task
---
<!-- knudge:parent task_017rrpwn -->