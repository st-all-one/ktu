# Revisão do `pi` — o que incorporar no `ktu`

> Análise do `REF/pi` (monorepo TypeScript, v1.0.x) com o filtro do ktu:
> **simplicidade · kernel · usabilidade objectiva**. O pi é o **análogo mais próximo**
> do ktu (um *harness* mínimo e extensível) — logo, a comparação é a mais directa.
> Data: 2026-10-17.

---

## 0. Escala e tese

| | `pi` | `ktu` |
|---|---|---|
| Linguagem | TypeScript (npm monorepo) | Rust |
| Linhas `src/` | **172 864** | 14 236 |
| Ficheiros de teste | 634 | — (161 testes Rust) |
| Pacotes | 11 (`agent`, `ai`, `chord`, `codemode`, `coding-agent`, `durable`, `env`, `evals`, `mcp`, `telemetry`, `tui`) | 7 crates |
| Tese | *minimal, extensible agent harness* — "adapta o Pi ao teu workflow" | *kernel mínimo que alcança o quality_test* |

O pi **diz explicitamente** que "skips features like **sub-agents and plan mode**" — que o
**ktu tem**. E o ktu não tem várias coisas que o pi tem (árvore de sessão, templates,
codemode). A troca é **nos dois sentidos**: vale colher ideias de kernel e usabilidade.

## 1. Onde olhar (estrutura do pi)

- `packages/coding-agent` (76 k): CLI, sessões, extensões, skills, templates, temas, RPC, SDK.
- `packages/agent` (2,5 k): runtime do agente com tool calling e estado.
- `packages/ai` (27 k): API multi-provider unificada + catálogo de modelos.
- `packages/chord` (11 k): **host de plugins** com contratos, estado replicado e RPC.
- `packages/codemode` (3 k): execução de **JavaScript em sandbox** que só chama tools.
- `packages/durable` (25 k): runtime durável de conversa/tarefa/documento.
- `packages/{tui,telemetry,mcp,env,evals}`.
- Docs: `how-pi-works`, `session-format`, `compaction`, `packages`, `prompt-templates`,
  `slash-commands`, `virtual-models`, `extensions`, `skills`, `themes`, `rpc`, `sdk`.

---

## 2. Candidatos a incorporar

Veredicto: **incorporar** · **avaliar** · **não**.

### 2.1 Árvore de sessão (branching / fork / clone) · **AVALIAR (alto valor)**

**O que é:** o log da sessão é uma **árvore** (`id`/`parentId`), não uma linha; cada
caminho é um *branch*. `/tree` (mover dentro da sessão), `/fork` (nova sessão a partir de
uma mensagem anterior), `/clone` (copiar o branch activo). Ao sair de um branch, o pi pode
**resumir** o branch abandonado e anexar o resumo ao novo.

**Porque interessa ao ktu:**
- É a **diferença mais estrutural**: o log do ktu é **linear**; o katu já tinha deferido
  isto ("S2 árvore de sessão"). É **kernel** (o log é a fonte da verdade) e usabilidade
  objectiva (explorar alternativas sem perder trabalho).
- Casa com a compactação aditiva (o resumo do branch é o mesmo mecanismo).

**Custo:** médio-alto (eventos com `parent_id`, projecção por branch, comandos `/tree`
`/fork` `/clone`, migração de logs lineares). **Risco:** tocar no log (fonte da verdade) —
exige invariantes I1–I3 preservados e testes de ouro.

### 2.2 Prompt templates (Markdown → `/` comando) · **INCORPORAR**

**O que é:** ficheiros Markdown em `prompts/` tornam-se **comandos `/nome`** reutilizáveis,
com `description`, `argument-hint` e substituição (`$1`, `$@`, `${1:-default}`).

**Porque interessa:**
- **Simplicidade máxima:** um `.md` + frontmatter; **zero** código de execução.
- **Usabilidade objectiva:** repetir prompts sem os redigitar.
- **Kernel:** é dado; o loader é fail-open (como as skills do ktu).

**Nota:** sobrepõe-se às **receitas** (do goose, backlog 19). Decidir: um mecanismo só
("templates") ou dois (templates leves + receitas com parâmetros/settings). O ktu já tem
`.agents/skills/` — os templates são o primo "sem instruções de apoio".

**Custo:** baixo. **Risco YAGNI:** baixo.

### 2.3 Steering / follow-up messages · **AVALIAR**

**O que é:** o utilizador pode enviar mensagens **durante** um turno (*steering*, entra
após o passo do assistente) ou **após** o trabalho pendente (*follow-up*); abortar devolve
as mensagens enfileiradas ao editor.

**Porque interessa:** o ktu tem `cancel`/`continue` fora-de-banda na porta, mas não
**enfileirar** mensagens. É usabilidade objectiva (corrigir o rumo sem matar o turno).

**Custo:** médio (fila na porta + pontos de injecção no loop). **Risco:** introduzir
não-determinismo no turno — exige ordem estável e log.

### 2.4 Virtual models (roteamento) · **AVALIAR**

**O que é:** um modelo **selecionável** que escolhe um modelo **físico** por pedido
(por tarefa/custo/estado). O pi mantém **selecção** e **dispatch** separados: o que se
seleciona fica em `model_change`; o que responde fica na mensagem do assistente.

**Porque interessa:** o ktu tem um modelo por sessão. Um "virtual model" é **kernel-
friendly** (dado + roteador puro) e casa com o `--model`/`--effort`. É o primo do
"modelo por fase" (backlog 8) mas **mais simples** (sem máquina de fases).

**Custo:** médio (registo + roteador + log da selecção vs dispatch). **Risco:** roteamento
não-determinístico se o roteador não for puro.

### 2.5 Pi packages (bundlar e distribuir recursos) · **AVALIAR**

**O que é:** um pacote (npm/git/local) que agrega **extensões + skills + templates +
temas**, com descoberta convencional (`extensions/`, `skills/`, `prompts/`, `themes/`) ou
manifesto explícito.

**Porque interessa:** o ktu já tem `.agents/{skills,guardrails}`; um conceito de **pacote**
dá distribuição/reutilização. **Custo:** médio (manifesto + instalador + trust). **Nota:**
o pi usa npm; o ktu precisaria de git/tarball.

### 2.6 Codemode (compor tools com código em sandbox) · **AVALIAR (alto valor, alto custo)**

**O que é:** o modelo escreve **JavaScript** (QuickJS, sem APIs de SO) que chama as outras
tools; **só a saída** chega ao modelo — logo pode correr chamadas **em paralelo** e
**filtrar** resultados grandes antes de os mostrar.

**Porque interessa:** ataca directamente o "performance terciária" (menos *round-trips*,
menos tokens) e a composição de tools. O `plan.md` §0 lista WASM como YAGNI, mas o
*codemode* é uma **alternativa mais simples** a paralelismo de tools (backlog 11).

**Custo:** alto (runtime JS + sandbox + catálogo de tools tipado + limites). **Risco:**
superfície de execução nova; determinismo. Só com número.

### 2.7 Compactação parametrizada + resumo de branch · **AVALIAR**

**O que é:** gatilho `contextTokens > contextWindow - reserveTokens` (reserve 16 k),
retenção `keepRecentTokens` (20 k), **resumo iterativo** (o resumo anterior entra no
próximo) e **resumo de branch** ao mudar de caminho.

**Porque interessa:** enriquece a compactação do ktu (relevante para **Q4**). O
`reserveTokens`/`keepRecentTokens` são parâmetros simples e explícitos.

**Custo:** baixo-médio (parâmetros + resumo iterativo). **Risco:** mexer no caminho do
gate — medir.

### 2.8 Temas (TUI) · **AVALIAR (baixo)**

**O que é:** temas JSON para as cores do terminal.

**Porque interessa:** pouco — a TUI do ktu é mínima. Baixo esforço, baixo valor.

### 2.9 Project trust · **AVALIAR**

**O que é:** o pi resolve a **confiança no projecto** antes de carregar settings/recursos
do projecto (extensões/skills/templates podem executar código ou instruir o modelo).

**Porque interessa:** segurança — o ktu carrega `.agents/` sem confiança explícita. Um
modelo de trust (fail-closed) é simples e defensivo. **Custo:** baixo-médio.

### 2.10 Sessões versionadas + migração · **AVALIAR**

**O que é:** o formato de sessão tem `version` (v1 linear → v2 árvore → v3), com
**migração automática** ao carregar.

**Porque interessa:** o ktu tem eventos versionados implicitamente; tornar a **versão** e a
**migração** explícitas é barato e protege o formato. **Custo:** baixo.

### 2.11 Chord (contratos + estado replicado de plugins) · **AVALIAR**

**O que é:** um **host de plugins tipado** com contratos, estado replicado e RPC sobre
*links* JSON.

**Porque interessa:** o ktu tem plugins por processo + hot reload por geração; o *chord*
acrescenta **contratos** (verificação de interface) e **estado replicado**. Mais avançado
do que o ktu precisa hoje. **Custo:** alto.

### 2.12 Já cobertos ou equivalentes

`skills` (ktu tem), `compaction aditiva` (ktu tem), `JSON/print/RPC` (ktu tem porta +
stdout=dados), `slash-commands` (backlog 20), `export` (backlog 25), `custom providers`
(backlog 14), `containerization` (backlog 15), `model catalog` (backlog 24).

---

## 3. Não trazer (YAGNI / fora de escopo)

| Feature do pi | Porquê não |
|---|---|
| `packages/ai` (27 k, 15+ providers) | o ktu tem llama.cpp + provider fino por dialeto |
| `packages/durable` (25 k) | o ktu tem log + checkpoint; subsistema próprio |
| `packages/mcp` | YAGNI §0 |
| `packages/env` (execução remota via SSH) | fora (o ktu corre local) |
| `packages/tui` (19 k, differential rendering) | a TUI do ktu é mínima (backlog 17) |
| `packages/telemetry` | o ktu tem `Metrics` local (backlog 33) |
| `packages/evals` | o ktu tem `quality_test` |
| Extensões TypeScript in-process | o ktu tem plugins por processo (mais isolados) |
| npm/git como único transporte | o ktu usa git/tarball |
| Share/bug-report na nuvem | fora |
| Shell aliases / terminal setup | conveniência de shell, não kernel |

---

## 4. Recomendação

**Incorporar já (barato, alto valor):**
1. **Prompt templates** (§2.2) — decidir a relação com as receitas do goose.

**Avaliar com medição/decisão:**
2. **Árvore de sessão** (§2.1) — a ideia mais forte; toca o log, exige cuidado.
3. **Steering/follow-up** (§2.3), **virtual models** (§2.4), **packages** (§2.5).
4. **Codemode** (§2.6) — só com número que o exija.
5. **Compactação parametrizada** (§2.7), **project trust** (§2.9), **sessões versionadas**
   (§2.10) — baratos, defensivos.

**Manter fora:** `ai`/`durable`/`mcp`/`env`/`tui`/`telemetry`/`evals`, extensões TS
in-process, npm como transporte, share na nuvem.

> **Nota:** o pi e o ktu são o mesmo *arquétipo* (harness mínimo). O ktu tem o que o pi
> deliberadamente não tem (**subagentes, modo plano, guardrails fail-closed**); o pi tem o
> que o ktu ainda não tem (**árvore de sessão, templates, codemode**). A troca é simétrica.
