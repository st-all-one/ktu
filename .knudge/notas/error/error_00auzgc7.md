---
id: error_00auzgc7
type: error
statement: katu: um assistant por tool_call causou HTTP 400 com 2+ tool calls
created_at: 2026-10-13T00:52:07.579Z
body_hash: a6b58b2e
schema_version: 2
tags: [ktu, error, provider]
supports: [decision_01qcf59e]
revision: 2
anchors: [plan.md]
---
Por quê: um passo com N calls era emitido como N mensagens assistant.
Evidência: REF/katu STAGING.md (invalid_request_error do upstream).
Consequência: projeção de wire separada — 1 passo = 1 assistant com N tool_calls + N tool results.