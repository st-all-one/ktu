---
id: fact_002xhvz0
type: fact
statement: O portao unico de qualidade do ktu e make check (fmt + clippy -D warnings + test + layers + file-length)
created_at: 2026-10-13T01:01:50.426Z
body_hash: 596a19d9
schema_version: 2
anchors: [Makefile, Cargo.toml]
---
Evidência: Makefile; make check verde no esqueleto F0 (7 crates, 1 teste cada). Consequência: nenhuma fase fecha sem este portão verde.