---
id: fact_00ghyy87
type: fact
statement: Modo plano: ModeSet logado sobrevive a resume; AgentSpec read-only+plan; transicao exige Approval
created_at: 2026-10-17T00:31:54.992Z
body_hash: fd881d93
schema_version: 2
anchors: [crates/ktu-core/src/plan.rs, crates/ktu-core/src/session/mod.rs, crates/ktu-tools/src/agents.rs]
---
ktu-core/src/plan.rs (Mode/Approval/Plan) + plan/actions.rs (tool plan: stage_add/task_add/task_state/list) + plan/store.rs (artefacto .ktu/plan/<sessao>.json atomico temp->fsync->rename) + plan/host.rs (FsPlanHost). O modo vive no log da sessao (Event::ModeSet) e sobrevive a resume; build->plan e livre, plan->build exige Approval::Approved (I15); ktu mode <plan|build> [--approve]. plan_mode_spec = read-only + plan + subagent; ScopedTools+guard negam escrita. CLI --plan. Testes: 7 unit plan + plan_mode (ktu-tools) + smoke e2e; make check verde.