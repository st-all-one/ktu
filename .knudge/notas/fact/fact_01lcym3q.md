---
id: fact_01lcym3q
type: fact
statement: Guardrails declarativos fail-closed (I16) com schema .md e ChainGuard; seed em .agents/guardrails
created_at: 2026-10-17T07:02:33.550Z
body_hash: 2208eff5
schema_version: 2
anchors: [crates/ktu-core/src/guardrails.rs]
---
ktu-core/src/guardrails.rs: schema (id/version/applies_to/fact/match/decision/reason), loader fail-closed (ficheiro invalido impede o arranque, nomeando-o), regras que tentam permitir sao ignoradas, ChainGuard(structural->declarativo) so aperta. Seed de 3 regras gravado por ktu init. Comandos: ktu guardrails list.