---
id: task_00ghiyx7
type: task
statement: turn/session + tipos publicos (TurnOutcome/Event) estaveis
created_at: 2026-10-17T22:54:41.426Z
body_hash: "02096086"
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:11:38.157Z
    notes: "Ktu::turn(&self, log: &mut SessionLog, prompt) -> Result<TurnOutcome> e Ktu::session/save_session (persistencia pela porta Fs, store::load_recovering/save). O log e EXPLICITO: e a I3 tornada visivel e e o que a CLI precisa (lock, checkpoint, projecao) - o embedder ve Ktu/TurnOutcome/SessionLog/Event e as portas, nunca Deps/Ports/Capabilities/LoopConfig/run_turn (doc do run_turn e do Deps apontam para a fachada; privar run_turn/Deps fica adiado). O AgentSpec e escolhido PELO MODO (build -> registry.primary(); plan -> plan_spec) e aplicado em duas camadas (ScopedTools + StructuralGuard); sem plan_spec o modo plano e RECUSADO (I15, fail-closed) e o log nao e tocado. TurnOutcome { reason, text } com is_ok(); facade::final_text (o run.rs deixou de ter a sua copia) e facade::system_prompt (a CLI tambem usa, para o host de subagentes)."
anchors: [wiki/backlog/crate-facade.md]
status: closed
scope: task
---
<!-- knudge:parent task_00dkc6dk -->