---
id: task_01vw262v
type: task
statement: A8 - Inbox duravel / idempotencia do steering
created_at: 2026-10-18T00:50:24.485Z
body_hash: f91da500
schema_version: 2
tags: [fase3]
anchors: [crates/ktu-core/src/agent/deps.rs, crates/ktu-core/src/session/mod.rs]
scope: task
---
O steering JA funciona: entregue numa fronteira segura (inicio de passo), logado como Event::Steer (duravel no log da sessao, a fonte da verdade) e projectado para o modelo (model-visible <=> logged); porta Host::steer + HttpPort + rota /v1/session/{id}/steer; REPL le stdin numa thread dedicada e drena sem bloquear entre eventos. Falta apenas: (a) enfileiramento duravel ANTES da entrega (perde-se se o processo morrer a meio); (b) idempotencia por ID; (c) modo queue (pendente enquanto a sessao pode continuar). Reabrir quando houver um front multi-cliente (web/TUI) com admissao idempotente ou requisito de nao perder a mensagem num crash.
<!-- knudge:parent task_016wnwcs -->