# Convenções do projecto (`.agents/`) — H10

> **Âncora:** [consolidado](./backlog/consolidado.md) · **Desenho:**
> [revisao-proposito](./revisao-proposito.md) §5.1 · **Issue:** H10 (`task_01tp3uj0`) ·
> **Implementação:** [H10-convencoes](./implementation_phase2/H10-convencoes.md).
>
> Este documento é o **contrato** da personalização por projecto (uso #4): tudo o que o
> projecto declara em `.agents/` e que entra no prompt ou no registo de agentes — **dado**,
> nunca código. Nada aqui executa efeitos: o que altera o mundo continua a passar pelos
> guardrails (I10).

## 1. Mapa do `.agents/`

| Caminho | O quê | Consumido por |
|---|---|---|
| `AGENTS.md` | instruções do projecto (raiz), escritas pelo humano | system prompt |
| `**/AGENTS.md` | instruções **locais** (por directório) | system prompt |
| `.agents/SYSTEM.md` | **substitui** a base do agente | system prompt |
| `.agents/APPEND_SYSTEM.md` | **acrescenta** ao fim do prompt | system prompt |
| `.agents/commands/*.md` | comandos do projecto (`/nome`) | REPL (`ktu session`) |
| `.agents/agents/*.md` | agentes declarativos (tools allow/deny) | registo de agentes |
| `.agents/guardrails/*.md` | regras declarativas (só apertam) | `DeclarativeGuard` (H6) |
| `.agents/permissions.toml` | permissões por tool | `PermissionGuard` (H6) |
| `.agents/config.toml` | configuração do projecto (H2), incl. `deferred_tools` | CLI/fachada |
| `.agents/memory/` | memória durável (H7/H8) | tools `memory_*` |

`.ktu/` é o **runtime** (sessões, planos, trash, plugins) — produzido pelo agente, não revisto à
mão. A separação é a mesma desde a Fase 1: `.agents/` versiona-se, `.ktu/` não.

## 2. Directriz: `SYSTEM.md` e `APPEND_SYSTEM.md`

Dois ficheiros **opcionais**, em dois níveis:

| Nível | `SYSTEM.md` | `APPEND_SYSTEM.md` |
|---|---|---|
| Projecto | `.agents/SYSTEM.md` | `.agents/APPEND_SYSTEM.md` |
| Utilizador | `$XDG_CONFIG_HOME/ktu/SYSTEM.md` | `…/APPEND_SYSTEM.md` |

- **`SYSTEM.md` substitui** a base do agente; **`APPEND_SYSTEM.md` acrescenta** (nunca substitui).
- **Precedência:** o **projecto ganha** ao utilizador, ficheiro a ficheiro — homónimos **não** se
  combinam (`SYSTEM.md` de projecto + `APPEND_SYSTEM.md` de utilizador é uma combinação válida).
- **Fail-open:** ausente, vazio ou ilegível ⇒ nada (nunca erro).
- **Ordem no prompt:** directriz → `AGENTS.md` (raiz + aninhados) → acrescento.
- É **texto simples**: um ficheiro, sem DSL nem lógica (o risco de virar *prompt engineering* é
  explicitamente rejeitado no plano).

```console
$ cat .agents/SYSTEM.md
És o assistente deste monorepo: responde em português de Portugal e cita sempre o ficheiro.

$ ktu prompt            # o prompt efectivo (é o mesmo texto que o modelo recebe)
És o assistente deste monorepo: responde em português de Portugal e cita sempre o ficheiro.
…
```

## 3. `AGENTS.md` aninhados

O `AGENTS.md` da raiz dá as instruções do projecto; um `AGENTS.md` mais fundo dá instruções **locais** (módulos,
subprojectos). A descoberta é **determinística** e **limitada**:

- raiz primeiro; depois os aninhados por ordem de caminho (o mais fundo vem **mais tarde** no
  prompt — o mais próximo "sobrepõe" na leitura do modelo);
- cada documento aninhado entra com o cabeçalho `## <caminho relativo>` (o modelo sabe de onde vem);
- **tectos:** profundidade 8, 8 KiB por ficheiro, 32 KiB no total, 512 directórios visitados;
- **saltados:** directórios ocultos (`.git`, `.ktu`, `.agents`, …) e `target`, `node_modules`,
  `vendor`, `third_party`, `dist`, `build`, `out`, `REF`, `__pycache__`, `Pods`;
- fail-open: um ficheiro ilegível é ignorado.

Para o caso "o mais próximo ganha" existe a resolução por ficheiro
(`ktu_core::agents_md::nearest`): para `a/b/c.rs` procura-se `a/b/AGENTS.md`, depois `a/AGENTS.md`,
depois o da raiz. A injecção **por leitura de ficheiro** (contexto a pedido) fica adiada — hoje o
prompt é composto no arranque da sessão (ver [ADIADOS](./implementation_phase2/ADIADOS.md)).

## 4. Comandos do projecto (`.agents/commands/*.md`)

Um comando é um **template Markdown** invocável por `/nome args` na sessão interactiva:

```markdown
---
name: revisao
description: Revê o diff com atenção aos invariantes
---

Revê $ARGUMENTS e responde com uma lista de problemas concretos.
```

- frontmatter: `name` (por omissão, o nome do ficheiro) e `description` (para `/help`);
- o **corpo é o prompt**; `$ARGUMENTS` é substituído pelos argumentos da linha (sem marcador, os
  argumentos são acrescentados no fim);
- `ktu commands ls` lista os comandos (`--json` para máquina);
- **fail-open:** um ficheiro malformado é ignorado com aviso (`ktu: comando ignorado: …`), nunca
  impede a sessão;
- **sem shell embutido:** o template é texto puro — se o comando precisar de correr algo, o modelo
  usa a tool `bash`, que passa pelos guardrails (adiado: ver ADIADOS).

Um comando **não** tem lógica própria: renderiza o prompt e entrega-o ao turno normal (M4).

## 5. Agentes declarativos (`.agents/agents/*.md`)

Um agente é **dado** — o conjunto de tools é o *enforcement* estrutural (não um pedido no prompt):

```markdown
---
name: revisor
mode: subagent
tools: {"*": false, "read_*": true}
---
```

| Campo | Obrigatório | Valores |
|---|---|---|
| `name` | sim | nome estável (`ktu`/`explorer` são reservados) |
| `mode` | não | `subagent` (por omissão) · `primary` |
| `tools` | sim | `{"*": false, "read_*": true}` ou lista `read_file, glob` |

**Política de tools** (mínimo privilégio): a chave `*` é o *default*; um padrão **exacto** ganha a
qualquer padrão com `*`; entre padrões com `*`, ganha o **mais longo**; sem regra, vale o *default*.

- `mode: subagent` → registado por nome e usável pela tool `subagent` (profundidade 0: não cria
  netos);
- `mode: primary` → **substitui** o agente primário da sessão (um só por projecto; dois ⇒ erro);
- **fail-closed:** ficheiro inválido, nome reservado, `mode` desconhecido, tool desconhecida
  (regra exacta fora do catálogo) ou agente sem tools **impedem o arranque**, nomeando o ficheiro;
- a tool `subagent` **enumera os nomes disponíveis** na sua descrição (`explorer`, …);
- `ktu agents ls` mostra nome, modo e as tools **efectivas**;
- o **modo plano** continua read-only estrutural (I15): um agente declarativo nunca dá escrita ao
  plano.

## 6. Tools diferidas (`deferred_tools`) e `tool_search`

Um catálogo grande gasta contexto e confunde a escolha do modelo. A exposição é **dado**:

```toml
# .agents/config.toml — tools fora do catálogo até serem pedidas
deferred_tools = ["glob", "grep"]
```

- a tool sai do **catálogo** (não vai ao modelo) mas continua no **conjunto do agente** (o guard
  conhece-a: exposição ≠ permissão);
- quando existem diferidas, o loop oferece `tool_search` (BM25 sobre nome/descrição, determinístico);
- o resultado devolve as melhores (≤ 8) com o respectivo schema e **promove-as para o pedido
  seguinte** — registado no log como `tool.loaded` (nunca em silêncio);
- sem diferidas, `tool_search` **não** é oferecida (nada de superfície morta);
- `ktu config set deferred_tools glob,grep` (aceita lista separada por vírgulas ou o array TOML).

## 7. Instruções locais na leitura de um ficheiro

Ao ler um ficheiro (`read_file`/`edit_file`), o `AGENTS.md` **mais próximo** (do directório do
ficheiro ou de um pai, até à raiz) é anexado ao resultado:

```
…conteúdo do ficheiro…

[instrucoes locais: sub/AGENTS.md]
regra local do modulo
```

- uma vez por turno e por fonte (não repete o mesmo `AGENTS.md` na mesma sessão de trabalho);
- o evento `context.local` vai pela porta de log (o texto está no log — *model-visible ⟺ logged*);
- o mais próximo ganha: em `a/b/x.rs` vale `a/b/AGENTS.md`; se não existir, o de `a/`; depois o da raiz;
- é uma **porta** (`Instructions`): quem embute pode desligá-la (não passar nada) ou dar outra fonte.

## 8. Título de sessão

O título é derivado do **primeiro** pedido — heurística determinística, sem LLM: ≤ 8 palavras e
≤ 60 caracteres, sem *tags* XML, cercas de código nem pontuação de extremidade. É guardado no
*checkpoint* (`.ktu/sessions/<id>.checkpoint.json`, um índice **derivado** — o log continua a ser a
verdade) e mostrado por `ktu session ls` (última coluna; `--json` em `title`). Sem pedido utilizável,
o `id` é o *fallback*.

## 9. Receitas: `settings`, argumentos posicionais e *shell* guardado

Um comando pode impor configuração ao turno e correr *shell* — sempre **explícito** e guardado:

```markdown
---
name: revisao
description: Revê o diff
provider: llama
model: qwen3-coder
max_steps: 12
shell: true
---

Revê $1 e $2 com cuidado.
Estado do repositório:
!git status --short
```

- `provider`/`base_url`/`model`/`max_steps` são as **settings** do comando; precedência
  **flag > comando > `--params` > `config`**; correm por `ktu run --command <nome> [args]`
  (no REPL as settings são ignoradas, com aviso — o REPL não troca de provider a meio);
- `$1`…`$9` (posicionais, com aspas) e `$ARGUMENTS` (tudo); sem marcadores, os argumentos vão no fim;
- **`shell: true` é opt-in**: sem ele, `!cmd` é texto normal (nada corre por surpresa). Com ele,
  cada segmento passa pelo **encadeamento da fachada** (`StructuralGuard` → guardrails →
  permissões, via `facade::evaluate_fact`) antes de correr; negado ou a exigir aprovação ⇒ o
  comando **não** corre (exit 1) com remédio accionável;
- a execução usa a **mesma** `BashTool` do turno (não há segundo motor de shell) e o resultado
  entra no prompt com a marca de conteúdo não-confiável (I13).

## 10. O que continua fora

| Item | Porque |
|---|---|
| `sub_recipes` (receitas que invocam receitas) | sem consumidor: compõe-se com a tool `subagent` ou com dois comandos |
| `parameters` declarados (nome/omissão/tipo) | `$1`…`$9` + `$ARGUMENTS` chegam; um schema só compensa com validação por tipo |
| `model`/`hidden`/`color` nos agentes declarativos | o provider é por sessão; falta plumbing que ninguém pediu |

Condições de reabertura em [ADIADOS](./implementation_phase2/ADIADOS.md).

## 11. Verificação

```
$ cargo test -p ktu-core --lib prompt agents_md command agent_file frontmatter session::title tool_search
$ cargo test -p ktu-core --test loop    # tool_search + instruções locais no loop
$ cargo test -p ktu-cli --test commands --test agents --test session
```

| Critério (gate H10) | Prova |
|---|---|
| Directriz de projecto ganha à do utilizador | `directives_replace_and_append_the_system_prompt` |
| `AGENTS.md` aninhado entra com o caminho | `discovers_root_first_then_nested_by_path`, `renders_the_root_without_a_heading_and_the_rest_with_one` |
| Comando invocado por `/` | `a_project_command_runs_as_a_turn_with_rendered_arguments` |
| Agente declarativo restringe tools **estruturalmente** | `a_declarative_primary_restricts_tools_structurally` (`negado: capability`), `apply_replaces_the_primary_and_registers_subagents` |
| `mode`/`tools` como **enums** (N8) | `AgentMode`, `ToolAccess` (sem bools na API) |
| Descoberta determinística | `load_is_fail_closed_and_deterministic`, `stops_at_the_depth_limit` |
| Fail-closed nos agentes | `apply_rejects_unknown_tools_two_primaries_and_empty_agents` |
| `ktu prompt` mostra o prompt efectivo (projecto ganha) | `prompt_shows_the_effective_system_prompt` |
| `tool_search` carrega a tool no próximo pedido | `tool_search_promotes_a_deferred_tool_to_the_next_request` |
| Instruções locais (mais próximo, sem repetir) | `local_instructions_are_attached_once_per_source`, `reading_a_file_attaches_the_nearest_agents_md` |
| *Shell* embutido passa pelos guardrails | `run_command_applies_settings_and_guarded_shell` |
| Gate | `make check` verde |
