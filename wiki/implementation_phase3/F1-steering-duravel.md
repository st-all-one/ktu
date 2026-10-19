# F1 — *Steering* durável e idempotente

> **Issue:** `task_00ly3qez` · **Âncora:**
> [steering-followup](../backlog/steering-followup.md) · **Desenho:**
> [revisao-pi-profunda](../revisao-pi-profunda.md) §2.3 · **Pré-requisitos:** H6 ·
> **Gatilho:** um front multi-cliente (web/TUI) **ou** um requisito de não perder a mensagem num
> *crash*.

## Objetivo

O *steering* **já funciona**: a mensagem entra numa fronteira segura e é logada (`Event::Steer`),
logo é durável *depois* de entregue. Falta a **admissão durável** (gravada no momento em que é
aceite, não só quando é entregue) e a **idempotência por id** — para que uma reentrega (retry do
front, `resume` após *crash*) não duplique a mensagem no contexto do modelo.

**Invariante novo — M7:** *uma mensagem admitida é entregue **exactamente uma vez***, mesmo após
*crash*/*resume* e mesmo que o front repita o pedido.

## Estado verificado (2026-10-18)

| Peça | Onde | Estado |
|---|---|---|
| entrega na fronteira segura | `agent/mod.rs:295` | feito (H6) |
| log da entrega | `session/mod.rs:27` (`Event::Steer`) | feito (H6) |
| porta | `Host::steer`, rota `/v1/session/{id}/steer` | feito (H6) |
| REPL entrega sem bloquear | `crates/ktu-cli/src/repl.rs` (`drain_input`) | feito (H6) |
| **admissão durável** | — | **falta** |
| **idempotência por id** | — | **falta** |

## Desenho proposto

1. **Dois eventos, um id.** `Event::SteerQueued { id, text }` (admissão, gravado **antes** de
   qualquer entrega) e `Event::Steer { id, text }` (entrega, com o **mesmo** id). A projeção
   (`derive_messages`) materializa só `Steer` e **descarta** um `Steer` cujo id já apareceu — é aí
   que vive a idempotência (I3: *model-visible ⟺ logged*).
2. **O id vem de fora.** O kernel **não** gera ids (sem `Rng`/tempo no caminho ⇒ I6): o id é
   gerado na **borda** e viaja no pedido — `Host::steer(id, text)`. Um front que repita o mesmo
   `id` é **idempotente por construção**.
3. **Recuperação no arranque.** Ao retomar (`resume`), um `SteerQueued` **sem** `Steer`
   correspondente é entregue na primeira fronteira segura do turno seguinte — a mensagem não se
   perde se o processo morrer a meio.
4. **Sem `select!`.** A entrega continua a ser *poll* entre eventos (nada de `tokio::select!`,
   banido por `disallowed_macros`).

## Passos

1. `Event::{SteerQueued, Steer}` com `id` (o `Steer` ganha campo) + `SessionLog::pending_steers()`.
2. Projeção: dedup por id (o primeiro `Steer` vence) + teste de reentrega.
3. Porta: `Host::steer` passa a `(id, text)`; `ktu-port`/`HttpPort` propagam; a borda gera o id.
4. REPL/HTTP: um id por mensagem (contador da borda, não do kernel).
5. Recuperação: no arranque do turno, drenar os `SteerQueued` pendentes.

## Tarefas kd

| Tarefa | Entrega |
|---|---|
| `task_01vw262v` | Inbox durável + idempotência (existente) |
| `task_01jzwiv3` | Projeção idempotente por id + recuperação no `resume` |

## Gate / testes

- **G16** — ver [GATES](./GATES.md).
- Unitário: `Steer` duplicado (mesmo id) ⇒ **uma** `Message::User` na projeção.
- Integração: gravar `SteerQueued`, terminar o processo antes da entrega, `resume` ⇒ a mensagem
  entra **uma vez** no turno seguinte.
- Negativo: ids diferentes com o **mesmo texto** ⇒ **duas** mensagens (não é dedup por texto).

## Riscos

- Duplicar a mensagem no contexto (o modelo vê-a duas vezes) → dedup **por id**, na projeção.
- Meter tempo/`Rng` no kernel para gerar ids → proibido (I6); o id vem da borda.
- Segundo caminho de entrega → reutilizar a fronteira segura existente (M2).

## Estado da implementação (2026-10-18) — **fechada** ✅

Gate **G16** verde (9/9): [relatório](../gates/G16-steering.md) · artefactos em
`wiki/gates/artifacts/G16/`.

| Entrega | Onde |
|---|---|
| `Event::SteerQueued { id, text }` (admissão) + `Event::Steer { id, text }` (entrega) | `crates/ktu-core/src/session/mod.rs` |
| `SessionLog::{pending_steers, knows_steer}` + dedup por id em `derive_messages` | `crates/ktu-core/src/session/mod.rs` |
| porta `Steering::take() -> Vec<SteerMessage>` | `crates/ktu-core/src/agent/deps.rs` |
| admissão **antes** da entrega, por fronteira | `crates/ktu-core/src/agent/steering.rs` (extraído de `mod.rs`) |
| `Host::steer(session_id, id, text)` (o id é do front) | `crates/ktu-port/src/{host,port,channel,http}.rs` |
| fila do `Handle` com dedup por id + contador `repl-<n>` | `crates/ktu-cli/src/hub.rs`, `crates/ktu-cli/src/repl.rs` |
| `POST /v1/session/{id}/steer {id?, text}` (`http-<n>` se ausente) | `crates/ktu-cli/src/serve.rs` |

**Semântica final:** a mensagem é admitida na fronteira em que chega e entregue na
**seguinte** (ou no turno seguinte, se o turno acabar antes — o *follow-up*). É o preço da
admissão durável: antes era entregue no mesmo passo e nunca ficava registada.

**Testes:** 2 no kernel (`a_queued_steer_is_pending_until_delivered`,
`a_repeated_steer_id_is_projected_once`, `a_repeated_queued_steer_id_is_pending_once`),
1 de loop (`a_pending_steer_is_delivered_in_the_next_turn`), 1 de REPL
(`a_line_during_a_turn_is_admitted_and_delivered_next_time`) e 1 de porta HTTP
(`http_steering_is_admitted_once_per_id`).

**Limite assumido:** a admissão é gravada na **fronteira do loop** (o kernel não tem I/O; a
gravação do log é do actor), pelo que a janela coberta é *admissão registada → morte antes
da entrega*. Fechar a janela anterior exigiria persistir no `steer()` da borda, com dois
escritores do mesmo log (risco de *lost update*) — **não** se fez (registado no gate).

## Notas de execução (skills Rust)

> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais + deltas da Fase 3).
- **N1:** os pendentes são um `BTreeMap` por id — determinismo no log.
- **N8:** `SteerState` (`Pendente`/`Entregue`) é enum, não bool.
- **N2:** um `SteerQueued` órfão (sem `Steer`) **não** é erro — é trabalho pendente.
- **Gate:** contribui para **G10** (cadeia longa com correcções a meio).
