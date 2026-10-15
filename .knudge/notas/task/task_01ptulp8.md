---
id: task_01ptulp8
type: task
statement: Sessão: resume por id + cache de prefixo (determinismo de prefixo; não invalidar) + checkpoint
created_at: 2026-10-13T00:50:48.868Z
body_hash: 7be7eba8
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:57:53.309Z
    notes: "Persistencia da sessao: ktu-core/src/store.rs (to_jsonl/from_jsonl/save/load) grava .ktu/sessions/<id>.jsonl de forma duravel (temp + rename atomico; porta Fs ganhou rename). CLI: --session <id> e --resume carregam/continuam o log; verificado e2e: run1 grava 6 eventos, run2 --resume carrega-os e acrescenta 6. Determinismo de prefixo: teste wire_projection_is_deterministic_and_prefix_stable prova que a projecao de wire e reproduzivel e que acrescentar um turno nao altera as mensagens anteriores (o reasoning nunca e reenviado). 3 testes de store + 1 de sessao; make check verde (68 testes)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->