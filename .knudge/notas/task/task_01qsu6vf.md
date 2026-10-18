---
id: task_01qsu6vf
type: task
statement: "A6 - H5: fila de mutacao, remedio por regra e ancoras proximas"
created_at: 2026-10-18T00:31:58.552Z
body_hash: 91f765c7
schema_version: 2
tags: [fase3]
anchors: [crates/ktu-tools/src/tool_thread.rs, crates/ktu-core/src/guard.rs]
scope: task
---
Adiados da H5. (a) Fila de mutacao por caminho: as tools correm sequencialmente numa unica thread T2 (WorkerCtx::serve serve um pedido de cada vez; subagentes partilham a T2) - nao ha escrita concorrente ao mesmo ficheiro (YAGNI). (b) Remedio por regra de guardrail: Decision::Deny(DenyReason) nao transporta dados; o remedio estrutural por DenyReason ja cobre todos os motivos. (c) edit a sugerir as ancoras unicas mais proximas: a mensagem actual ja devolve a ancora procurada. Reabrir quando houver paralelismo de tools / uma regra precisar de remedio proprio / sintoma de ancoras erradas.
<!-- knudge:parent task_016wnwcs -->