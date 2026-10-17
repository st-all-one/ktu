---
id: fact_00zwualq
type: fact
statement: "Revisao do goose: 3 features a incorporar (receitas, slash commands, permission store); 4 a avaliar"
created_at: 2026-10-17T12:34:56.907Z
body_hash: "54218748"
schema_version: 2
anchors: [wiki/revisao-goose.md]
---
wiki/revisao-goose.md analisa o REF/goose (v1.54.0; 275 757 linhas src, 3660 testes, 1319 pacotes) com o filtro simplicidade/kernel/usabilidade objectiva. Incorporar: receitas declarativas (recipe), slash commands e permission store (aprovacoes memorizadas). Avaliar com medicao: resumo estruturado de compactacao (Q4 do gate), contagem de tokens, limite de contexto por modelo, export de sessao. Nao trazer: MCP, ACP, desktop, candle, roaming, scheduler, OAuth/15+ providers, voz, OTel, download manager e o state machine do goose-agent. As 7 candidatas ficaram como paginas 19-25 no wiki/backlog/.