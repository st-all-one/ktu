---
id: decision_00aw5etj
type: decision
statement: GDK nao implementado: goose-sdk alpha (ACP) + goose-providers com rmcp/MCP e ~167 crates extra
created_at: 2026-10-17T12:14:13.461Z
body_hash: 9bcf5520
schema_version: 2
anchors: [plan.md, crates/ktu-provider/src/wire.rs]
---
Avaliacao da G9 (task_010qrmko): (1) goose-sdk 0.1.0-alpha.12 com features por omissao so reexporta tipos ACP para falar com `goose acp` por stdio; a biblioteca de providers e goose-providers (tambem alpha). (2) goose-providers puxa ~167 pacotes extra (Cargo.lock de um crate novo), incluindo rmcp/MCP (na lista YAGNI do plan.md 0); o ktu tem 267 crates unicos. (3) Nao-deterministico de testar (rede; sem test double HTTP proprio). (4) Desajuste arquitectural: o goose e um agente (ACP) ou traz tipos de mensagem proprios; integra-lo como Provider obriga a adaptador e/ou segundo agente. (5) O caminho OpenAI-compatible (opencode-go) cobre a necessidade remota atual. Decisao: nao implementar agora. Alternativa se surgir necessidade de Anthropic/dialetos: provider fino por dialeto (ex.: Anthropic Messages), deterministico e testavel com o test double HTTP existente.