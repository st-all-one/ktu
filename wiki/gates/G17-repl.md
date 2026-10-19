# G17 — REPL interactivo (comandos `/` e Ctrl-C)

- **Veredicto:** ✅ **verde** (10/10) · **Artefactos:** `artifacts/G17/` · **Data:** 2026-10-18
- **Item:** [F2](../implementation_phase3/F2-repl-interactivo.md) ·
  **Critério:** [GATES da Fase 3](../implementation_phase3/GATES.md) §G17.

## Cenário

O REPL `ktu session` com os comandos `/` que faltavam e um Ctrl-C a meio de um turno (tornado
lento por um plugin `observe` de 2 s — sem inferência e sem depender do *timing* do provider).

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | a sessão tem log (pré-condição dos comandos de gestão) | exit 0 |
| 2 | `/help` lista `/compact`, `/fork`, `/memory`, `/resume` | presentes |
| 3 | `/memory` delega em `ktu memory list` | nota listada |
| 4 | `/fork` copia a sessão | **byte-a-byte** igual |
| 5 | `/resume` muda de sessão | o turno seguinte escreve no log novo |
| 6 | `/compact` grava o **mesmo corte** que `--compact` | `keep_from` igual nos dois |
| 7 | `/compact` não altera o log cru | 3 turnos intactos |
| 8 | Ctrl-C **não mata** o processo | exit 0 |
| 9 | o turno cancelado fecha com `StopReason::Cancelled` | `reason=cancelled` |
| 10 | I1/I2 limpos depois do Ctrl-C | `ok` |

## Artefactos

`s.jsonl` (o log da sessão, com o turno cancelado) · `c2.checkpoint.json` (o corte do
`/compact`) · `help.txt` (a ajuda do REPL).

## Desvios (registados)

- **A Tab (completar `/` e caminhos) fica adiada.** Exige *raw mode* (`crossterm`) e, com ele,
  **gestão do terminal em volta dos logs do turno**: o REPL escreve em `stderr` durante o turno
  (transcrição, *steering*, hooks) enquanto o utilizador escreve — em *raw mode* isso exige
  guardar/restaurar o cursor e redesenhar a linha. Não é um `if`: é um editor de linha. O
  gatilho continua a ser «uso interactivo real a pedir o atalho».
- **Sem regressão no caminho *pipe*:** o REPL continua a ler linha a linha quando `stdin` não é
  TTY — o **G12** (log byte-idêntico `run`/REPL/crate) é o teste que o trava, e continua verde.
- O `/resume` **não** tem verbo na porta: é o REPL a compor `create_session` + `subscribe` +
  `state` (três comandos da porta), sem lógica de conversa (M4).
