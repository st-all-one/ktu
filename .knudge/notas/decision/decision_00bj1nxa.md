---
id: decision_00bj1nxa
type: decision
statement: GDK adiado: goose-sdk (alpha) e ACP/goose-providers com rmcp; nao cabe em simplicidade/determinismo
created_at: 2026-10-17T11:21:10.466Z
body_hash: 1847c869
schema_version: 2
anchors: [plan.md]
---
O goose-sdk real (0.1.0-alpha.12) com features por omissao so reexporta tipos ACP (agent-client-protocol + goose-sdk-types) para falar com o binario `goose acp` por stdio; a biblioteca de providers e `goose-providers` (alpha), que depende de rmcp (MCP, na lista YAGNI) e exige rede (nao-deterministico de testar). Integrar GDK como Provider do ktu criaria um segundo agente a competir com o loop do kernel. Decisao: manter task_010qrmko aberta mas adiada; o caminho OpenAI-compatible (opencode-go) cobre os providers remotos atuais. Reavaliar se surgir necessidade concreta de Anthropic/dialetos.