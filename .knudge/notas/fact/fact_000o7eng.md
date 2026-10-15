---
id: fact_000o7eng
type: fact
statement: A porta Memory do ktu vive em ktu_core::memory (o kernel precisa dela); o adaptador JsonlMemory vive em ktu-memory
created_at: 2026-10-13T02:50:45.092Z
body_hash: d468c12d
schema_version: 2
anchors: [crates/ktu-core/src/memory.rs]
---
Evidência: crates/ktu-core/src/memory.rs (trait Memory, Hint, MemoryDraft, MemoryWrite, MemoryError) e crates/ktu-memory/src/store.rs (JsonlMemory implementa Memory). Por quê: o firewall de camadas proíbe ktu-core depender de ktu-memory, mas o loop (kernel) tem de chamar o recall; logo o contrato fica no núcleo e o backend na borda (DI). Hints sao ponteiros (id+statement+score).