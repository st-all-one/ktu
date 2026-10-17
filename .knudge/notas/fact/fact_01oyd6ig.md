---
id: fact_01oyd6ig
type: fact
statement: "ktu serve: axum HTTP/1.1+HTTP/2 com SSE, health, estado, turno, cancel, modo e web"
created_at: 2026-10-17T02:06:38.211Z
body_hash: "74024766"
schema_version: 2
anchors: [crates/ktu-cli/src/serve.rs]
---
crates/ktu-cli/src/serve.rs: GET /v1/health, POST /v1/session, GET /v1/session/{id}, POST .../turn (202), GET .../events (SSE), POST .../cancel, POST .../mode, GET / (pagina web fetch+SSE). TESTE de integracao (tests/serve_http.rs) sobe o binario e conduz a sessao via HttpPort.