---
id: decision_00c1uuue
type: decision
statement: "O confinamento de path e estrutural (resolve_within lexical, recusa absoluto e .. que escape), nunca regex"
created_at: 2026-10-13T01:18:17.478Z
body_hash: a9f8e0c3
schema_version: 2
anchors: [crates/ktu-core/src/guard.rs]
---
Por quê: tese §2.8: enforcement são factos tipados, fail-closed. Evidência: ktu-core/src/guard.rs com testes adversariais (/etc/passwd, ../..). Consequência: write_file/edit_file (F2) usam o mesmo funil.