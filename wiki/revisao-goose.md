# Revisão do `goose` — o que incorporar no `ktu`

> Análise das features do `REF/goose` (v1.54.0) com o filtro do ktu:
> **simplicidade · kernel · usabilidade objectiva**. Não é para copiar o goose (é
> general-purpose); é para **colher ideias** que caibam num kernel mínimo.
> Data: 2026-10-17.

---

## 0. Escala (para calibrar)

| | `goose` | `ktu` |
|---|---|---|
| Linhas `src/` | **275 757** | 14 236 |
| Testes | 3 660 | 161 |
| Pacotes (`Cargo.lock`) | 1 319 | 345 |
| Crates | 15 | 7 |
| Escopo | general-purpose (código, research, escrita, automação, dados) | kernel mínimo focado em código |

O goose é o **oposto** do ktu em escopo: desktop + CLI + API + ACP, 15+ providers,
70+ extensões via **MCP**, inferência local (candle), agentes em roaming, voz, OTel.
Nada disso cabe no ktu. O valor está em **algumas ideias** de kernel e usabilidade.

## 1. Onde olhar (estrutura do goose)

- `crates/goose-agent` (1,7 k linhas): **máquina de estados** do agente com `Operation`/
  `Inference`/`EffectHandler` — limpa, mas o ktu já tem o seu loop; adoptar seria reescrever.
- `crates/goose-context-management` (1,2 k): **compactação** com resumo **estruturado**.
- `crates/goose/src/recipe` (YAML/JSON): **receitas** reutilizáveis.
- `crates/goose/src/slash_commands`: **atalhos** interactivos.
- `crates/goose/src/permission`: modos + **store** de aprovações memorizadas.
- `crates/goose/src/{session,token_counter,model_catalog,context_limit,tool_inspection,skills,hints}`.
- `crates/goose-mcp` (6 k): MCP — **fora** (YAGNI).

---

## 2. Candidatos a incorporar

Filtro aplicado: **entra** se for simples, respeitar o kernel (dado, não efeito) e
servir **usabilidade objectiva**. Veredicto: **incorporar** · **avaliar** · **não**.

### 2.1 `recipe` — receitas reutilizáveis · **INCORPORAR**

**O que é:** uma **definição declarativa e versionada** de uma tarefa: `title`,
`description`, `instructions`, `prompt`, `parameters`, `settings` (provider/modelo/
temperatura/max_turns), `sub_recipes`. Ficheiros `*.yaml`/`*.json`; lançadas por
`/recipe` ou CLI.

**Porque cabe no ktu:**
- **Kernel:** é **dado** (como os guardrails declarativos), não efeito — o loader
  (fail-closed) + o comando `ktu run --recipe` não tocam no loop.
- **Simplicidade:** um schema + um loader + um parâmetro; zero dependências novas
  (`serde`/`toml` já existem; YAML exigiria `serde_yaml`, ou usar TOML).
- **Usabilidade objectiva:** repetir/reproduzir workflows provou valor (é o que a
  bateria `quality_test` já faz à mão). Casa com o **modo plano** existente.

**Onde entra:** `.agents/recipes/*.toml` + `ktu run --recipe <nome> [--param k=v]`;
sub-receitas podem mapear para **subagentes** já existentes.

**Esforço:** baixo-médio. **Risco YAGNI:** baixo (reutiliza infra existente).

### 2.2 `slash_commands` — atalhos interactivos · **INCORPORAR**

**O que é:** comandos `/...` na sessão interactiva (TUI/run), resolvidos por **nome**
com `source` (`Builtin`/`Recipe`/`Skill`).

**Porque cabe no ktu:**
- **Usabilidade objectiva:** `/compact`, `/mode`, `/recipe`, `/skill` — mapeiam para
  comandos que **já existem na porta** (`SetMode`, `cancel`, `continue`).
- **Simplicidade:** um parser de linha + um registo de atalhos; nada no kernel.
- **Kernel:** os atalhos são **fronteira** (front-end), não política.

**Esforço:** baixo. **Risco YAGNI:** baixo (superfície pequena e fechada).

### 2.3 `permission_store` — aprovações memorizadas · **INCORPORAR**

**O que é:** modos de permissão (`auto`/`approve`/`chat`) + **store** que memoriza uma
aprovação (por tool/caminho) para não a pedir de novo.

**Porque cabe no ktu:**
- O ktu tem `Approval`/`Mode` e `Escalate`, mas **cada** escrita fora do root pede
  aprovação de novo. Uma memória de aprovações é usabilidade objectiva.
- **Kernel:** decisão sobre **factos tipados** (já é o seam do `Guard`); o store é um
  `BTreeMap` determinístico, logado.

**Esforço:** baixo-médio (store + regra "aprovado para (tool, path)"). **Risco:** alargar
privilégio — mitigar com escopo mínimo e TTL/expiração.

### 2.4 `StructuredSummary` — compactação estruturada · **AVALIAR**

**O que é:** a compactação do goose produz um resumo **estruturado** (`user_intent`,
`technical_concepts`, `files`/`FileActivity`, `errors_and_fixes`, `problem_solving`,
`pending_tasks`, `current_work`, `next_step`) com parsing **leniente** (campos extra não
descartam o resumo).

**Porque interessa ao ktu:** o gate G4 tem **Q4 com `compacted=true`**; um resumo com
campos estáveis (sobretudo `pending_tasks`/`next_step`) pode **melhorar a qualidade pós-
compactação**. O ktu já tem compactação aditiva; isto seria **enriquecer o resumo**, não
mudar o modelo.

**Esforço:** médio (schema + template + parsing leniente + teste de ouro). **Risco:** baixo,
mas é o único candidato que toca o **caminho crítico do gate** — medir antes/depois.

### 2.5 `token_counter` — contagem precisa · **AVALIAR**

**O que é:** contagem de tokens com `tiktoken` + estimativa do custo de esquemas de tools.

**Porque interessa:** o `BudgetGate` do ktu estima **≈4 bytes/token**. Uma contagem mais
fiel melhora a decisão de compactar/pinar. **Custo:** dependência `tiktoken-rs` (o ktu
tem zero deps de tokenização) e o modelo local usa outro tokenizer. **Veredicto:** só com
sintoma (orçamento a errar); provavelmente manter a estimativa.

### 2.6 `model_catalog` + `context_limit` — limites por modelo · **AVALIAR**

**O que é:** catálogo de modelos com **limite de contexto** e gatilho de compactação a
**0.8** da janela.

**Porque interessa:** o ktu tem `max_tokens` fixo no `BudgetGate`; derivar o tecto do
**contexto real do modelo** (ex.: `n_ctx` do `llama-server`) é uma melhoria pequena e
determinística. **Custo:** baixo (já se lê `n_ctx` no `providers doctor`).

### 2.7 `session export` (markdown/HTML) · **AVALIAR**

**O que é:** exportar a sessão para Markdown/HTML.

**Porque interessa:** o ktu já tem `ktu memory export --md`; exportar a **sessão** em
Markdown é barato e útil para revisão/handoff. **Custo:** baixo.

### 2.8 Já cobertos pelo ktu (não trazer)

`skills` (ktu tem `.agents/skills/`), `hints` (ktu tem `AGENTS.md` condensado),
`subagents` (ktu tem `explorer` + fan-out), `error handling` (ktu já devolve erros ao
modelo como tool result), `tool_inspection`/`tool_monitor` (ktu tem eventos estáveis),
`session`/`resume` (ktu tem).

---

## 3. Não trazer (YAGNI / fora de escopo)

| Feature do goose | Porquê não |
|---|---|
| MCP (70+ extensões) | YAGNI §0; plugins cobrem extensibilidade |
| ACP server / ACP providers | fora (o ktu tem a sua porta) |
| Desktop app / UI | fora (o ktu tem CLI/TUI/web mínimos) |
| Inferência local (candle) | o ktu usa `llama.cpp` |
| Roaming agents / peerbook | multi-agente distribuído, sem consumidor |
| Scheduler (tarefas agendadas) | YAGNI |
| OAuth / 15+ providers | só via providers finos por dialeto (ver backlog) |
| Voz/dictation, OTel, PostHog | fora |
| Download manager | o ktu tem `scripts/setup-model.sh` |
| `goose-agent` state machine | o ktu já tem o loop; adoptar seria reescrever |
| `goose-context-management` (crate) | adoptar só a **ideia** do resumo estruturado (2.4) |
| Custom distros / deeplinks | fora |

---

## 4. Recomendação

**Incorporar já (baixo risco, alto valor de usabilidade, zero efeitos no kernel):**
1. **Receitas** (§2.1) — dado declarativo, reutilização de workflows.
2. **Slash commands** (§2.2) — atalhos sobre comandos da porta.
3. **Permission store** (§2.3) — aprovações memorizadas.

**Avaliar com medição (tocam o orçamento/gate):**
4. **Resumo estruturado de compactação** (§2.4) — medir antes/depois no Q4.
5. **Limite de contexto por modelo** (§2.6) e **export de sessão** (§2.7) — baratos.

**Manter fora:** MCP, ACP, desktop, candle, roaming, scheduler, OAuth, voz, OTel,
download manager, e o state machine do goose.

> **Regra que se mantém:** cada ideia entra como **dado + teste**, nunca como subsistema.
> O goose dá **padrões de usabilidade** e um **schema de resumo** — não dá arquitectura
> para copiar.
