# Fachada de incorporação (`ktu_core::facade::Ktu`)

> **Estado:** **implementado** (H9). Código: `crates/ktu-core/src/facade/`; teste de
> incorporação: `crates/ktu-core/tests/embed.rs`; exemplo compilado:
> [`examples/ktu-embed`](../examples/ktu-embed/src/main.rs). Âncora:
> [crate-facade](./backlog/crate-facade.md).
>
> **Guia de integração (detalhado, por tópicos):** [wiki/integration/](./integration/README.md)
> — construtor, turnos, sessões, portas, projecto, erros, boas práticas, exemplos e um resumo
> contratual [para IA](./integration/10-para-ia.md).

## O que é

O **único** caminho suportado para embutir o ktu noutro projecto Rust (uso #1 do
[propósito](./revisao-proposito.md) §2). Quem embute vê `Ktu`, `KtuBuilder`, `TurnOutcome`,
`SessionLog` e `Event` — e as portas que **injeta**. Nunca vê `Deps`, `Ports`, `Capabilities`,
`LoopConfig` nem `run_turn`.

```rust
let ktu = Ktu::builder()
    .root(".")
    .fs(&fs).clock(&clock).logger(&logger).metrics(&metrics)   // portas
    .provider(&provider).tools(&tools)                          // efeitos
    .budget(&budget).cancel(&cancel).registry(&registry)        // controlo
    .build()?;                                                  // fail-closed
let mut log = SessionLog::new();                                // o log É a sessão (I3)
let outcome = ktu.turn(&mut log, "faz X").await?;               // TurnOutcome
ktu.save_session("s1", &log)?;                                  // persistência pela porta Fs
```

O exemplo acima é um **doc test executável** (com *fakes*), na documentação do módulo.

## A API

| Entrada | Obrigatória | Papel |
|---|---|---|
| `root` | ✅ | raiz do projecto (`.agents/`, `.ktu/`, `AGENTS.md`) |
| `fs` | ✅ | guardrails, permissões, `AGENTS.md`, sessões |
| `clock` / `logger` / `metrics` | ✅ | telemetria e tempo (nunca silenciados por omissão) |
| `provider` / `tools` | ✅ | o modelo e o executor de tools (T2) |
| `budget` / `cancel` | ✅ | tecto de contexto (I14) e sinal de cancelamento (I4) |
| `registry` | ✅ | agente primário: tools oferecidas + *enforcement* estrutural |
| `plan_spec` | — | especificação read-only; **sem ela o modo plano é recusado** |
| `memory` / `hooks` / `subagents` / `steering` | — | capacidades opcionais (default: desligadas) |
| `max_steps` | — | tecto de passos (default: `LoopConfig`) |

| Saída | Papel |
|---|---|
| `turn(&self, log, prompt) -> Result<TurnOutcome>` | um turno; `TurnOutcome { reason, text }` (I5) |
| `session(&self, id)` / `save_session(&self, id, log)` | ler/gravar `.ktu/sessions/<id>.jsonl` |
| `root()` / `system()` / `sessions_dir()` | inspecção |
| `Ktu::CONTEXT_BUDGET_TOKENS` | tecto de contexto por turno (dono único: `BudgetGate`) |

## A separação: composição pura vs adaptadores

O `ktu-core` é **puro** (M2): não constrói adaptadores reais. Por isso a fachada faz a parte
**pura** da composição e recebe o resto por injecção:

| A fachada resolve (puro, via portas) | A borda injecta (efeitos) |
|---|---|
| guardrails declarativos (fail-closed, I16) | provider do modelo |
| permissões do projecto (último elo da cadeia) | thread de tools (T2) |
| system prompt (`BASE` + `AGENTS.md` condensado) | memória (`ktu-memory`) |
| orçamento, cancelamento, modo → `AgentSpec` | plugins, subagentes, *steering* |
| sessão (`store::load_recovering` / `store::save`) | adaptadores `std` (`StdFs`, relógio, log) |

Assim, a fachada **escolhe o `AgentSpec` pelo modo** (`build` → primário; `plan` → read-only) e
aplica-o em duas camadas: `ScopedTools` (tools oferecidas) e `StructuralGuard` (recusa estrutural)
— o read-only do modo plano nunca depende do prompt.

## Invariantes travados por teste

| Invariante | Prova |
|---|---|
| **I3** (o log é a sessão) | `turn` escreve no `SessionLog` dado; `save_session`/`session` fazem o *round-trip* |
| **I5** (nunca em silêncio) | `TurnOutcome.reason`; teste do cancelamento devolve `StopReason::Cancelled` |
| **I15** (plano read-only estrutural) | `plan_mode_offers_only_the_read_only_tools`; sem `plan_spec` ⇒ `Error::Invalid` |
| **I16** (guardrails fail-closed) | `a_broken_guardrail_stops_the_build` / `broken_permissions_stop_the_build` (nomeiam o ficheiro) |
| **I14** (um dono do tecto) | o `BudgetGate` é injectado e partilhado com os subagentes |
| **I4** (cancelável) | `the_injected_cancel_token_stops_the_turn` |
| **I6** (replay determinístico) | `the_same_script_gives_byte_identical_logs` (JSONL byte-idêntico) |

## Como o CLI a usa (M4 — nenhum caminho com lógica própria)

O `ktu-cli` deixou de ter composição própria: `turn_runtime::execute_turn` constrói os
**adaptadores reais** e entrega-os à fachada, que corre o turno. `ktu run`, o REPL (`ktu session`),
o `serve` e o `--batch` passam todos por ali; o `prime` continua a ser o protocolo do modelo.

Consequência: o **caminho crate** e os caminhos da CLI partilham a **mesma** composição — não há
duas listas de guardrails nem dois system prompts para manter em sincronia.

## O que **não** é exposto (adiado)

- **`run_turn`/`Deps` como API privada:** continuam públicos porque o host de subagentes
  (`CliSubagents`) e os testes do loop os usam; a fachada é o único caminho **suportado** para
  quem embute. Torná-los `pub(crate)` exige primeiro dar ao host de subagentes uma porta própria.
- **Fachada que constrói adaptadores `std`** (como o `Knudge` do knudge, `D214`): no ktu isso
  implicaria uma crate nova acima de `ktu-tools`/`ktu-memory`/`ktu-plugin`; fica adiado até haver
  um embedder que não queira montar as suas portas.
- **`Session` própria / `Ktu` dona do log:** o log é explícito (`&mut SessionLog`) — é a I3
  tornada visível, e é o que a CLI precisa (lock, *checkpoint*, projecção).
- **`Env`/`Rng` na fachada:** são portas declaradas mas **sem consumidor** no loop — aceitá-las
  seria cerimónia (YAGNI); quando aparecer um consumidor, o construtor cresce.
