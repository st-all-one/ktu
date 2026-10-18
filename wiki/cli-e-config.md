# CLI e configuração — desenho e revisão crítica

> Resposta ao pedido: **o CLI é o front primário** (não o TUI); o REPL `ktu session` é a
> experiência principal; a superfície segue o **knudge** (`kd`) instalado no host; a configuração
> divide-se em **global** (`~/.config/ktu/`) e **local** (projecto). Revisão com cautela, com a
> superfície real do `kd` como referência. Data: 2026-10-17.

---

## 0. A superfície do `kd`, medida (o modelo a seguir)

Verificado no host (`kd --help`, `kd ask --help`, `kd write --help`, `kd self --help`,
`kd config --help`, `kd prime`):

| Traço | No `kd` | Trazer para o `ktu`? |
|---|---|---|
| **Áreas por verbo** | `ask` (toda a pesquisa), `write` (toda a escrita), `task`, `map`, `doctor`, `drain`, `rewind`, `config`, `forget`, `sync`, `self`, `prime`, `init` | ✅ (com verbos do agente) |
| **Flags globais** | `--json`, `--log-level`, `--quiet` | ✅ (+ `--root`, `--session`, `--params`, `--batch`) |
| **`--params <JSON>`** | objeto com **tudo**; `-` lê stdin (D147) | ✅ |
| **`--batch <FONTE>`** | lote JSONL; `-` lê stdin (D147) | ✅ (com paralelo/sequencial) |
| **stdin/heredoc** | `-` lê stdin; vazio + pipe também | ✅ |
| **Listas repetíveis** | `--id a,b`, `--type x,y` | ✅ |
| **Envelope JSON** | `{success, command, data?, error{code,message,retryable}, warnings?}` | ✅ (já é o do ktu) |
| **Códigos de saída** | 2 invalid, 3 not_found, 4 conflict, 5 io, 6 timeout, 7 config, 8 schema, 70 internal | ✅ |
| **`prime`** | protocolo estático, byte-idêntico por versão | ✅ (`71`) |
| **`self`** | `setup`/`completions`/`upgrade`/`version`/`surface` | ✅ (`setup` adiado: A3) |
| **`config`** | `get`/`set`/`unset`/`list` (+`promote`) | ✅ (global+local) |
| **Rodapé de ajuda** | CICLO, ÂNCORAS, CORPO, "veja: `kd help <verbo>`" | ✅ |
| **Config em dois níveis** | `~/.config/local/knudge/config.toml` + `.knudge/config.toml` | ✅ (sem o `local/` extra) |

---

## 1. Revisão crítica das tuas decisões

### 1.1 "O CLI é mais importante que o TUI" — **✅ concordo, e é minimalista**
O TUI é **cliente de `ktu serve`** (processo extra, porta HTTP). O CLI é **directo** (sem servidor).
Priorizar o CLI **reduz** superfície (o TUI fica como extra). Alinha com M1 (mínimo) e com o facto
de o ktu ser uma ferramenta de terminal. **Consequência:** o REPL e a superfície do CLI são
**Eixo 0** do backlog; o TUI desce a "cogitável".

### 1.2 "REPL em `ktu session`" — **✅ com uma nota de semântica**
`session` é substantivo; usá-lo como REPL funciona, mas convém que a **área** `session` seja
coerente: `ktu session [id]` **abre** o REPL nessa sessão; `ktu session ls|rm|fork|resume` **gere**
sessões. Assim o verbo é uma **área** (abrir + gerir), não um caso especial.

### 1.3 "Efêmero de 1 turno como padrão" — **✅ mas é uma mudança de comportamento**
Hoje `ktu run` persiste em `--session default`. Tornar o **efémero** o padrão é uma **quebra** para
quem dependa do log. Mitigação: `ktu` (nu) e `ktu <prompt>` = efémero; `ktu run --session X` =
persistente; `--resume` implica persistente. Documentar no `prime`/`help`.

### 1.4 "Lock também à sessão" — **✅ necessário, com um detalhe**
Dois `ktu` no mesmo projecto (um REPL de sessão + um turno efémero) **não colidem** se tiverem ids
diferentes — o efémero não tem id, o REPL tem. O lock é **por id de sessão**
(`.ktu/sessions/<id>.lock`): impede dois processos no **mesmo** id. Recursos partilhados (memória)
são **append-only** (JSONL) → seguros por construção. O lock é do `64`.

### 1.5 Superfície agrupada, `prime`/`help`/`self` — **✅**
É o modelo certo e já existe no `kd`. O `prime` do ktu é o **protocolo do agente** (o que o modelo
lê) — **byte-idêntico por versão** (`71`), como o `kd prime`. O `self upgrade` reusa o `install.sh`
com checksum (G8, já feito). O `self surface` mede a **superfície pública** (subcomandos, flags e
tools) e é a medida do teto em [`surface.toml`](../surface.toml) — conferida por `make check`
(`xtask check-surface`); ver a [ADR 0011](./adr/0011-limite-de-400-linhas-e-firewall-mecanico.md).

### 1.6 `--params`/`--batch`/stdin/heredoc — **✅ com uma disciplina**
- **`--params <JSON>`** é o **contrato de máquina**; as flags são **açúcar** sobre ele. Precedência:
  **flag explícita > `--params` > config > default**. Schema versionado (`ktu self schema`).
- **`--batch`** recebe JSONL de `--params`; **paralelo por omissão** (N efémeros, concorrência
  limitada) e **`--sequential`** (uma sessão, N turnos). Resultados ordenados pelo índice de entrada
  (determinismo). **Não** é um subsistema novo: reusa o loop + um pool limitado.

### 1.7 Config global/local — **✅ com 4 correcções**
1. **Nomes:** o prefixo `global_` é redundante dentro de `~/.config/ktu/` → `skills/`,
   `guardrails/`, `plugins/`, `recipes/`.
2. **`auth.toml` nunca no projecto:** ✅ obrigatório. Reforços: modo `0600`, **nunca logado**,
   **nunca enviado ao modelo**; env é o *override* (precedência `flag > env > auth.toml`).
3. **`memory.toml` "interfere com o knudge":** risco de **dupla fonte de verdade**. Decisão: o ktu
   **não** reimplementa a config do knudge; `memory.toml` é a **vista do ktu** que **projeta** nas
   chaves do knudge (ou simplesmente **aponta** para `.knudge/config.toml`). Uma só verdade.
4. **Guardrails globais "sempre ativos":** isto **é** o controlo determinístico (M5) — o utilizador
   define um **piso** que o projecto **não pode remover**, só **adicionar**. Skills/plugins/recipes
   globais são *defaults* (o projecto sobrepõe por nome); guardrails globais são *piso* (aditivo).
   Coerente com "agir SOBRE o agente", mas convém **declarar** a assimetria.

### 1.8 Tensão com M6 ("project-bound") — **resolvida**
O global é **do utilizador**, não do agente: dá **segredos** (auth), **defaults** (skills/plugins) e
um **piso** (guardrails). O projecto **ganha** chave a chave; **nunca** há estado global que o
projecto não veja. M6 mantém-se: o **projecto** é a autoridade; o global é o ponto de partida.

---

## 2. Desenho da superfície do CLI

### 2.1 Áreas (agrupamento no `--help`)

```
CONVERSA      (default) · session · run
CONHECIMENTO  memory · guardrails · plugins
PROJECTO      init · config · doctor · mode
SISTEMA       self · providers · prime · help
PORTA         serve · tui
```

| Comando | O que faz |
|---|---|
| `ktu [<prompt>]` | **1 turno efémero** (default); prompt de args, `-` (stdin) ou heredoc |
| `ktu session [id]` | **REPL** na sessão `id` (default: `main`) |
| `ktu session ls\|rm\|fork\|resume` | gestão de sessões |
| `ktu run` | turno explícito (com `--session` para persistir) |
| `ktu memory` | `list\|ask\|write` (inspecção da memória) |
| `ktu guardrails` | `list\|check` |
| `ktu plugins` | `list\|reload` |
| `ktu init` | cria `.agents/` + `.ktu/` |
| `ktu config` | `get\|set\|unset\|list` (`--local`/`--global`) |
| `ktu doctor` | saúde (layout, memória, provider) |
| `ktu mode` | `plan\|build` |
| `ktu self` | `version\|upgrade\|completions\|surface` (`setup` adiado: A3) |
| `ktu providers` | diagnóstico do provider |
| `ktu prime` | protocolo do agente (byte-idêntico por versão) |
| `ktu help [verbo]` | ajuda por área |
| `ktu serve` | porta HTTP (para TUI/web) |
| `ktu tui` | cliente da porta (**secundário**) |

### 2.2 Flags globais

`--json` · `--log-level <N>` · `--quiet` · `--root <PATH>` · `--session <id>` · `--ephemeral` ·
`--params <JSON>` (`-`=stdin) · `--batch <FONTE>` (`-`=stdin) · `--sequential` · `--concurrency <N>`.

### 2.3 REPL (`ktu session`) — UX

- **Prompt** com id + modo (`plan`/`build`) + estado (`●` a pensar).
- **Streaming** dos eventos do turno (texto, tool calls, resultados).
- **Comandos `/`:** `/plan`, `/mode`, `/compact`, `/resume`, `/fork`, `/undo`, `/redo`, `/memory`,
  `/help`, `/quit` — **todos** são comandos da porta (nenhuma lógica no REPL).
- **Entrada multi-linha** (bloco: linha em branco ou `\` final), **histórico** persistido,
  **Ctrl-C** cancela o turno, **Ctrl-D** sai, **Tab** completa `/` e caminhos.
- **Cor automática** (desligada quando não é TTY → pipe-friendly).

### 2.4 `--params` e `--batch`

```jsonc
// ktu --params '{"prompt":"faz X","session":"s1","plan":true,"effort":"low"}'
// echo '{"prompt":"..."}' | ktu --params -
{ "prompt": "…", "session": "s1", "ephemeral": false,
  "provider": "opencode-go", "model": "…", "effort": "low",
  "max_steps": 24, "plan": false, "root": "…" }
```

- `--batch <f.jsonl>`: um `--params` por linha.
  - **paralelo (default):** N turnos efémeros, `--concurrency N` (default 4); saída JSONL ordenada
    pelo índice.
  - **`--sequential`:** uma sessão, turnos em ordem.

---

## 3. Desenho da configuração

### 3.1 Global — `$XDG_CONFIG_HOME/ktu/` (default `~/.config/ktu/`)

```
~/.config/ktu/
  auth.toml        # credenciais de providers/modelos — 0600, nunca no projeto, nunca logado
  config.toml      # defaults de comportamento (provider, orçamento, effort, max_steps)
  memory.toml      # vista do ktu sobre a config da memória (projeta no knudge; uma só verdade)
  skills/          # skills globais (sobrescritas por locais de mesmo nome)
  guardrails/      # guardrails globais (PISO: sempre ativos; locais só adicionam)
  plugins/         # plugins globais (aditivos)
  recipes/         # receitas globais (se implementadas — G1)
```

### 3.2 Local — `<project>/.agents/`

```
.agents/
  AGENTS.md        # router de instruções do projeto
  config.toml      # config do projeto (ganha chave a chave sobre a global)
  skills/          # sobrescrevem as globais de mesmo nome
  guardrails/      # adicionam aos globais (nunca removem)
  plugins/         # adicionam
  recipes/         # adicionam
  memory/          # dados da memória (notas.jsonl) — local
```

### 3.3 Precedência

| Chave | Ordem |
|---|---|
| Comportamento | `flag` > `--params` > `.agents/config.toml` > `~/.config/ktu/config.toml` > default |
| Credenciais | `flag` > `env` > `~/.config/ktu/auth.toml` > nenhuma |
| Skills/plugins/recipes | local (mesmo nome) **sobrepõe** global |
| Guardrails | global **+** local (**aditivo**; o global nunca é removível) |

### 3.4 Segurança

- `auth.toml` **0600**, fora do repo (global), **nunca** no log nem no contexto do modelo.
- Um guardrail/teste prova que um `auth.toml` **não** é replicável ao projecto nem ecoado.
- Guardrails globais são **piso**: um projecto que tente "desligar" um guardrail global vê a
  tentativa **ignorada** (como o `deny-explorer-write` de defesa em profundidade).

---

## 4. `ktu init` — preparar o ambiente do projecto

Modelado no `kd init`/`onboard` (idempotente, com marcadores geridos).

```
ktu init [--force] [--git-excluded | --git-tracked] [--no-prompt]
```

### 4.1 O que faz (idempotente)
1. **Cria o layout:** `.agents/` (`skills/`, `guardrails/`, `memory/`, `recipes/`) + `.ktu/`
   (`sessions/`, `plugins/`, `trash/`, `plan/`, `logs/`).
2. **Copia elementos:**
   - **skill do ktu** `.agents/skills/ktu/SKILL.md` (como a skill `kd` do knudge, D162) — ensina o
     protocolo ao agente;
   - **template de config** `.agents/config.toml` (só se ausente, ou `--force`);
   - **bloco gerido no `AGENTS.md`** (protocolo do ktu) com marcadores e version marker.
3. **Ajusta o git** (nunca `.gitignore`):
   - **`.git/info/exclude`** — linhas geridas, dependentes do modo (§4.2);
   - **`.gitattributes`** — bloco gerido (`# ktu:start`/`# ktu:end`): `eol=lf` nas notas/eventos,
     `merge=union` no log de eventos (append-only), `binary` no derivado.
4. **Emite o `prime`** (protocolo) — salvo `--no-prompt`.
5. **Relatório** por item (`novo`/`atualizado`/`inalterado`), como o `OnboardReport` do knudge.

### 4.2 Modos de persistência
| Flag | `.git/info/exclude` | Efeito |
|---|---|---|
| `--git-tracked` (default) | exclui só o **derivado/runtime** (`.ktu/`, `.agents/memory/.idx/`, `cache/`, `.locks/`) | conhecimento versionado |
| `--git-excluded` | exclui **tudo** (`/.agents/`, `/.ktu/`) | nada do ktu no git (local-only) |

Alternar entre modos **reverte** as linhas do modo anterior (sem duplicar) — tática do knudge.

### 4.3 Marcadores idempotentes (a tática do knudge)
- `AGENTS.md`: `<!-- ktu:start -->` / `<!-- ktu:end -->` + `<!-- ktu:version: N -->`.
- `.gitattributes`: `# ktu:start` / `# ktu:end`.
- `.git/info/exclude`: linhas identificadas por prefixo/comment gerido.

O `upsert` casa a **linha inteira** (após `trim`): reexecutar **não** duplica nem reescreve. O
`--force` só sobrescreve a **config**, nunca a memória/notas do utilizador.

### 4.4 Segurança
- Fora de um repositório git, as operações de git são **no-op** (não falha).
- O `--force` **não** apaga conhecimento; só reescreve ficheiros geridos.

---

## 5. Memória: embeddings locais **opcionais** (incentivados)

> **Implementado na H8** — contrato e resultados em [memoria-semantica](./memoria-semantica.md).

O ganho de recall com embeddings é expressivo (o knudge usa-o); no ktu é **opcional** e
**incentivado** — sem ele, o recall é lexical (funciona); com ele, melhora muito.

```
ktu memory drain [--status | --digest | --force]
ktu memory drain service [--install | --subscribe | --unsubscribe | --status | --uninstall | --reconcile | --dry-run]
    [--every <DUR>] [--port <N>] [--model <PATH>] [--no-deps] [--keep-model | --remove-model]
```

### 5.1 `service --install` (equivalente ao `kd drain service --install`)
- Instala o **agendador** (`systemd --user`/`launchd`) + o **servidor de embeddings** persistente
  (`llama.cpp` + GGUF) + **cadastra o projecto actual**.
- Baixa `llama.cpp` + o **mesmo modelo do knudge** (`google/embeddinggemma-2`, `embeddinggemma-2-Q8_0.gguf`)
  se ausentes (`--no-deps` para pular).
- Porta por omissão **8889**; período `--every 1h`.

### 5.2 Ciclo de vida (tática do knudge)
| Flag | Efeito |
|---|---|
| `--status` (default) | saúde: agendador, servidor, fila por projecto |
| `--subscribe`/`--unsubscribe` | cadastra/descadastra o projecto (multi-projecto) |
| `--reconcile` | alinha `memory.toml` (`endpoint`/`model`) ao worker instalado |
| `--uninstall` | remove o sistema; `--keep-model` (default) preserva o GGUF, `--remove-model` move-o para o **trash** (nunca `rm`) |
| `--dry-run` | só mostra o plano |

### 5.3 Princípios
- **Fila derivada:** o índice/embeddings são **derivados**; `--digest` drena em lotes; `--force`
  apaga `.idx/` e reindexa (último recurso).
- **Degradação graciosa:** sem worker, o canal semântico **desliga sem aviso**; o recall lexical
  continua.
- **Uma só verdade:** `memory.toml` projeta no knudge; o modelo/endpoint vêm do mesmo sítio.
- **Segurança:** `--remove-model` usa o `trash` do ktu; nunca `rm`.

---

## 6. Impacto no backlog

| Eixo 0 (caminhos) — agora **liderado pelo CLI** | Estado |
|---|---|
| [`cli-repl`](./backlog/cli-repl.md) — `ktu session`, UX | actualizado |
| [`cli-superficie`](./backlog/cli-superficie.md) — áreas, `prime`/`help`/`self`, `--params`/`--batch` | **nova** |
| [`modo-efemero`](./backlog/modo-efemero.md) — default efémero + lock | actualizado |
| [`config-global`](./backlog/config-global.md) — `~/.config/ktu/` (auth, skills, guardrails, …) | **nova** |
| [`ktu-init`](./backlog/ktu-init.md) — prepara o ambiente, git, marcadores idempotentes | **nova** |
| [`memory-drain-service`](./backlog/memory-drain-service.md) — embeddings locais opcionais (worker) | **nova** |
| [`crate-facade`](./backlog/crate-facade.md) — fachada de incorporação (uso #1) | **implementado** (H9) |
| [convenções](./convencoes.md) — directriz, `AGENTS.md` aninhados, comandos, agentes declarativos | **implementado** (H10) |
| [`receitas`](./backlog/receitas.md) — `settings`, `$1`…`$9`, *shell* guardado (`ktu run --command`) | **implementado** (H10) |
| [`exposicao-tools`](./backlog/exposicao-tools.md) — `deferred_tools` + `tool_search` | **implementado** (H10) |

Consolidações afectadas: **G9 config** (`76`) passa a incluir o **global/local**; **`100`
credenciais** → `auth.toml`. O **TUI** (`17`) desce a "cogitável".

---

## 7. Recomendação

1. **CLI é o Eixo 0.** Ordem: `cli-superficie` → `cli-repl` → `modo-efemero` → `config-global`.
2. **Adoptar o modelo do `kd`** (áreas, `--params`, `--batch`, `prime`/`self`, envelope, exit codes).
3. **Inverter o default** para efémero e **aplicar o lock** por sessão.
4. **Config em dois níveis**, com `auth.toml` global-only, guardrails globais como **piso** e
   `memory.toml` como **vista** (nunca dupla verdade).
5. **Travar por teste:** "dois caminhos, uma sessão" (M4); "auth não ecoa" (M3); "guardrail global
   não removível" (M5).

> **Nota crítica final.** A superfície do `kd` é excelente, mas é de uma **ferramenta de memória**;
> o ktu é um **agente**. Copiar o *estilo* (áreas, envelope, `--params`) sim; copiar os *verbos*
> (`ask`/`write`) não — no ktu esses são **tools do modelo**, não comandos do utilizador. A
> superfície do CLI é para **o humano controlar o agente**, não para o substituir.
