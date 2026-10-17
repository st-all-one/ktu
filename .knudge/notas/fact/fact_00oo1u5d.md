---
id: fact_00oo1u5d
type: fact
statement: "Machete limpo: deps nao usadas removidas (indexmap, tokio-util, thiserror/tracing/serde por crate)"
created_at: 2026-10-17T01:06:49.157Z
body_hash: 43d1c730
schema_version: 2
anchors: [Cargo.toml]
---
Removidas: indexmap (core+workspace), tokio-util (workspace, reservado mas nunca usado), thiserror/tracing do provider, thiserror do tools, serde/serde_json/thiserror do memory, e as 5 deps do ktu-port (stub, sem deps ate a G6). cargo machete reporta zero deps nao usadas; build e make check verdes.