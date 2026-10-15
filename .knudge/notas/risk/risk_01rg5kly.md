---
id: risk_01rg5kly
type: risk
statement: Em CPU-only um turno do Ornith-1.5-9B-Q4_K_M demorou ~8 minutos
created_at: 2026-10-13T02:39:57.853Z
body_hash: bcdca031
schema_version: 2
anchors: [scripts/run-server.sh]
---
Evidência: turn.start 02:31:47 -> turn.end 02:39:30 (~463 s) para um turno pequeno (glob+read+resposta). Causa: modelo denso 9B em CPU + reasoning por omissao. Consequência: a bateria G4 tem varias perguntas e pode ser lenta; mitigacoes possiveis: reduzir o contexto/tecto de geracao, desligar thinking via chat_template_kwargs enable_thinking=false para tarefas simples, ou aceitar o wall-clock (performance e terciaria).