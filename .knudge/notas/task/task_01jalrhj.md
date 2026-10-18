---
id: task_01jalrhj
type: task
statement: Ktu/KtuBuilder (root/ports/provider) que esconde o kernel
created_at: 2026-10-17T22:54:41.089Z
body_hash: 643ceda0
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:11:37.973Z
    notes: "Ktu/KtuBuilder em crates/ktu-core/src/facade/{mod,builder}.rs (ambos <=400 linhas). DECISAO: a fachada vive no ktu-core e recebe TUDO por injecao (portas + efeitos) - o ktu-core e puro (M2) e nao constroi adaptadores std; a alternativa (crate nova com adaptadores, como o Knudge/D214 do knudge) fica adiada. Obrigatorias: root, fs, clock, logger, metrics, provider, tools, budget, cancel, registry. Opcionais: plan_spec, memory, hooks, subagents, steering, max_steps. build() resolve a parte PURA do projecto (guardrails + permissoes fail-closed I16, system prompt BASE+AGENTS.md, LoopConfig) e falha alto nomeando o que falta. DESVIO ao plano: sem Env/Rng (portas declaradas sem consumidor no loop - cerimonia/YAGNI) e com budget/cancel/registry/plan_spec (controlo + escolha do agente) que o plano nao enumerava. Novo Error::Invalid (aditivo, non_exhaustive) para o fail-closed. layout::SESSIONS_DIR + Ktu::CONTEXT_BUDGET_TOKENS (dono unico do tecto, I14)."
anchors: [wiki/backlog/crate-facade.md]
status: closed
scope: task
---
<!-- knudge:parent task_00dkc6dk -->