# H10 — Convenções

> **Issue:** `task_01tp3uj0` · **Estado:** **concluída** · **Âncora:**
> [consolidado](../backlog/consolidado.md) · **Desenho:**
> [revisao-proposito](../revisao-proposito.md) §5.1 · **Pré-requisitos:** H4, H2 ·
> **Contrato:** [convencoes](../convencoes.md).

## Objetivo
Incorporar as **convenções globais** (`.agents`, `AGENTS.md`) e a personalização **por projecto**
(uso #4): templates/comandos, nome de sessão, AGENTS.md aninhados, agentes declarativos, directriz.

## Passos
1. **Templates/receitas/comandos** (`41`/G1) — Markdown com frontmatter, invocáveis por `/`; o shell
   embutido é a variante rica (**gated** por guardrail). + **slash commands** (`20`).
2. **Nome automático de sessão** (`84`) — a partir do primeiro turno.
3. **`AGENTS.md` aninhados** (`52`) — descoberta por directório; + **agentes declarativos** (`51`) —
   `mode`/`model`/`tools` allow-deny.
4. **`SYSTEM.md`/`APPEND_SYSTEM`** (`97`) — directriz/persona do projecto; + **exposição de tools**
   (`93`) — `declared`/`deferred` + `tool_search`.

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_01k5thqh` | Templates/comandos + slash |
| `task_00n0u20s` | Nome automático de sessão |
| `task_01p2aal8` | `AGENTS.md` aninhados + agentes declarativos |
| `task_00n15wb8` | `SYSTEM.md`/`APPEND_SYSTEM` + exposição de tools |

## Gate / testes
- Template invocado por `/`; shell embutido passa por `Guard::evaluate`.
- `AGENTS.md` aninhado mais próximo **sobrepõe** (determinismo de descoberta).
- Agente declarativo restringe **tools** estruturalmente (não por prompt).
- `SYSTEM.md` é **project-bound**; `tool_search` carrega a tool no **próximo** pedido.

## Riscos
- Directriz virar prompt-engineering/DSL → **texto simples** (um ficheiro), sem lógica.
- `tool_search` duplicar o A4 rejeitado → é **distinto** (declared/deferred, não roteamento).

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **proptest:** parse de frontmatter de templates e de `AGENTS.md` aninhados com entradas
  arbitrárias; descoberta **determinística** (mais próximo sobrepõe).
- **N1:** catálogo de skills/templates em `BTreeMap` (ordem estável no prompt).
- **N2:** sem `unwrap` na descoberta; ficheiro malformado = ignorado (fail-open), nunca panic.
- **N8:** `mode`/`model`/`tools` (allow-deny) como **enums**, não bools.
- **Gate:** contribui para **G10/G11** (mais contexto, mesma qualidade).

## Estado da implementação

Tudo em `ktu-core` (puro, a ler pela porta `Fs`) + borda (`ktu-cli`):

| Entrega | Onde |
|---|---|
| `SYSTEM.md`/`APPEND_SYSTEM.md` (projecto > utilizador) | `prompt::{Directive, compose_layers}`, `facade::{system_prompt_in}`, `KtuBuilder::user_dir`, `layout::{SYSTEM_FILE, APPEND_SYSTEM_FILE}` |
| `AGENTS.md` aninhados (descoberta + tectos) | `agents_md::{discover, nearest, render}`, `layout::{AGENTS_DIR, AGENTS_FILE}` |
| Comandos do projecto + `/` | `command::{Command, parse, load, find}`, `layout::COMMANDS_DIR`, REPL, `ktu commands ls` |
| Agentes declarativos | `agent_file::{ToolPolicy, ToolAccess, AgentMode, AgentFile, load, apply}`, `layout::AGENTS_FILES_DIR`, `ktu agents ls` |
| Nome automático de sessão | `session::{title_from, title_of}`, `checkpoint.title`, `ktu session ls` |
| Exposição de tools `declared`/`deferred` + `tool_search` | `ToolExecutor::deferred`, `tool_search::{spec, rank, render}` (BM25), `events::TOOL_LOADED`, `Config::deferred_tools`, `Toolbox::with_deferred` |
| Instruções locais na leitura de ficheiro | porta `Instructions` (`ports::Local`, fake `MapInstructions`), `agents_md::label`, `events::CONTEXT_LOCAL`, adaptador `ktu-cli/src/instructions.rs` |
| Receitas: `settings`, `$1`…`$9` e *shell* guardado | `command::{CommandSettings, ShellAccess}`, `facade::evaluate_fact`, `ktu-cli/src/command_run.rs`, `ktu run --command` |
| Frontmatter partilhado | `frontmatter::{split, parse, body, unquote, required}` (guardrails passam a usá-lo) |
| Nomes de agentes no catálogo | `SubagentHost::agents`, `subagent::spec_with`, `AgentId::Named`, `AgentRegistry::{with_named, with_primary, subagent_names}` |

**Decisões de desenho.** (1) A directriz do utilizador é **injectada** (`KtuBuilder::user_dir`) — o `ktu-core` é puro e não lê `$HOME` (a CLI resolve `config::global_dir`). (2) O título da sessão
 vive no **checkpoint** (índice derivado), não no log — o log continua a ser a única verdade (I3).
(3) A descoberta de `AGENTS.md` tem tectos de profundidade/bytes/directórios: sem eles, um monorepo
com um `REF/` grande pagaria o custo em cada arranque. (4) Os agentes declarativos **reutilizam** o
enforcement existente (`AgentSpec` → `ScopedTools` + `StructuralGuard`): não há um segundo caminho
 de decisão. (5) `AgentId` passou a ter `Named(Box<str>)` (deixou de ser `Copy`) para que o agente
 apareça correctamente em `guard.decision` e em `applies_to`.

**Adiados primeiro, implementados de seguida.** A primeira passagem fechou a H10 com A9 (receitas
com `settings`/*shell*), A10 (`declared`/`deferred` + `tool_search`) e A11 (`AGENTS.md` na leitura)
em [ADIADOS](./ADIADOS.md); o dono do projecto pediu-os **antes** da H11 e os três foram
implementados (ver o quadro acima). O que continua fora, com razão registada:

- **`sub_recipes`** (receitas que invocam receitas) — sem consumidor; a composição faz-se com a
  tool `subagent` ou com dois comandos.
- **`model`/`hidden`/`color`** no frontmatter dos agentes — o provider é por sessão; a selecção de
  modelo por agente exige plumbing que ninguém pediu.
- **`parameters` declarados** (nome/descrição/omissão por parâmetro) — `$1`…`$9` + `$ARGUMENTS`
  chegam; um schema de parâmetros só compensa com validação por tipo.

## Prova (G10/G11)

```
$ cargo test -p ktu-core --lib          # 175 testes (prompt/agents_md/command/agent_file/title/tool_search)
$ cargo test -p ktu-core --test loop    # inclui tool_search e instruções locais no loop
$ cargo test -p ktu-cli --test commands --test agents --test session
$ cargo test -p ktu-cli --bin ktu prime # o protocolo documenta as convenções
$ make check                            # fmt + clippy -D warnings + test + layers + file-length
```

| Critério do gate H10 | Prova |
|---|---|
| Template invocado por `/` | `a_project_command_runs_as_a_turn_with_rendered_arguments` (o log contém o prompt renderizado) |
| `AGENTS.md` aninhado mais próximo sobrepõe | `discovers_root_first_then_nested_by_path` + `nearest_returns_the_closest_document_up_to_the_root` |
| Agente declarativo restringe tools estruturalmente | `a_declarative_primary_restricts_tools_structurally` (o guard nega `read_file` com `negado: capability`) |
| `SYSTEM.md` é project-bound | `directives_replace_and_append_the_system_prompt` (projecto ganha ao utilizador) |
| Determinismo da descoberta | `load_is_fail_closed_and_deterministic`, `stops_at_the_depth_limit` |
| `tool_search` carrega no **próximo** pedido | `tool_search_promotes_a_deferred_tool_to_the_next_request` (`tool_counts == [2, 3, 3]`) |
| Tool diferida sai do catálogo mas executa | `deferred_tools_leave_the_catalog_but_stay_executable` |
| Instruções locais: o mais próximo ganha e não se repete | `local_instructions_are_attached_once_per_source` |
| *Shell* embutido passa por `Guard::evaluate` | `run_command_applies_settings_and_guarded_shell` (com `bash` `never` o comando **não** corre) |
| Gate | `make check` verde |
