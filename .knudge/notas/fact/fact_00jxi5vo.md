---
id: fact_00jxi5vo
type: fact
statement: "O provider falso do F1 reencena Completion tipados de forma deterministica, nao fixtures byte-a-byte"
created_at: 2026-10-13T01:40:13.977Z
body_hash: 74a18123
schema_version: 2
anchors: [crates/ktu-core/src/testing.rs]
---
Evidência: testing.rs::FakeProvider guarda Vec<Completion> e avanca um cursor; nao ha ficheiros de fixture nem bytes HTTP. Os fixtures byte-a-byte de streaming/parsing sao a entrega do G2 task_01j2o6cg (test double HTTP).