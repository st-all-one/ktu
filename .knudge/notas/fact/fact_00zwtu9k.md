---
id: fact_00zwtu9k
type: fact
statement: Directriz SYSTEM.md/APPEND_SYSTEM.md: projecto ganha ao utilizador; ktu prompt mostra o efeito
created_at: 2026-10-18T02:58:58.746Z
body_hash: e5750f22
schema_version: 2
anchors: [crates/ktu-core/src/prompt.rs, crates/ktu-core/src/facade/mod.rs, crates/ktu-cli/src/prompt_cmd.rs, wiki/convencoes.md]
---
H10: ktu-core/src/prompt.rs ganhou SYSTEM_FILE/APPEND_SYSTEM_FILE, Directive{load(fs,dir) fail-open, resolve(user,project) com precedencia ficheiro a ficheiro, system()/append()/is_empty()} e compose_layers(system, agents_md, append) que ordena directriz -> AGENTS.md -> acrescento e omite camadas vazias (compose(base, agents_md) passou a delegar, mantendo o comportamento). O nivel do utilizador e <global>/SYSTEM.md|APPEND_SYSTEM.md (config::global_dir = XDG_CONFIG_HOME/ktu ou ~/.config/ktu). Como o ktu-core e puro (M2), o dir do utilizador e INJECTADO: facade::system_prompt_in(fs, root, user_dir) + KtuBuilder::user_dir; a CLI resolve-o com env (config_cmd::global_dir_path) e passa-o tambem ao host de subagentes (o prompt tem de ser o mesmo). Novo verbo ktu prompt mostra o prompt efectivo (stdout = dado, --json com system/bytes/user_dir) - e a forma de diagnosticar porque uma regra nao chegou ao modelo. Testes: prompt.rs (5 unit), facade (directives_replace_and_append_the_system_prompt: so utilizador; depois projecto ganha), ktu-cli/tests/commands.rs::prompt_shows_the_effective_system_prompt. Doc: wiki/convencoes.md §2, wiki/integration/06-projecto.md §5, prime.rs.