---
id: task_00n15wb8
type: task
statement: SYSTEM.md/APPEND_SYSTEM (97) + exposicao de tools declared/deferred (93)
created_at: 2026-10-17T22:54:43.158Z
body_hash: 1c673305
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:58:23.920Z
    notes: "SYSTEM.md/APPEND_SYSTEM.md (metade da tarefa; a exposicao de tools ficou adiada): ktu-core/src/prompt.rs ganhou SYSTEM_FILE/APPEND_SYSTEM_FILE, Directive{load fail-open, resolve com projecto>utilizador ficheiro a ficheiro, system/append/is_empty} e compose_layers(system, agents_md, append) (ordem: directriz, AGENTS.md, acrescento; camadas vazias omitidas); facade::system_prompt_in(fs, root, user_dir) + KtuBuilder::user_dir + layout::{AGENTS_DIR,AGENTS_FILE,SYSTEM_FILE,APPEND_SYSTEM_FILE}; a CLI resolve o dir do utilizador (config::global_dir via XDG_CONFIG_HOME/HOME) e passa-o ao builder e ao host de subagentes; novo verbo ktu prompt (mostra o prompt efectivo, --json com bytes e user_dir) documentado no prime. Testes: unit (Directive/compose_layers) + facade (projecto ganha, substitui/acrescenta) + ktu-cli/tests/commands.rs::prompt_shows_the_effective_system_prompt. ADIADO: exposicao declared/deferred + tool_search (A10/task_01moxmoz) - catalogo de 13 tools cabe no prompt."
anchors: [wiki/backlog/system-prompt-override.md]
status: closed
scope: task
---
<!-- knudge:parent task_01tp3uj0 -->