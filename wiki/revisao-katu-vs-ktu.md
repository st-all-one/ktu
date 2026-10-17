# Revisão: `katu` (1.ª tentativa) × `ktu` (2.ª tentativa)

> Documento de análise, não de plano. Compara o antecessor `REF/katu` com o `ktu`
> atual: o que foi feito, o que não foi, as abordagens/ferramentas/táticas e o
> estado do novo projecto. Base: código e docs de ambos + `plan.md`/`epic.md` do ktu.
> Data: 2026-10-17.

---

## 0. Sumário executivo

O **katu** tinha a tese certa e a arquitectura certa, mas construiu os subsistemas
**pela ordem errada**: motor de política, auditoria densa, rate-distortion, confiança
e sandbox **antes** de o loop sequer funcionar. Resultado: ~73 mil linhas, ~1000
testes, e **zero sessões a funcionar** — o contexto perdia-se, o modelo entrava em
loop e o wire dos providers dava `HTTP 400`.

O **ktu** inverte a ordem: **fazer funcionar, depois melhorar**. Reconstrói o
*slice vertical* mínimo (kernel puro + portas + 2 threads + porta única + front-ends),
atinge o **gate de qualidade** (`quality_test` p1–p4, equivalência de grau) e só
então acrescenta robustez — com **um portão único** (`make check`) e a lista do
**não-fazer** a travar cada subsistema até haver um teste que o exija.

**Números** (ordem de grandeza):

| | `katu` (1.ª) | `ktu` (2.ª) |
|---|---|---|
| Linhas `src/` (crates + xtask) | ~72 800 | ~14 200 |
| Testes (`#[test]`/`#[tokio::test]`) | ~1 001 | ~161 |
| Crates (+ xtask) | 6 | 7 |
| Pacotes no `Cargo.lock` | 276 | 345 |
| Limite de ficheiro | 400 linhas | **300 linhas** |
| MSRV | 1.97 | 1.99 |
| Binário de release | — | 6,0 MB |
| Sessões a funcionar | **0** (thrashing, loop) | **sim** (gate G4 verde) |

---

## 1. O que era o `katu`

**Tese** (`REF/katu/wiki/_ref/plan/00-tese-e-escopo.md`): o knudge define o que é
boa memória; o katu seria a metade que **torna impossível** violar o protocolo —
*enforcement* determinístico sobre factos tipados, com evidência, não prompts.

**Arquitectura** (`ARCHITECTURE.md`): kernel + política puros, fronteira LLM-free,
portas (`Clock`/`Rng`/`Fs`/`Env`) com fakes, `diag` transversal, log append-only,
TOON/colunar para economia de tokens, motor de política como **dado**
(`policy/containment.toml`, `prices.toml`, `tiers.toml`, `coverage-ledger.json`),
26 ADRs.

**Onde falhou** (resumido do `plan.md` §0 e do próprio `STAGING.md` do katu):

| Falha | Causa-raiz |
|---|---|
| Contexto perdido a cada turno | estado vivo só em memória; o log não era a fonte da verdade |
| Tool com argumento inválido repetida 3× | schema fraca; erro de tool não voltava de forma accionável |
| Sessão não retomável | compactação ambígua |
| `HTTP 400` com ≥2 tool calls | um `assistant` por `ToolCall` em vez de **um passo = uma mensagem** |
| Modelo relia os mesmos ficheiros em ciclo | `raw_min` de contexto pequeno (4 k) → thrashing |
| 70 k linhas, 0 sessões | policy/audit/rate-distortion/confiança/containment **antes** do loop |

A lição não foi "as ideias estavam erradas" — foi **a ordem de construção**.

---

## 2. O que é o `ktu`

**Tese** (`plan.md` §0): *fazer funcionar, depois melhorar*. Um **slice vertical que
funciona** e só avança quando o critério de qualidade é atingido. O gate (G4) é
**inegociável**: o agente tem de alcançar o `quality_test` (p1–p4, como sujeito
isolado, com persistência entre sessões) antes de qualquer fase seguinte.

**Arquitectura** (`plan.md` §2):

```
ktu-core   kernel puro: domínio, portas, loop agêntico (sem efeitos)
ktu-tools  thread OS dedicada (T2): fs, exec, memória, subagentes
ktu-provider  built-in: llama.cpp via llama-server (SSE, tool-calling nativo)
ktu-port   porta única: protocolo versionado + Channel (embedded) / Http (SSE)
ktu-memory porta Memory + motor knudge (hints são ponteiros, não o corpo)
ktu-plugin runtime de plugins externos (JSON-lines stdio) + hot reload
ktu-cli    binário: run / serve / tui / web + adaptadores das portas
```

- **Kernel puro** + **portas** (`Clock`/`Rng`/`Fs`/`Env`/`Logger`/`Metrics`) com fakes;
  zero `unsafe`/`unwrap`/`expect`/`panic` em `src/` (I7).
- **Duas threads**: kernel e tools em threads separadas, sem estado partilhado com o
  front-end (I8); `tool.result` num ponto único.
- **Porta única** agnóstica; `run` (embedded), `serve` (axum/SSE), TUI (ratatui) e
  web são **clientes** da mesma porta, sobre a mesma sessão.
- **Determinismo**: só `BTreeMap`/`BTreeSet`; ids de evento estáveis; stdout = dados,
  stderr = logs.

---

## 3. O que foi **feito** (adoptado do katu)

O `plan.md` §2.12 faz o veredicto explícito do que herdar. **Adoptado agora** (crítico
para a métrica):

| Do katu | No ktu | Porquê |
|---|---|---|
| `context/select` + `pin_unit` + `BudgetGate` | §2.10, I14 (um só dono do teto) | o thrashing de contexto era a falha central |
| `kernel/project::wire_messages` | §2.4 (um passo = uma mensagem) | era o `HTTP 400` real |
| `prompt.rs` + `skill.rs` | §2.11 (AGENTS.md condensado + skills) | sem instruções de projecto não há qualidade |
| `taint.rs` (spotlighting) | I13 (envelope *untrusted*) | defesa contra prompt injection |
| `agent.stall` + taxonomia | §2.4, I5 (`StopReason` fechado) | modo de falha observado |
| `kernel/memory_gate` | §2.7 (recall/write automáticos) | persistência não pode depender do modelo |
| `kernel/budget` + caps + `prices`/`tiers` | §2.10 + guardrail | custo/tokens sem dono corre sem fim |
| `tools/trash` | tool `trash` + `ktu trash` | o agente nunca destrói irreversivelmente |
| `plan` (artefacto) | §2.13 (modo plano + `.ktu/plan/*.json`) | tarefa longa sem plano perde o fio |

**Acrescentado na 2.ª tentativa** (não existia, ou era só *seam*, no katu):

- **Guardrails em 3 camadas** (código → declarativos → plugins), fail-closed (I10/I16),
  com *seed* em `.agents/guardrails/*.md`.
- **Plugins com hot reload por geração** (I11/I12): gate só **nega**, nunca liberta um
  `Deny` estrutural; ordem kernel → gates → execução.
- **Porta única + front-ends** (G6) e **comandos out-of-band** (cancel/SetMode/continue).
- **Subagentes read-only** com fan-out determinístico + **modo plano** (I9/I15).
- **Gate de qualidade** executável (`xtask quality` + `scripts/quality.sh`), isolado por
  sujeito, com **sonda de persistência** entre sessões.
- **Packaging/releases** (`install.sh`, `dist.sh`, workflow de release) e **benchmarks
  com evidência** (`wiki/benchmarks/ktu-bench.json`).

---

## 4. O que **não** foi feito (deferido/rejeitado)

O katu tinha estes subsistemas; o ktu **deferiu-os com razão** (`plan.md` §2.12-C): cada
um só entra com **um consumidor atual e um teste que o exija**.

| Subsistema do katu | Veredicto no ktu |
|---|---|
| `audit/` (bin + bloom + índice) | deferido — não há consumidor |
| `confidence/` + FDR/conformal | deferido — estatística sem caso de uso |
| `feedback.rs` | deferido |
| `diag` com catálogo de 122 eventos | reduzido a um conjunto estável de eventos |
| Contexto por rate-distortion/submodular | deferido — a heurística simples bastou |
| `prices`/`tiers` completos | mínimo (o custo sem dono é que importa) |
| plugin ABI / WASM / dispatch paralelo | fechado como **não justificado** por medição (`decision_01e2183g`) |
| MCP | fora (YAGNI §0) |
| providers além de llama.cpp + GDK | **GDK decidido não implementar** (`decision_00aw5etj`); caminho OpenAI-compatible cobre o presente |
| `stats`/`report` | deferido |
| contenção **soft** | substituída por **guardrails declarativos fail-closed** (mais simples e determinística) |
| sandbox de SO (Landlock/seccomp) | fora; mitigação de `bash` por análise de *argv* (determinística) |
| TUI extravagante | TUI mínima como cliente da porta |

**Nota de convergência:** o próprio `12-plugins-e-abi.md` do katu dizia que o *seam* de
controlo é a **política (dado)** e que plugins são **efeito colateral**, nunca regra — é
exatamente o que o ktu adoptou (§2.9).

> Cada uma destas features tem uma **página de decisão** em [`wiki/backlog/`](backlog/README.md)
> (o que é, porque não entrou, pré-condições para reconsiderar, custo/risco, decisão pendente).

---

## 5. Abordagens, ferramentas e táticas

### 5.1 Método

- **Fases G0–G9**, cada uma fechada com evidência; a anterior não abre a seguinte
  ("ou o slice funciona, ou não passou").
- **Um portão único:** `make check` = `fmt --check` + `clippy -D warnings` + `test` +
  firewall de camadas (`xtask check-layers`) + tamanho de ficheiro (≤ 300 linhas).
- **YAGNI explícito** (`plan.md` §0): nada entra sem issue que aponte o teste que o exija.
- **Performance terciária:** só depois do gate e **só com número** — daí os benchmarks.

### 5.2 Ferramentas

| Ferramenta | Uso |
|---|---|
| `cargo` + `rust-toolchain.toml` (1.99) | build/test; MSRV travada |
| `clippy.toml` + `[workspace.lints]` | rigor máximo; proíbe `unwrap`/`panic`, `HashMap`, `Instant::now`, etc. |
| `rustfmt.toml` (edition 2024) | estilo único |
| `cargo-deny` + `cargo-machete` + `typos` | supply chain (`make supply-chain`) |
| **Miri** (nightly, só `miri_smoke`) | prova de ausência de UB no núcleo |
| `proptest` | propriedades puras |
| `xtask` | `check-layers`, `quality` |
| `scripts/quality.sh` + `wiki/quality_test/` | bateria p1–p4 isolada por sujeito |
| `scripts/dist.sh` + `install.sh` + workflow de release | empacotamento e releases |
| `benches/core.rs` | benchmarks determinísticos com artefacto JSON |
| **knudge** (`kd`) | memória do projecto: `ask` → `write` → `task` |

### 5.3 Táticas de engenharia (o que muda face ao katu)

- **Portas/DI** com fakes → testes do núcleo reprodutíveis byte a byte (herdado do katu,
  agora com a superfície mínima).
- **Thread T2** dedicada para tools (mpsc + oneshot), em vez de contaminar o loop.
- **Actor por sessão** (`SessionHub`): dono do `SessionLog`, `cancel` fora-de-banda.
- **`turn_runtime::execute_turn` partilhado** entre `ktu run` e `ktu serve` (zero DI duplicada).
- **Eventos de log estáveis** como contrato; `tool.result` num ponto único.
- **Testes de ouro** (contrato de bytes), *fixtures* adversariais para guardrails, e
  *test double* HTTP para o streaming sem modelo.
- **Hot reload por geração**: `dispatch` fixa o `Arc<Dispatcher>` da geração corrente (I12).
- **Sem commits pelo agente**: o versionamento é humano (regra local).

---

## 6. Estado atual do `ktu`

**Fases:** G0–G9 concluídas (`epic_00ynu6q8` fechado).

**Funcionalidades:** `ktu init` · `run` (embedded) · `serve` (axum/SSE) · `tui` ·
web · `providers` · `trash` · `memory` · `doctor` · `mode` · `plugins` · `guardrails`.

**Qualidade (verificada):**

- `make check` **verde** e `make supply-chain` **verde**
  (`advisories ok, bans ok, licenses ok, sources ok` · `machete` zero · `typos` ok).
- **Gate G4** re-validado com o **modelo de referência** (`deepseek-v4.1-flash`):
  equivalência de grau em Q1–Q4, `compacted=true`, sonda de persistência = passa
  (`wiki/quality_test/ktu-veredicto.md`).
- **161 testes**; todos os `src/` ≤ 300 linhas; zero `HashMap`/`HashSet`;
  sem `unsafe`/`unwrap`/`expect`/`panic` em `src/`.
- **17 invariantes** (I1–I17) travadas por teste.
- **knudge saudável**: `healthy: true`, `audit.clean: true`, `total: 0`.

**Números:** ~14 200 linhas `src/`, 7 crates, 345 pacotes, binário 6,0 MB.

**Evidência produzida:** `wiki/quality_test/` (artefactos p1–p4 + veredicto),
`wiki/benchmarks/` (benchmark + leitura), `STAGING.md` (§1–§13), `epic.md`, `plan.md`.

---

## 7. Veredicto

- **O ktu cumpriu a tese do katu, invertendo a ordem.** O que faltava ao katu era um
  **slice vertical que funciona**; o ktu construiu-o, submeteu-o ao gate inegociável e
  só depois endureceu.
- **A barra de qualidade do katu foi mantida** (portas/DI, determinismo, rigor de lints,
  ficheiros pequenos, log como fonte da verdade) — mas com **1/5 do código** e um
  **portão único** que se corre num comando.
- **O que ficou de fora não foi esquecido:** está **documentado e decidido**
  (deferido com razão, ou fechado como não justificado por medição). É a diferença
  essencial face à 1.ª tentativa — que construía subsistemas sem consumidor.
- **O que ainda falta** é *opcional* e conhecido: higiene menor do knudge (1 warn
  advisório de `body`), reabrir a G9 se surgir necessidade de outro provider, e cortar
  a release (gesto humano). O núcleo do produto está **completo e verde**.

> **Regra que se provou:** um agente moderno não falha por falta de ideias — falha por
> ordem de construção e por falta de um critério de qualidade que o trave. O ktu nasceu
> com esse critério (G4) e nunca o contornou.
