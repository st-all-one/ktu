# Crate facade — embutir o ktu noutro projecto Rust

- **Estado:** **implementado** (H9) — `ktu_core::facade::{Ktu, KtuBuilder, TurnOutcome}`, com
  exemplo de incorporação como **doc test** e teste de integração; a CLI passou a usar a fachada
  (uma só composição). **Adiado:** fachada que constrói adaptadores `std` (crate nova) e
  `run_turn`/`Deps` privados — ver [fachada-crate](../fachada-crate.md) §"O que não é exposto".
- **Origem:** revisão por propósito §2 (uso #1); knudge D214 (facade + layout).
- **Âncora:** `crates/ktu-core/src/facade/mod.rs`, `crates/ktu-core/tests/embed.rs`.

## O que é
Uma **fachada** pública e estável sobre o núcleo, para outro crate Rust abrir uma sessão e correr
turnos sem conhecer `Deps`/`LoopConfig`/`Ports`/`Capabilities`:

```rust
let ktu = Ktu::builder()
    .root(".")
    .provider(my_provider)      // porta
    .ports(my_ports)            // Clock/Rng/Fs/Env/Logger/Metrics
    .build()?;
let out = ktu.turn("faz X")?;   // um turno
let log = ktu.session("id")?;   // sessão persistida
```

A fachada **esconde** os internos (como `Knudge`/`KnudgeBuilder` do knudge, D214): o embedder vê
`Ktu`, `TurnOutcome`, `Event`, e as **portas** que quiser substituir — nunca o kernel.

## Porque interessa
É o **uso #1**: integrar o core em pipelines/serviços Rust. Hoje o núcleo é **funcional** (`run_turn`
com structs soltos), não **ergonómico** — um embedder tem de montar 6 tipos. A fachada é o contrato
público que estabiliza a API e permite evoluir o kernel sem quebrar quem embute.

## O que o ktu tem hoje
`run_turn(log, deps, config)` + `Deps`/`Ports`/`LoopConfig`/`Capabilities`/`Telemetry` públicos.
Nenhuma fachada. `testing` dá fakes.

## Pré-condições para reconsiderar
- Um consumidor real que embuta o ktu (não só o binário) — **cumprida**: a própria CLI passou a
  ser esse consumidor (a composição é a mesma; não há duas listas de guardrails).

## Custo e risco
- **Esforço:** baixo-médio (uma struct + builder; sem lógica nova).
- **Risco de não fazer:** API pública instável; ninguém embute.
- **Risco de fazer:** expor demais — mitigar escondendo o kernel atrás da fachada (o embedder só
  vê portas e eventos).

## Decisão pendente
Incorporar **quando** houver um embedder. A fachada é o **único** caminho suportado de integração;
nunca expor `run_turn`/`Deps` como API pública a longo prazo.
