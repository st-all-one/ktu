# H4 — REPL `ktu session`

> **Issue:** `task_010l8e4e` · **Âncora:** [cli-repl](../backlog/cli-repl.md) ·
> **Desenho:** [cli-e-config](../cli-e-config.md) §2.3 · **Pré-requisitos:** H0, H3.

## Objetivo
O **front primário** de conversa longa: `ktu session [id]` — REPL multi-turno no terminal, sem
servidor, com UX cuidada. O `ktu tui` passa a **secundário**.

## Passos
1. **Loop de leitura** multi-turno na mesma sessão + **histórico** persistido + **streaming** dos
   eventos do turno (texto, tool calls, resultados).
2. **Comandos `/`** — `/plan`, `/mode`, `/compact`, `/resume`, `/fork`, `/undo`, `/redo`, `/memory`,
   `/help`, `/quit` — **todos comandos da porta** (nenhuma lógica no REPL).
3. **UX** — multi-linha (linha em branco ou `\`), **Ctrl-C** cancela o turno, **Ctrl-D** sai,
   **Tab** completa `/` e caminhos, **cor automática** (off quando não é TTY → pipe-friendly).
4. **Gestão** — `ktu session ls|rm|fork|resume`; a área `session` é abrir **e** gerir.

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_0064js3u` | Loop multi-turno + histórico + streaming |
| `task_01qzufzi` | Comandos `/` via porta |
| `task_00cw2avd` | UX (multi-linha, Ctrl-C/D, Tab, cor) |
| `task_0133oljm` | `session ls/rm/fork/resume` + teste M4 |

## Gate / testes
- **M4 — dois caminhos, uma sessão:** mesmo prompt por `run` e por REPL ⇒ **mesmo log** (I3/I6).
- Os comandos `/` mapeiam em comandos da porta (sem código próprio).
- Não-TTY: sem cor; saída limpa para pipe.

## Riscos
- Lógica própria no REPL (viola M4) → **proibido**; só fala à porta.
- Duplicar a TUI → a TUI é cliente; o REPL é directo.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **M4/N5:** o REPL **não** tem lógica própria — fala à porta; os `/` são comandos da porta.
- **N2:** leitura de `stdin` sem `unwrap`; EOF/Ctrl-D tratados como valor.
- **Determinismo:** histórico via porta `Fs`; ordem estável; detecção de TTY por porta (não
  `std::io::IsTerminal` no kernel).
- **12-async:** **não** usar `tokio::select!` ad hoc — usar o `CancellationToken` do ktu.
- **Gate:** **G12** (dois caminhos, uma sessão) e **G10** (cadeia longa).

## Execução (concluída — âmbito reduzido)
- **Implementado:** `ktu session [id]` = REPL multi-turno **in-process** sobre a porta `Host`
  (`SessionHub`): streaming dos eventos, multi-linha (linha com `\` final), Ctrl-D sai, cor
  automática (off fora de TTY), histórico em `.ktu/sessions/<id>.history`. Comandos `/help`,
  `/quit`, `/plan`, `/build`, `/mode`, `/cancel` — **todos** mapeiam verbos da porta (M4). Gestão:
  `ktu session ls|rm|fork`.
- **Verificado:** **M4** — `ktu run "ola" --session a` e o REPL na sessão `b` produzem
  `a.jsonl == b.jsonl` (teste `tests/session.rs`). `stdout` = respostas; `stderr` = transcrição e
  *chrome*.
- **Ficheiros:** `render.rs` (extraído da `tui`), `repl.rs`, `session_cmd.rs`, `actions.rs`,
  `run_cmd.rs` (split de `main.rs`/`cli.rs` para o limite de 400 linhas).
- **Adiado (A5):** Ctrl-C (cancelar turno) e Tab (completar) — exigem *raw mode*/`signal`; e os
  `/compact|/resume|/fork|/undo|/redo|/memory` — **não têm verbo na porta** (o REPL não inventa
  lógica). **Tarefa kd:** `task_00dbywbk`.
- **Gate:** `make check` + `typos` + `machete` verdes.
