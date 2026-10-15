---
id: decision_016svmhn
type: decision
statement: "Firewall de camadas do ktu é imposto pelo alvo xtask check-layers, nao por revisao manual"
created_at: 2026-10-13T01:01:50.331Z
body_hash: fd5197e8
schema_version: 2
anchors: [xtask/src/main.rs, Cargo.toml]
---
Por quê: a arquitectura §2.1 só se mantém se for verificada por máquina. Evidência: xtask/src/main.rs lê cargo metadata e recusa ktu-port→ktu-memory (violação não-cíclica) com exit 1. Consequência: qualquer dependência proibida falha o portão antes do merge.