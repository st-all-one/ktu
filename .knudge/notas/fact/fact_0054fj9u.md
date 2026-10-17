---
id: fact_0054fj9u
type: fact
statement: Revisao do pi: 1 a incorporar (prompt templates) + 10 a avaliar; arvore de sessao e a ideia mais forte
created_at: 2026-10-17T12:42:53.906Z
body_hash: 9b3af652
schema_version: 2
anchors: [wiki/revisao-pi.md]
---
wiki/revisao-pi.md analisa o REF/pi (monorepo TS; 172 864 linhas src, 634 ficheiros de teste; 11 pacotes) com o filtro simplicidade/kernel/usabilidade. pi e o harness mais proximo do ktu (minimal/extensivel) e diz explicitamente que skips sub-agents e plan mode, que o ktu tem. Incorporar: prompt templates (Markdown -> comando /, com argumentos). Avaliar: arvore de sessao (branching/fork/clone; a ideia mais forte, toca o log), steering/follow-up, virtual models, pi packages, codemode (JS sandbox que compoe tools), temas TUI, project trust, compactacao parametrizada (reserve/keepRecent + resumo iterativo), sessoes versionadas + migracao, chord (contratos/estado de plugins). Nao trazer: ai/durable/mcp/env/tui/telemetry/evals, extensoes TS in-process, npm como transporte. As 11 candidatas ficaram como paginas 40-50 no wiki/backlog/.