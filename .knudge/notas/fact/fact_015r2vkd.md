---
id: fact_015r2vkd
type: fact
statement: H0 em curso: prime + self version/upgrade + rodape de areas + stdin/heredoc; make check verde (177+3 testes)
created_at: 2026-10-17T23:08:10.415Z
body_hash: 910f6d4f
schema_version: 2
anchors: [crates/ktu-cli/src/prime.rs]
---
Implementacao da H0 (superficie do CLI) iniciada, em incrementos com teste antes/depois. Baseline: 177 testes + clippy limpos. Incremento 1 (H0.4, task_01l8s4ev): novo crates/ktu-cli/src/prime.rs com PROTOCOL byte-identico por versao (ktu prime) e testes (nomes de todas as tools + contrato de saida); novo crates/ktu-cli/src/self_cmd.rs (ktu self version, ktu self upgrade que corre install.sh local ou imprime o comando canonico); rodape AFTER_HELP no Cli com as AREAS (CONVERSA/CONHECIMENTO/PROJECTO/SISTEMA/PORTA) e o ciclo; comandos Prime e SelfCmd (nome clap self). Deferidos: self completions/setup (precisam de clap_complete/recipe system). Incremento 2 (H0.5 parcial, task_012xuf1i): run::resolve_prompt le stdin quando o prompt e - ou vazio com pipe (heredoc); doc do arg actualizada; testes do join/vazio. Correcao de gate pre-existente: scripts/check_file_length.sh passou a excluir crates/knudge/ (checkout de exploracao, ja excluido do workspace) — o make check estava vermelho por causa do clone. Estado: make check VERDE (fmt+clippy+test+check-layers+file-length); 19 testes no ktu-cli (17+2 novos). Ficheiros: crates/ktu-cli/src/{prime,self_cmd}.rs (novos), cli.rs/main.rs/run.rs (alterados), scripts/check_file_length.sh.