---
id: task_01vw262v
type: task
statement: Inbox duravel: admitir o steering no log antes da entrega e idempotencia por id (M7)
created_at: 2026-10-18T00:50:24.485Z
body_hash: b68624b8
schema_version: 2
tags: [fase3]
revision: 5
outcomes:
  - status: success
    recorded_at: 2026-10-18T14:01:07.540Z
    notes: "Inbox duravel: a admissao fica no log (Event::SteerQueued) antes da entrega (Event::Steer), na fronteira do loop; o id vem da borda (Host::steer(session_id, id, text)), pelo que repetir o pedido nao duplica. Gate G16 verde 9/9 (wiki/gates/G16-steering.md, artefactos em wiki/gates/artifacts/G16/). Limite: a janela coberta e admissao registada -> morte antes da entrega (a gravacao do log e do actor; ver decision_019o5bcj)."
anchors: [wiki/implementation_phase3/F1-steering-duravel.md]
status: closed
scope: task
---
O steering JA funciona: entregue numa fronteira segura (inicio de passo), logado como Event::Steer (duravel no log da sessao, a fonte da verdade) e projectado para o modelo (model-visible <=> logged); porta Host::steer + HttpPort + rota /v1/session/{id}/steer; REPL le stdin numa thread dedicada e drena sem bloquear entre eventos. Falta apenas: (a) enfileiramento duravel ANTES da entrega (perde-se se o processo morrer a meio); (b) idempotencia por ID; (c) modo queue (pendente enquanto a sessao pode continuar). Reabrir quando houver um front multi-cliente (web/TUI) com admissao idempotente ou requisito de nao perder a mensagem num crash.
<!-- knudge:parent task_00ly3qez -->