# Integrar o ktu num projecto Rust

Guia completo da **fachada de incorporação** (`ktu_core::facade::Ktu`): como abrir uma sessão,
correr turnos, injectar portas, configurar o projecto e diagnosticar o que corre mal — com
exemplos compilados.

> **Exemplo canónico (compilado no CI):** [`examples/ktu-embed`](../../examples/ktu-embed/src/main.rs)
> — `cargo run -p ktu-embed -- --backend scripted "explica o que fazes"`.
> **Contrato resumido:** [10-para-ia](./10-para-ia.md) (tabelas de API, invariantes e checklists).

## Pré-requisitos

| Requisito | Detalhe |
|---|---|
| Rust | `1.99`, edition `2024` (`rust-version` do workspace) |
| Crates | `ktu-core` (obrigatório); `ktu-tools`, `ktu-provider`, `ktu-memory`, `ktu-plugin` conforme o que injectares |
| Sistema | nenhum serviço obrigatório: com um provider scripted corre **sem modelo**; o provider real fala `llama-server` em `http://127.0.0.1:8080` |
| Async | a fachada é `async` (`tokio`, um runtime `current_thread` basta) |

## Quickstart (30 linhas)

```toml
# Cargo.toml
[dependencies]
ktu-core   = { path = "../ktu/crates/ktu-core" }   # ou git/registry
ktu-tools  = { path = "../ktu/crates/ktu-tools" }
tokio      = { version = "1", features = ["rt", "macros"] }
```

```rust
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::facade::Ktu;
use ktu_core::ports::{FixedClock, MemFs, NoopLogger, NoopMetrics, Timestamp};
use ktu_core::testing::{FakeProvider, FakeTools};
use ktu_core::session::SessionLog;
use ktu_core::agent::{AgentRegistry, AgentSpec};
use ktu_core::guard::AgentId;

let fs = MemFs::new();
let clock = FixedClock::new(Timestamp::from_millis(0));
let provider = FakeProvider::new(vec![/* completions */]);
let tools = FakeTools::new(vec![], vec![]);
let registry = AgentRegistry::new(AgentSpec::new(AgentId::Primary, vec![], 1), None);
let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
let cancel = CancelToken::new();
let logger = NoopLogger;
let metrics = NoopMetrics;

let ktu = Ktu::builder()
    .root("/projeto").fs(&fs).clock(&clock)
    .logger(&logger).metrics(&metrics)
    .provider(&provider).tools(&tools)
    .budget(&budget).cancel(&cancel).registry(&registry)
    .build()?;

let mut log = SessionLog::new();
let outcome = ktu.turn(&mut log, "faz X").await?;
println!("{}", outcome.text);
```

Com adaptadores reais é o mesmo, trocando os *fakes* por `StdFs`, um `Clock`/`Logger` teus e um
provider (`LlamaProvider`) — ver [09-exemplos](./09-exemplos.md) e o crate `ktu-embed`.

## Mapa dos documentos

| # | Documento | Cobre |
|---|---|---|
| — | [README](./README.md) | índice, quickstart, mapa de tarefas |
| 01 | [Panorama](./01-panorama.md) | onde a fachada vive, o que resolve, o que **não** faz, threads |
| 02 | [Construtor](./02-construtor.md) | referência de `KtuBuilder`/`Ktu`, obrigatoriedade, `build()`, ownership |
| 03 | [Turnos](./03-turnos.md) | `turn`, `TurnOutcome`, `StopReason`, cancelamento, steering, modos, guardrails |
| 04 | [Sessões](./04-sessoes.md) | `SessionLog`/`Event`, persistência, retoma, *checkpoint*, compactação, concorrência |
| 05 | [Portas e adaptadores](./05-portas-e-adaptadores.md) | cada porta, adaptadores prontos, *fakes*, escrever os teus |
| 06 | [Projecto](./06-projecto.md) | `.agents/`, `.ktu/`, guardrails, permissões, `AGENTS.md`, directriz, comandos, agentes, `memory.toml` |
| 07 | [Erros e diagnóstico](./07-erros-e-diagnostico.md) | catálogo de erros, fail-closed vs degradação, tabela sintoma→correção |
| 08 | [Boas práticas](./08-boas-praticas.md) | ownership, runtime, determinismo, logs, segurança, o que não contornar |
| 09 | [Exemplos](./09-exemplos.md) | receitas completas: teste, serviço, batch, memória, plano, plugins |
| 10 | [Para IA](./10-para-ia.md) | contrato resumido, invariantes, checklists, anti-padrões |

## Quero fazer X → vai a Y

| Quero… | Ver |
|---|---|
| correr um turno e obter a resposta | [03-turnos](./03-turnos.md) §1 |
| guardar/retomar sessões em disco | [04-sessoes](./04-sessoes.md) §2 |
| testar sem modelo, determinístico | [09-exemplos](./09-exemplos.md) §1 + [05](./05-portas-e-adaptadores.md) §4 |
| usar o `llama-server` local | [05](./05-portas-e-adaptadores.md) §2 |
| dar tools de ficheiro/bash ao modelo | [05](./05-portas-e-adaptadores.md) §2 (`ToolThread`) |
| dar memória durável (knudge) | [06](./06-projecto.md) §6 + [09](./09-exemplos.md) §5 |
| embeddings locais opcionais | [06](./06-projecto.md) §6.3 |
| modo plano read-only | [03-turnos](./03-turnos.md) §5 |
| dar instruções por directório (`AGENTS.md` aninhados) | [06](./06-projecto.md) §5 + [convenções](../convencoes.md) §3 |
| instruções locais ao ler um ficheiro | [03](./03-turnos.md) §9 + [convenções](../convencoes.md) §7 |
| encurtar o catálogo de tools (`tool_search`) | [03](./03-turnos.md) §9 + [convenções](../convencoes.md) §6 |
| correr uma receita com `settings`/*shell* | [convenções](../convencoes.md) §9 |
| substituir/estender o prompt do projecto | [02](./02-construtor.md) §2 (`.user_dir`) + [convenções](../convencoes.md) §2 |
| definir comandos (`/nome`) ou agentes próprios | [convenções](../convencoes.md) §4–§5 |
| cancelar um turno a meio | [03-turnos](./03-turnos.md) §4 |
| impedir uma tool (política) | [06](./06-projecto.md) §4 |
| perceber um erro/`StopReason` | [07](./07-erros-e-diagnostico.md) §4 |
| rever uma integração | [10-para-ia](./10-para-ia.md) §3 (checklists) |

## O que existe hoje (e o que está adiado)

**Existe:** a fachada (`Ktu`/`KtuBuilder`/`TurnOutcome`), a persistência de sessões pela porta `Fs`,
o *enforcement* estrutural por `AgentSpec` (modo plano incluído), guardrails/permissões
fail-closed, plugins, subagentes, memória com canal lexical ou semântico, e as **convenções do
projecto** (directriz `SYSTEM.md`/`APPEND_SYSTEM.md`, `AGENTS.md` aninhados, comandos
`.agents/commands/*.md`, agentes declarativos `.agents/agents/*.md`, título de sessão) — ver
[convenções](../convencoes.md).

**Adiado (com razão registada):** uma fachada que construa os adaptadores `std` por ti (crate nova),
`run_turn`/`Deps` privados, um tipo `Session` próprio, e `Env`/`Rng` no construtor (portas declaradas
sem consumidor no loop). Enquanto isso, o `Clock`/`Logger`/`SubagentHost`/`Steering`/`Instructions`
reais são **teus** — 30 linhas copiáveis em [05](./05-portas-e-adaptadores.md) §3 e no crate
`ktu-embed`.

## Contrato mínimo (leitura de 1 minuto)

1. **O log é a sessão** (I3): `turn` escreve num `SessionLog`; gravar/retomar é `save_session`/`session`.
2. **A fachada esconde o kernel**: não constróis `Deps`/`Ports`/`Capabilities`/`LoopConfig` nem chamas `run_turn`.
3. **Tudo o que é efeito entra por portas** (M2): a fachada é pura; tu injectas provider, tools, memória, plugins.
4. **Fail-closed** (I16): guardrails/permissões inválidos **impedem** o `build()`.
5. **Modo plano é read-only estrutural** (I15): sem `plan_spec` o modo plano é **recusado**.
6. **Um só dono do tecto de contexto** (I14): injecta **um** `BudgetGate`, partilhado com os subagentes.
7. **Um turno termina sempre com `StopReason`** (I5) — `TurnOutcome.reason`, nunca em silêncio.
8. **stdout = dado, stderr = log**; ids de evento estáveis (`ktu_core::events`).
9. **Nada de `unsafe`/`unwrap`/`expect`/`panic`** — a fachada devolve `Result` e erros são valores.
10. **Determinismo** (I6): com *fakes* (`FixedClock`, `MemFs`, provider scripted) o JSONL é byte-idêntico.

Próximo: [01 — Panorama](./01-panorama.md).
