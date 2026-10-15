---
id: task_016rvfop
type: task
statement: "Provider llama.cpp via llama-server (chat/completions streaming SSE, tool-calling nativo + fallback texto)"
created_at: 2026-10-13T00:50:48.668Z
body_hash: d73bded0
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:50:49.504Z
    notes: LlamaProvider em ktu-provider/src/llama.rs: POST /v1/chat/completions stream=true via reqwest (sem TLS); SSE framado (sse.rs) e acumulado (stream.rs); tool-calling nativo + fallback texto/thinking (fallback.rs); cancelamento observado a meio da stream (CancelToken no ProviderRequest -> ProviderError::Cancelled -> StopReason::Cancelled); retry/backoff para 5xx/indisponivel. request_body puro em wire.rs. 12 testes unitarios verdes.
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->