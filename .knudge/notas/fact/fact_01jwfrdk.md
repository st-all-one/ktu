---
id: fact_01jwfrdk
type: fact
statement: H0.4 parcial: prime + self version/upgrade/completions + help; setup deferido (receitas H10); H0 concluida
created_at: 2026-10-17T23:29:20.641Z
body_hash: f8b7f14d
schema_version: 2
anchors: [crates/ktu-cli/src/self_cmd.rs]
---
H0.4 (task_01l8s4ev) fechada como partial. Feito: ktu prime (protocolo byte-identico por versao, crates/ktu-cli/src/prime.rs); ktu self version; ktu self upgrade (install.sh local ou comando canonico); ktu self completions <shell> (novo, via clap_complete 4: bash/zsh/fish/powershell/elvish; shell invalido -> erro); ktu help <verbo> (clap). Nova dep clap_complete (workspace + ktu-cli); cargo machete limpo. typos.toml: aceites posicional e oficial (vocabulario PT). Deferido: ktu self setup (instala recipe de cliente) — precisa do sistema de receitas (H10/G1). Com isto a H0 fica concluida: H0.1 (areas+flags+envelope), H0.2 (--params), H0.3 (--batch), H0.4 (prime/self/completions, setup deferido), H0.5 (envelope em todos os verbos + exit codes). Estado: make check + typos + machete verdes; ktu-cli 28+2+1 testes.