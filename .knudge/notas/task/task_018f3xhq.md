---
id: task_018f3xhq
type: task
statement: Modo plano: AgentSpec read-only + tool plan + artefacto .ktu/plan/<sessao>.json + transição aprovada (I15)
created_at: 2026-10-13T00:50:49.854Z
body_hash: 3ea0ad9e
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T00:31:55.287Z
    notes: Modo plano implementado: Mode/Approval/Plan em ktu-core/src/plan.rs com artefacto atomico .ktu/plan/<sessao>.json (plan/store.rs) e FsPlanHost (plan/host.rs); tool plan (stage_add/task_add/task_state/list); plan_mode_spec read-only (ScopedTools+guard negam escrita); modo logado como Event::ModeSet (sobrevive a resume); plan->build exige --approve (I15); CLI --plan e ktu mode. Testes 7 unit + integracao plan_mode + smoke e2e; make check verde.
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00j43pma -->