---
id: fact_009g1lxx
type: fact
statement: Gateway OpenAI em thinking mode exige reasoning_content reenviado; llama-server nao
created_at: 2026-10-17T10:52:09.654Z
body_hash: 0178f14d
schema_version: 2
anchors: [crates/ktu-provider/src/wire.rs]
---
Message::Assistant ganhou o campo reasoning (preenchido de Event::Assistant). wire.rs::message_wire reenvia reasoning_content em TODAS as mensagens de assistente quando Flavor::OpenAi (mesmo vazio); o dialeto Llama continua a descartar o pensamento. Corrige HTTP 400 do deepseek-v4.1-flash ("reasoning_content in the thinking mode must be passed back"). Testes openai_flavor_echoes_reasoning_content e openai_flavor_echoes_empty_reasoning_too.