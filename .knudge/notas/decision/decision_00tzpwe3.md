---
id: decision_00tzpwe3
type: decision
statement: "Testes do ktu: proptest para propriedades puras, goldens para contrato de bytes, CI em .github/workflows/ci.yml"
created_at: 2026-10-13T01:09:42.608Z
body_hash: 014e636a
schema_version: 2
anchors: [crates/ktu-core/tests/ports_properties.rs, .github/workflows/ci.yml]
---
Por quê: rigor sem modelo real; o harness roda o loop com provider falso quando existir (G1). Evidência: crates/ktu-core/tests/ports_properties.rs; goldens run/ini; CI faz make check + feature instrument. Consequência: o provider falso com fixtures entra em G1 (task_00mabsbe).