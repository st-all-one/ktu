---
id: fact_00muvzs7
type: fact
statement: H1 concluida: ktu init idempotente (ktu-core::git puro + borda init.rs); git gerido; make check verde
created_at: 2026-10-17T23:39:10.013Z
body_hash: fe084cb6
schema_version: 2
anchors: [crates/ktu-core/src/git/mod.rs, crates/ktu-cli/src/init.rs]
---
H1 (task_01v41q8n) concluida. Abordagem revista: a logica pura (upsert de blocos, render de AGENTS.md/.gitattributes, padroes de exclude) vive num novo modulo ktu-core::git (block/agents/attributes/exclude), golden-testavel sem I/O; os efeitos (ler/escrever) ficam na borda ktu-cli::init (a porta Fs nao tem create_dir_all/write_atomic; init.rs ja usava std::fs). Marcadores: AGENTS.md <!-- ktu:start/end --> + <!-- ktu:version: 1 -->; .gitattributes # ktu:start/end. .git/info/exclude gerido: tracked -> /.agents/memory/index/ + /.ktu/; excluded -> /.agents/ + /.ktu/; alternar REVERTE (verificado). Nunca .gitignore. .gitattributes: eol=lf (ids estaveis), merge=union no log de eventos, binary/linguist-generated no derivado. --force reescreve SO skill+config; nunca memoria nem guardrails editados. --no-prompt; relatorio por item + envelope --json. Fora de repo git: no-op, exit 0. Layout+skill+config em ktu-core::layout (SKILL_FILE/BODY, CONFIG_FILE/TEMPLATE). Desvios YAGNI: NAO criados .agents/recipes/ nem .ktu/logs/ (nada os usa; plan.md 2.11 nem os lista) — ficam para H10. Lints: flags->enums (Overwrite, GitMode), Options com 1 bool, indexing_slicing evitado (get(..)). Ficheiros: ktu-core/src/git/{mod,block,agents,attributes,exclude}.rs, layout.rs, ktu-cli/src/{init,cli,main}.rs. Estado: make check + typos + machete verdes; init.rs 397 linhas (limite 400).