---
id: fact_006ubil7
type: fact
statement: Supply chain e CI endurecidos: deny.toml + deny/audit/machete/typos + miri smoke (sniff-css)
created_at: 2026-10-17T01:06:48.843Z
body_hash: 271fe95c
schema_version: 2
anchors: [deny.toml, .github/workflows/ci.yml]
---
deny.toml (advisories yanked=deny, licencas allowlist, wildcards=deny com allow-wildcard-paths, sources deny). CI com jobs check/supply-chain/miri; make deny|machete|typos. cargo machete limpo. cargo-deny/audit/typos correm no CI (nao instalados localmente).