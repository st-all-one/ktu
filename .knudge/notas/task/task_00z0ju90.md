---
id: task_00z0ju90
type: task
statement: Memory gate no loop: recall de início de sessão + pre_write com dedup idempotente (0.75 cria / merge / rejeita)
created_at: 2026-10-13T00:50:49.134Z
body_hash: b1c0ef9e
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:54:54.141Z
    notes: "Memory gate no loop implementado: Deps ganhou memory: Option<&dyn Memory>; no inicio de sessao (log vazio) o loop chama recall com a instrucao, emite o evento memory.recall e anexa os ponteiros (compose_hints) ao system do turno — so no primeiro turno, para nao invalidar o cache de prefixo. pre_write via tools memory_ask/memory_write (ktu-tools::MemoryTools) com dedup idempotente 0.75/0.92 do adaptador. Teste de integracao loop::memory_is_recalled_only_at_session_start prova recall 1x e so no inicio. make check verde (84 testes)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01iz1xh1 -->