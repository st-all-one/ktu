---
id: fact_00wn17un
type: fact
statement: Revisao profunda do goose: seguranca (injecao/egress/allowlist) e consciencia situacional (MOIM) sao o que falta
created_at: 2026-10-17T13:02:39.544Z
body_hash: 5048f44c
schema_version: 2
anchors: [wiki/revisao-goose-profunda.md]
---
wiki/revisao-goose-profunda.md le docs, modulos internos e codigo do REF/goose (v1.54.0) para alem dos candidatos iniciais (receitas, slash commands, permission store, resumo estruturado, tokens, contexto por modelo, export). Novos candidatos a incorporar: MOIM/memoria de trabalho (bloco <turn-context> por turno: hora, cwd, compactacao, orcamento; instrucoes persistentes re-lidas por turno, tecto 64KB, so acima de 32k); deteccao de injecao de prompt (patterns.rs: ThreatPattern com risk_level/category, confianca, limiar, alerta Allow Once/Deny, finding id; a outra metade do taint.rs); resposta grande -> ficheiro (>200k chars -> temp file + referencia); permissoes por tool (Always/Ask/Never, sobrepoe ao modo); migracoes de config (idempotente + variante read nao destrutiva); nome automatico de sessao; editar em sessao/fork (truncar ou forkar por prefixo). Avaliar: modo adversario (revisor LLM por tool call, fail-open, custo/nao-determinismo), hooks (eventos -> scripts, Open Plugins spec), checks de revisao (.agents/checks/*.md como subagentes), diagnostico empacotado (caps + redacao), allowlist, inspector de egress, success checks. Consolidacoes: tool shim -> 67, final output -> 74, smart approval -> 21, code mode -> 36/45, sumarizacao de output antigo -> 22, auto-compaction dois niveis -> 48. Nao trazer: MCP, ACP, desktop, candle, roaming, scheduler, OAuth, voz, OTel, download manager, gateway, state machine. 14 paginas novas no backlog (79-92).