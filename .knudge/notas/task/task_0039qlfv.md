---
id: task_0039qlfv
type: task
statement: Supply chain e CI: deny.toml + cargo-deny/audit/machete/typos/miri inspirado no sniff-css
created_at: 2026-10-17T00:42:25.153Z
body_hash: 56a2ed1a
schema_version: 2
tags: [supply-chain]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T01:06:55.339Z
    notes: deny.toml + CI (check/supply-chain/miri) + make deny|machete|typos + tests/miri_smoke.rs; cargo machete limpo. cargo-deny/audit/typos correm no CI.
anchors: [deny.toml]
status: closed
scope: task
---
Adicionar deny.toml (advisories yanked=deny, licencas allowlist, wildcards=deny, allow-wildcard-paths, sources deny) e endurecer o clippy.toml/toolchain. CI: fmt + clippy + test + deny + audit + machete + typos + miri smoke (core puro). Makefile com alvos check/deny/machete/typos.
<!-- knudge:parent task_015epqsj -->