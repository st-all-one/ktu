# Revisão do `opencode` — o que incorporar no `ktu`

> Análise do `REF/opencode` (monorepo Bun/TypeScript, v2.0.27) com o filtro do ktu:
> **simplicidade · kernel · usabilidade objectiva**. O opencode é o **maior** dos
> referenciais e o mais próximo de um produto; o ktu já adoptou dele o modelo de
> **subagentes read-only** (`plan`/`explorer`). Data: 2026-10-17.

---

## 0. Escala

| | `opencode` | `ktu` |
|---|---|---|
| Linguagem | TypeScript (Bun monorepo) | Rust |
| Linhas `src/` | **394 600** | 14 236 |
| Pacotes | ~37 | 7 crates |
| Escopo | coding agent completo: CLI + TUI + desktop + web + server + SDK + enterprise | kernel mínimo |

O opencode é ~28× o ktu. O valor está em **padrões de kernel e usabilidade**, não em copiar
a arquitectura (Effect-TS, SQLite, server multi-instância, desktop, GUI).

## 1. Onde olhar

- **`.opencode/`** (config declarativa do projecto): `agent/`, `command/`, `skills/`,
  `plugins/`, `themes/`, `tool/`, `glossary/`, `opencode.jsonc`.
- `packages/core` (67 k): `agent`, `permission`, `formatter`, `persistent-pty`, `git`,
  `bus`, `instruction-discovery`, `credential`/`oauth`, `database`, `models-dev`, `mcp`,
  `plugin`, `location`, `managed-policy`.
- `packages/{protocol,schema,server,client,sdk}`: a **porta + protocolo + SDK**.
- `packages/{tui,session-ui,theme,ui}`: TUI.
- `packages/{codemode,plugin,simulation,containers}`.
- `packages/ai` (31 k): providers.

---

## 2. Candidatos a incorporar

Veredicto: **incorporar** · **avaliar** · **não**.

### 2.1 Agentes declarativos (`.opencode/agent/*.md`) · **INCORPORAR**

**O que é:** agentes definidos como **dados** (Markdown + frontmatter): `mode`
(`primary`/`subagent`), `hidden`, `model`, `color`, `tools` (allow/deny por wildcard,
ex.: `{"*": false, "github-triage": true}`). O opencode traz **build** (acesso total) e
**plan** (read-only) embutidos, e um subagente **general** (`@general`).

**Porque cabe no ktu:**
- O ktu tem `AgentSpec` **em código** (primário + `explorer`); torná-lo **declarativo**
  é kernel (dado) e usabilidade (definir agentes próprios) — e já há precedente
  (guardrails declarativos em `.agents/guardrails/`).
- O modo plano read-only do ktu (I15) já é a ideia do agente `plan` do opencode.

**Onde entra:** `.agents/agents/*.md` com `mode`/`tools`/`model`; loader fail-closed.

**Esforço:** baixo-médio (schema + loader + integração com `AgentSpec`/guard). **Risco:** baixo.

### 2.2 AGENTS.md aninhados / descoberta de instruções · **INCORPORAR**

**O que é:** ficheiros `AGENTS.md` podem existir a **qualquer nível** do directório; ao ler
um ficheiro, os `AGENTS.md` dos directórios-pai são carregados no contexto. O comando
`/learn` extrai aprendizagens da sessão para o `AGENTS.md` **mais próximo** do código.

**Porque cabe no ktu:**
- O ktu tem **um** `AGENTS.md` composto no prompt. Instruções **por directório** são
  context discovery simples e de alto valor (monorepos/módulos).
- `/learn` fecha o ciclo "sessão → conhecimento" no sítio certo.

**Onde entra:** `prompt.rs`/`layout.rs` (descoberta de `AGENTS.md` por nível) + comando
`/learn` (ou `ktu learn`). **Esforço:** baixo-médio. **Risco:** prompt maior (orçamento).

### 2.3 Comandos com shell embutido (`.opencode/command/*.md`) · **INCORPORAR**

**O que é:** comandos `/nome` (frontmatter: `description`, `model`, `subtask`) cujo corpo
pode **expandir shell** (`` !`git diff` ``) — a saída do comando entra no prompt.

**Porque cabe no ktu:** é o **prompt template** (pi, backlog 41) **mais** expansão de shell
e/ou contexto dinâmico (`!`git diff``). Usabilidade objectiva (ex.: `/commit` vê o diff).

**Nota:** resolver a sobreposição **templates/receitas/comandos** num só mecanismo.
**Esforço:** baixo-médio. **Risco:** execução de shell a partir de um `.md` (exige
guardrail — passa pelo `Guard`).

### 2.4 Formatter (auto-formatação após edição) · **INCORPORAR**

**O que é:** após editar um ficheiro, corre o **formatter do projecto** (builtins por
linguagem/extensão).

**Porque cabe no ktu:** usabilidade objectiva e barata — o código editado sai formatado,
sem o modelo gastar passos. É **efeito**, logo atrás de uma porta/tool.

**Onde entra:** na thread de tools, após `write`/`edit`; configuração em `.agents/`.
**Esforço:** baixo. **Risco:** correr comandos do projecto (guardrail + timeout).

### 2.5 Permissões declarativas + memorizadas · **AVALIAR (já há seam)**

**O que é:** `Ruleset` declarativo `{action, resource, effect: allow|deny|ask}` +
`PermissionSaved` (uma aprovação **memorizada** por `action`/`resource`/projecto).

**Porque interessa:** o ktu tem guardrails declarativos (fail-closed) e `Approval`, mas
**não** memoriza aprovações. É a mesma ideia da página [permission-store](./permission-store.md)
(goose), agora com um schema concreto (`action`/`resource`/`effect` + `save`).

**Decisão:** consolidar em **uma** página (permission-store), referenciando o schema do
opencode. **Esforço:** baixo-médio. **Risco:** alargar privilégio (TTL/escopo).

### 2.6 PTY persistente · **AVALIAR**

**O que é:** um **shell persistente** por sessão: `cd`, variáveis de ambiente e estado
sobrevivem entre comandos.

**Porque interessa:** o ktu corre `bash` num processo por comando (cwd = root). Um PTY
persistente melhora usabilidade (navegar, activar venvs) — mas complica o confinamento.

**Esforço:** médio. **Risco:** escapar ao confinamento do `bash` (mitigação actual).

### 2.7 Referências externas (`references` na config) · **AVALIAR**

**O que é:** a config declara **referências** (repo/path + descrição) que o agente pode
consultar (ex.: `effect` → repo Effect-TS; `opencode-local` → path de dados).

**Porque interessa:** dar ao agente **ponteiros** para recursos externos (docs, código de
referência) é simples e útil — dados, não efeito.

**Esforço:** baixo. **Risco:** acesso a caminhos fora do root (guardrail).

### 2.8 Codemode · **AVALIAR (já há página)**

**O que é:** o opencode tem `packages/codemode` (10 k) — a mesma ideia do pi: código que
chama tools, com só a saída a chegar ao modelo. **Página já existe:** [codemode](./codemode.md).
Acrescentar que **dois** referenciais independentes o implementam (sinal de valor).

### 2.9 Simulation (testes determinísticos) · **AVALIAR**

**O que é:** um harness de **simulação** (backend/frontend, control-server, gravação) para
testar o agente de forma **determinística**, sem LLM real.

**Porque interessa:** o ktu preza determinismo (portas/fakes, provider falso). Uma
**simulação** end-to-end (guião → eventos esperados) é um upgrade dos testes actuais.
**Esforço:** médio-alto. **Risco:** infra de teste a manter.

### 2.10 Custom tools (`.opencode/tool/*.ts`) · **AVALIAR**

**O que é:** tools definidas pelo utilizador em ficheiros (`.opencode/tool/`).

**Porque interessa:** o ktu tem plugins (que transformam/negam) mas **não** tools
definidas pelo utilizador. Uma tool declarativa/script simples é usabilidade objectiva.
**Nota:** o ktu é Rust — a tool seria um **processo** (como os plugins), não TS.
**Esforço:** médio. **Risco:** superfície de execução (guardrail).

### 2.11 Já cobertos ou equivalentes

`skills` (ktu tem), `plugins` + hot reload (ktu tem), `themes` (backlog 46), `models-dev`
(backlog 24), `protocol/server/sdk` (ktu tem porta), `mcp` (backlog 12), `bus` de eventos
(ktu tem eventos estáveis), `database` (ktu usa JSONL — mais simples), `containers`
(backlog 15), `desktop` (backlog 27), `gui-extensions` (fora).

---

## 3. Não trazer (YAGNI / fora de escopo)

| Feature do opencode | Porquê não |
|---|---|
| Arquitectura **Effect-TS** | o ktu tem as suas portas/DI em Rust |
| SQLite/`effect-drizzle` para persistência | o ktu usa **log JSONL** (fonte da verdade) — mais simples |
| **Server multi-instância / location / worktree** | o ktu tem uma porta local por sessão |
| **Desktop app / GUI / web** | fora (backlog 27) |
| **Enterprise / managed-policy / identity / SSO** | fora de escopo |
| `packages/ai` (31 k, 15+ providers) | provider fino por dialeto (backlog 14) |
| `merman` (21 k, diagramas), `latex` | fora |
| `app` (90 k, TUI/web app), `ui`, `storybook` | fora |
| `enterprise`, `stats`, `console` | fora |
| **OAuth/credentials** complexos | API keys (backlog 31) |
| MCP | YAGNI §0 |

---

## 4. Recomendação

**Incorporar já (barato, alto valor):**
1. **Agentes declarativos** (§2.1) — `AgentSpec` como dado.
2. **AGENTS.md aninhados** (§2.2) — instruções por directório.
3. **Comandos com shell** (§2.3) — unificar com templates/receitas.
4. **Formatter** (§2.4) — código formatado após edição.

**Avaliar:**
5. **Permissões declarativas + memorizadas** (§2.5) — consolidar com `permission-store`.
6. **PTY persistente** (§2.6), **referências externas** (§2.7), **custom tools** (§2.10).
7. **Simulation** (§2.9) — upgrade de testes determinísticos.

**Manter fora:** Effect-TS, SQLite, server multi-instância, desktop/GUI/web, enterprise,
`ai`/`merman`/`latex`, OAuth complexo, MCP.

> **Nota transversal:** três referenciais (goose, pi, opencode) convergem em **dados
> declarativos** (agentes, comandos, permissões) e em **codemode**. O ktu já usa o padrão
> "dado + loader fail-closed" nos guardrails — estendê-lo a **agentes/comandos** é a
> evolução mais coerente e barata.
