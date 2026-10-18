---
id: fact_00jd66nv
type: fact
statement: "H6+: steering no REPL (durante o turno) - porta Host::steer, Event::Steer logado, entrega em fronteira segura"
created_at: 2026-10-18T00:50:42.935Z
body_hash: 5e666b4d
schema_version: 2
anchors: [crates/ktu-core/src/agent/deps.rs, crates/ktu-core/src/agent/mod.rs, crates/ktu-cli/src/repl.rs, crates/ktu-cli/src/hub.rs]
---
Steering implementado (pedido do utilizador: "dentro do repl nao ha alguma maneira de ter isso?"). Kernel: nova porta ktu_core::agent::Steering (take() -> Vec<String>) em Capabilities.steering; o loop entrega o steering pendente no INICIO DE CADA PASSO (fronteira segura) e loga-o como Event::Steer { text } (project -> Message::User; verify ignora; compact resume-o) - preserva model-visible <=> logged. Novo event id turn.steer em events::ALL. Porta: Host::steer + Port::steer (Channel/HttpPort) + rota POST /v1/session/{id}/steer; SessionHub enfileira no buffer do Handle (nao passa pelo canal de comandos, que o actor esta ocupado a correr o turno) e o actor le-o via HandleSteering. REPL: o stdin passou a ser lido numa thread dedicada (canal tokio) e, durante o turno, o input e drenado SEM BLOQUEAR entre eventos (try_recv -> hub.steer); /cancel durante o turno cancela; a mensagem aparece como "(steer) ..." no stderr. Nota: tokio::select! foi tentado e rejeitado - disallowed_macros bane std::panic/std::unreachable, que a macro expande. Testes: 1 de loop (steering logado + no contexto) + 1 de integracao (linha durante o turno -> "steer" no jsonl; estavel em 8 corridas). Restante -> A8 (task_01vw262v): enfileiramento duravel antes da entrega + idempotencia por ID. Gate: make check + typos + machete verdes.