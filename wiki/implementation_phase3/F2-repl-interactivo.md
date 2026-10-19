# F2 — REPL interactivo (Ctrl-C, Tab, comandos `/`)

> **Issue:** `task_01mrz94g` · **Âncora:** [cli-repl](../backlog/cli-repl.md) ·
> **Pré-requisitos:** H4 (REPL), H10 (comandos `/`) · **Gatilho:** uso interactivo real a pedir o
> atalho (o REPL já é o front primário — ADR 0010).

## Objetivo

Fechar a UX do REPL **sem lhe dar lógica própria** (M4): **Ctrl-C** cancela o turno em curso
**sem matar o processo**, **Tab** completa `/` e caminhos, e os `/` que faltam mapeiam verbos que
**já existem** na CLI.

## Estado verificado (2026-10-18)

| Peça | Estado |
|---|---|
| `/help`, `/quit`, `/plan`, `/build`, `/mode`, `/cancel` + `/nome` (comandos do projecto) | feito (H4/H10) |
| `ktu session ls|rm|fork` | feito (H1) |
| `--compact` (em `ktu run`/`ktu session`) | feito |
| `ktu memory ls|export|project` | feito (H7) |
| **Ctrl-C cancela o turno** | **falta** (sem tratamento de SIGINT no REPL) |
| **Tab** (*raw mode*) | **falta** (sem `crossterm` no `repl.rs`) |
| `/compact`, `/fork`, `/memory`, `/resume` no REPL | **falta o mapeamento** |

> Correcção ao texto antigo (A5): dizia que estes `/` «não têm verbo na porta» — é **parcialmente
> falso**. Os verbos existem na CLI (`session fork`, `--compact`, `memory`); falta o
> **mapeamento** no actor do REPL (`Cmd` só tem `Turn`/`SetMode`/`State`).

## Passos

1. **Comandos `/` (sem *raw mode*):** `Cmd::{Compact, Fork, Memory, Resume}` no actor + `Slash`
   correspondentes. Cada um **delega** (nada de lógica no REPL): `Compact` → projeção forçada,
   `Fork`/`Resume` → `session_cmd`, `Memory` → `memory_cmd`. Actualizar o `/help`.
2. **Ctrl-C:** handler de SIGINT (borda) que chama o `CancelToken` da sessão — o processo **não**
   morre e o turno fecha com `StopReason::Cancelled`. Em *pipe* (não-TTY) mantém-se o
   comportamento actual.
3. **Tab (*raw mode*):** `crossterm` (já é dependência do `tui`) para completar `/` (embutidos +
   comandos do projecto) e caminhos (leitura do directório). **Só** quando `stdin`/`stderr` são
   TTY; em *pipe* cai no modo linha-a-linha actual.
4. **Histórico** por sessão (`.ktu/sessions/<id>.history`), sem segredos.

## Tarefas kd

| Tarefa | Entrega |
|---|---|
| `task_00dbywbk` | Comandos `/` que faltam (mapeamento) |
| `task_01r72uu1` | Ctrl-C + Tab (*raw mode*) |

## Gate / testes

- **G17** — ver [GATES](./GATES.md).
- `/compact` no REPL ⇒ a mesma projeção que `--compact` na CLI (byte-a-byte).
- SIGINT durante um turno ⇒ o processo **não** morre e o log fecha com `StopReason::Cancelled`.
- Em *pipe*, o REPL continua linha-a-linha: o **G12** (log byte-idêntico `run`/REPL/crate) é o
  teste de regressão.

## Riscos

- *Raw mode* a quebrar o modo *pipe* → **gated** por `is_terminal()`; o G12 trava a regressão.
- Lógica própria no REPL (viola M4) → cada `/` é um comando da porta; o REPL só mapeia.
- Sinais com `unsafe` → proibido (`forbid(unsafe_code)`); usar `crossterm`/handler da borda.

## Estado da implementação (2026-10-18) — **fechada excepto a Tab**

Gate **G17** verde (10/10): [relatório](../gates/G17-repl.md) · artefactos em
`wiki/gates/artifacts/G17/`.

| Entrega | Onde |
|---|---|
| `Slash::{Compact, Fork, Memory, Resume}` + ajuda | `crates/ktu-cli/src/slash.rs` |
| Delegação de cada `/` (sem lógica no REPL) | `crates/ktu-cli/src/repl_slash.rs` (novo) |
| `Cmd::Compact` no actor + `SessionHub::compact` | `crates/ktu-cli/src/hub/actor.rs`, `crates/ktu-cli/src/hub.rs` |
| `session_cmd::{copy, ids}` (reutilizados pelo `/fork` e `/resume`) | `crates/ktu-cli/src/session_cmd.rs` |
| `memory_cmd::list_text` (o `/memory` mostra em `stderr`) | `crates/ktu-cli/src/memory_cmd.rs` |
| `SessionState.keep_from` (o corte é observável na porta) | `crates/ktu-port/src/protocol.rs` |
| Corte restaurado do *checkpoint* ao retomar | `crates/ktu-cli/src/hub.rs` |
| Ctrl-C cancela o turno (SIGINT → porta `cancel`) | `crates/ktu-cli/src/repl.rs` (`tokio::signal`) |

**Ctrl-C:** uma tarefa de borda lê `tokio::signal::ctrl_c()` e chama `hub.cancel(id)` — o
processo **não** morre e o turno fecha com `StopReason::Cancelled`. O id é lido a cada sinal
porque `/resume` pode mudar de sessão. (`tokio` ganhou a *feature* `signal`.)

**Bug corrigido pelo caminho:** o `/compact` gravava o *checkpoint* com o id da sessão do
`RunConfig` (o do *front*), não o id da sessão do actor — `session_store::save_checkpoint_for`
passa a receber o id.

**Tab — adiada (com razão):** completar `/` e caminhos exige *raw mode* (`crossterm`) e, com
ele, **gestão do terminal em volta dos logs do turno** (o REPL escreve em `stderr` durante o
turno: transcrição, *steering*, *hooks*). Não é um `if`: é um editor de linha, com eco,
*backspace*, histórico e redesenho. Custo alto, valor baixo face ao risco — o gatilho continua a
ser «uso interactivo real a pedir o atalho».

## Notas de execução (skills Rust)

> Ver [NOTAS-RUST](./NOTAS-RUST.md).
- **N9:** sinais sem `unsafe` — handler da borda, `CancelToken` existente.
- **N10:** *raw mode* **só** em TTY (o `pipe` é o caminho dos gates).
- **N8:** os novos `Cmd`/`Slash` são variantes de enum, não bools.
