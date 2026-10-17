---
id: fact_010oxnp6
type: fact
statement: ktu run/serve --help vazava a API key; corrigido com hide_env_values no clap
created_at: 2026-10-17T11:32:52.143Z
body_hash: b08a021c
schema_version: 2
anchors: [crates/ktu-cli/src/cli.rs, plan.md]
---
Os args --api-key (env OPENCODE_GO_API_KEY) imprimiam o valor no --help ([env: OPENCODE_GO_API_KEY=oc_sk_...]). Corrigido com hide_env_values = true nos dois args (Run e Serve) em crates/ktu-cli/src/cli.rs; agora mostra [env: OPENCODE_GO_API_KEY] sem valor. Tambem atualizado plan.md 2.5/riscos: o reasoning nao e reenviado ao llama-server mas e reenviado como reasoning_content em gateways OpenAI em thinking mode.