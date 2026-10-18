---
id: task_01hqpcjw
type: task
statement: A7 - H6: steering / admissao duravel (inbox) sem consumidor
created_at: 2026-10-18T00:40:01.618Z
body_hash: a50c7a1c
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T00:50:31.496Z
    notes: "Steering implementado: Capabilities.steering (porta Steering no kernel); Event::Steer no log (project -> Message::User; model-visible <=> logged); inject_steering no inicio de cada passo; Host::steer + HttpPort + rota /v1/session/{id}/steer; REPL le stdin numa thread dedicada e drena com try_recv entre eventos (sem select!, que e disallowed_macros); /cancel durante o turno. Testes: 1 de loop (steering logado+entregue) + 1 de integracao REPL (linha durante o turno -> steer no jsonl). Restante -> A8 (task_01vw262v)."
anchors: [crates/ktu-core/src/agent/mod.rs, crates/ktu-port/src/protocol.rs]
status: closed
scope: task
---
Adiado da H6. O steering (mensagens durante um turno, entregues na proxima fronteira segura) e a inbox duravel nao tem consumidor hoje: o `ktu run` e one-shot; o REPL drena o turno de forma sincrona (nao aceita input durante o turno); o `serve`/TUI enviaria um novo turno, recusado como "ja ha um turno em curso". Exige ainda um evento novo no log (inbox/steer) para preservar "model-visible <=> logged", e idempotencia por ID. Reabrir quando houver um front que aceite input durante o turno (REPL assincrono ou TUI com steering) - entao: fila na porta, um ponto de injecao no loop (logado, ordem deterministica), idempotencia.
<!-- knudge:parent task_016wnwcs -->