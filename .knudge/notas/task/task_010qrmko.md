---
id: task_010qrmko
type: task
statement: Adaptador GDK (goose-sdk) atrás da feature gdk para os demais providers
created_at: 2026-10-13T00:51:17.729Z
body_hash: 98f76eb5
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: abandoned
    recorded_at: 2026-10-17T12:14:05.244Z
    notes: "Decidido nao implementar o adaptador GDK (goose-sdk/goose-providers). Evidencia: alpha 0.1.0; goose-sdk default = tipos ACP (goose acp por stdio); goose-providers puxa ~167 pacotes extra incluindo rmcp/MCP (YAGNI) e exige rede (nao-deterministico); desajuste arquitectural (goose e agente). O caminho OpenAI-compatible (opencode-go) cobre a necessidade remota. Alternativa documentada: provider fino por dialeto (ex.: Anthropic Messages), testavel com o test double HTTP. Ver decision da G9."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01t36xit -->