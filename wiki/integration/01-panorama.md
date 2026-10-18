# 01 — Panorama: onde a fachada vive e o que ela resolve

> **Público:** devs que vão embutir o ktu · **Índice:** [README](./README.md) · **Seguinte:** [02 — Construtor](./02-construtor.md)

## 1. O problema que a fachada resolve

O núcleo do ktu é **funcional e cru**: para correr um turno à mão terias de montar
`Deps` + `Ports` + `Telemetry` + `Capabilities`, escolher o `AgentSpec`, carregar guardrails e
permissões, compor o system prompt, criar o `BudgetGate`, o `CancelToken` e o `LoopConfig` — e só
depois chamar `run_turn`. São seis tipos internos e uma ordem que ninguém quer decorar.

A fachada é a **composição pronta**: um construtor com dependências explícitas, um método `turn` e
um resultado estável. Quem embute nunca vê `Deps`/`Ports`/`Capabilities`/`LoopConfig`/`run_turn`.

```rust
let ktu = Ktu::builder()…build()?;          // uma vez por sessão
let outcome = ktu.turn(&mut log, "…").await?;  // N vezes
```

## 2. Onde vive (e porque não é uma crate nova)

`ktu_core::facade` — **dentro do núcleo puro**. O `ktu-core` não conhece `ktu-tools`,
`ktu-provider`, `ktu-memory` nem `ktu-plugin` (firewall de camadas), e por isso a fachada **não
constrói adaptadores reais**: recebe-os por injecção. É a diferença face ao knudge (onde a fachada
`Knudge` vive numa crate que tem adaptadores `std` dentro).

```
┌──────────────────────────────── a tua aplicação ────────────────────────────────┐
│  main / serviço / teste                                                          │
│    ├── adaptadores reais: StdFs (ktu-tools), o teu Clock/Logger, LlamaProvider…   │
│    ├── Ktu::builder()…build()?          ← ktu_core::facade (composição pura)      │
│    └── ktu.turn(&mut log, prompt).await? ← loop do kernel                         │
└──────────────────────────────────────────────────────────────────────────────────┘
                │ injecta                          ▲ eventos/resultado
                ▼                                  │
┌───────────────────────────── ktu-core (puro) ────────────────────────────────────┐
│  facade · loop agêntico · sessão (log) · guardrails · permissões · plano ·        │
│  orçamento · cancelamento · compactação          ── portas: Provider, Tools,      │
│                                                     Memory, Hooks, Clock, Fs…    │
└──────────────────────────────────────────────────────────────────────────────────┘
```

**Consequência prática:** o `Clock`/`Logger`/`SubagentHost`/`Steering` reais vivem na CLI
(`ktu-cli`), que é um binário — logo **tu escreves os teus** (são pequenos; ver
[05](./05-portas-e-adaptadores.md) §3). Está adiado, com razão registada, uma fachada que os
construa por ti.

## 3. O que a fachada resolve vs o que injectas

| Resolvido pela fachada (puro, via portas) | Injectado por ti (efeitos) |
|---|---|
| guardrails declarativos de `.agents/guardrails/*.md` (**fail-closed**, I16) | `Provider` (modelo) |
| permissões de `.agents/permissions.toml` (último elo da cadeia) | `ToolExecutor` (as tools, tipicamente `ToolThread`) |
| system prompt = `BASE` + `AGENTS.md` condensado | `Memory` (ex.: `JsonlMemory`) |
| cadeia de guards (`StructuralGuard` → declarativos → permissões) | `Hooks` (ex.: `PluginHost`) |
| orçamento (`BudgetGate`), cancelamento (`CancelToken`) | `SubagentHost`, `Steering` (opcionais) |
| `AgentSpec` **pelo modo** (`build` → primário; `plan` → read-only) + `ScopedTools` | portas `Fs`, `Clock`, `Logger`, `Metrics` |
| sessão: ler/gravar `.ktu/sessions/<id>.jsonl` | directórios de runtime (a porta `Fs` não cria pastas) |

## 4. Duas threads, um log

O desenho do ktu tem **duas threads** (§2.2 do `plan.md`) e a fachada respeita-o:

| Thread | Quem | O que corre |
|---|---|---|
| **T1 — kernel** | `ktu-core` | o loop, o log, o provider, as decisões de turno (é onde o teu `turn` corre) |
| **T2 — tools** | `ktu-tools` | execução das tools (fs, bash, memória), com timeout e caps de saída |

A comunicação é por canais: `ToolThread::spawn(|ctx| ctx.serve(&toolbox))` devolve um handle que
implementa `ToolExecutor`; passas `&tool_thread` ao construtor e o kernel fala com a T2 sem
conhecer o filesystem. **Tu não geres canais** — geres o handle.

## 5. O que a fachada **não** faz (de propósito)

| Não faz | Onde está |
|---|---|
| ler `config.toml` (provider/modelo/effort/max_steps) | a app resolve a precedência (flags > params > config > defaults) — ver [06](./06-projecto.md) §7 |
| criar directórios (`.ktu/sessions`, …) | a borda, com `std::fs::create_dir_all` (a porta `Fs` não cria pastas) |
| lock de sessão / concorrência | a app (a CLI usa um lock por ficheiro; ver [04](./04-sessoes.md) §5) |
| sockets, HTTP, TUI, argv, sinais | a app — ou `ktu-port` + `ktu-cli` como referência |
| *checkpoint* / compactação forçada | a app (a CLI grava `checkpoint::Checkpoint`; ver [04](./04-sessoes.md) §4) |
| dreno de embeddings / projeção de sessão | `ktu memory drain` / `ktu memory project` (CLI) ou `ktu_memory::{drain, project}` |
| subagentes concretos | tu implementas `SubagentHost` (ou omites a capacidade) |

## 6. Quando **não** usar a fachada

| Objectivo | Alternativa |
|---|---|
| queres só o loop, com DI totalmente explícita | `ktu_core::agent::run_turn` (API de baixo nível) |
| queres um processo separado a falar com o ktu | `ktu-cli` (`ktu run`/`serve`) ou `ktu-port` (protocolo `ClientMsg`/`ServerMsg`) |
| queres reutilizar só a memória (knudge) | `ktu_memory::JsonlMemory` / `knudge_core` directamente |
| queres só o protocolo do agente para o prompt | `ktu_core::prompt::{BASE, compose}` |

## 7. Invariantes que a fachada trava por ti

| # | Invariante | Efeito na tua integração |
|---|---|---|
| I3 | o log é a fonte da verdade | guarda o `SessionLog`; a sessão é ele |
| I5 | um turno termina sempre com `StopReason` | trata `TurnOutcome.reason` (incl. `Error`) |
| I6 | replay determinístico | com *fakes*, o JSONL é byte-idêntico |
| I14 | um só dono do tecto de contexto | injecta **um** `BudgetGate` (e o mesmo para subagentes) |
| I15 | modo plano é read-only estrutural | o `AgentSpec` é escolhido pelo modo, não pelo prompt |
| I16 | guardrails fail-closed | um ficheiro inválido impede o arranque (nomeia o ficheiro) |
| I4 | cancelável | o `CancelToken` que injectas é o que a UI cancela |
| M2 | efeitos só por portas | não há atalhos: se queres um efeito, implementa a porta |

Próximo: [02 — Construtor](./02-construtor.md).
