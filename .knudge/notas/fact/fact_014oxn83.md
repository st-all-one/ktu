---
id: fact_014oxn83
type: fact
statement: ktu-port: protocolo NDJSON/SSE versionado + Port (Channel e HttpPort) e Host no servidor
created_at: 2026-10-17T02:06:37.642Z
body_hash: fd337df2
schema_version: 2
anchors: [crates/ktu-port/src/protocol.rs]
---
crates/ktu-port: protocol.rs (ClientMsg/ServerMsg, versao 1), port.rs (trait Port), host.rs (trait Host), channel.rs (in-process, sem socket) e http.rs (HTTP/1.1+HTTP/2, SSE com parser de linhas puro). O ktu-port nao contem logica de agente. Testes: round-trip NDJSON e parser SSE.