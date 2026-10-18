---
id: fact_0081qtit
type: fact
statement: ktu run embedded compoe o system prompt com o AGENTS.md do root e usa provider de demonstracao em F1
created_at: 2026-10-13T01:18:17.578Z
body_hash: 8d59cb00
schema_version: 2
revision: 2
anchors: [crates/ktu-cli/src/run.rs]
---
Evidência: ktu-cli/src/run.rs + provider_fake.rs; ktu run 'le AGENTS.md' imprime a resposta em stdout e eventos JSON em stderr. Consequência: o provider llama.cpp real entra em F2 sem tocar no loop.