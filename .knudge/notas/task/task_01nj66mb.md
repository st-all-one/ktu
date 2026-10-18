---
id: task_01nj66mb
type: task
statement: Exemplo de embedding + doc + teste de integracao
created_at: 2026-10-17T22:54:41.769Z
body_hash: d7067d4c
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:11:38.327Z
    notes: "Exemplo + doc + teste. O exemplo de incorporacao e um DOC TEST EXECUTAVEL (sem no_run) na documentacao do modulo facade: fakes do proprio ktu-core (MemFs, FixedClock, NoopLogger, NoopMetrics, FakeProvider, FakeTools), AgentRegistry/AgentSpec, BudgetGate, CancelToken, runtime tokio inline. Teste de integracao crates/ktu-core/tests/embed.rs (3): um embedder externo corre um turno com tools e persiste a sessao (round-trip JSONL byte-identico), o mesmo script da logs byte-identicos (I6) e o token de cancelamento injectado para o turno (I4). 10 testes unitarios em facade/tests.rs: build nomeia a porta em falta, turno completo com fakes, tool pelo executor injectado, AGENTS.md no system prompt, guardrail quebrado/permissao quebrada param o build (I16), modo plano sem plan_spec recusado e com plan_spec so oferece read-only, round-trip de sessao, final_text so do turno corrente. Docs: wiki/fachada-crate.md (contrato completo), H9 e backlog/crate-facade actualizados, README com o uso #1."
anchors: [wiki/backlog/crate-facade.md]
status: closed
scope: task
---
<!-- knudge:parent task_00dkc6dk -->