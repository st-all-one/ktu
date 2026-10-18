# H0 — Superfície do CLI

> **Issue:** `task_00qn6s14` · **Âncora:** [cli-superficie](../backlog/cli-superficie.md) ·
> **Desenho:** [cli-e-config](../cli-e-config.md) §2 · **Pré-requisitos:** nenhum.

## Objetivo
Reorganizar o CLI do ktu **por áreas** (modelo do `kd`), com contrato de máquina (`--params`,
`--batch`), `prime`/`self`/`help` e stdin/heredoc. É o **esqueleto** do REPL e da config.

## Passos
1. **Áreas no `--help`:** CONVERSA (`(default)`/`session`/`run`) · CONHECIMENTO (`memory`/
   `guardrails`/`plugins`) · PROJECTO (`init`/`config`/`doctor`/`mode`) · SISTEMA (`self`/
   `providers`/`prime`/`help`) · PORTA (`serve`/`tui`).
2. **Flags globais:** `--json`, `--log-level`, `--quiet`, `--root`, `--session`, `--ephemeral`.
3. **`--params <JSON>`** — objeto com tudo; `-` lê stdin; **schema versionado** (`ktu self schema`);
   precedência `flag > --params > config > default`.
4. **`--batch <JSONL>`** — lote de `--params`; **paralelo (default)** com `--concurrency N`;
   `--sequential` = uma sessão, turnos em ordem; saída ordenada pelo índice.
5. **`prime`/`self`/`help`:** `ktu prime` (byte-idêntico por versão, liga `71`); `ktu self
   setup|completions|upgrade|version`; `ktu help [verbo]` com rodapé.
6. **stdin/heredoc** (`ktu -`, pipe) + **envelope JSON** + **códigos de saída** documentados.

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_003n4ebj` | Áreas + flags globais |
| `task_00rtiog5` | `--params` + schema + precedência |
| `task_00sl9jsg` | `--batch` (paralelo/sequencial) |
| `task_01l8s4ev` | `prime` + `self` + `help` |
| `task_012xuf1i` | stdin/heredoc + envelope + exit codes |

## Gate / testes
- `ktu --help` mostra as 5 áreas; `ktu help <verbo>` por área.
- `--params`/`--batch` com `-` (stdin) e `--json`; envelope `{success,command,data?,error,warnings?}`.
- `--batch` paralelo: N efémeros, resultado ordenado pelo índice; `--sequential`: uma sessão.
- `ktu prime` byte-idêntico em duas execuções (golden).

## Riscos
- `--params` a duplicar flags → **precedência declarada** + schema.
- `--batch` virar scheduler → **reusa** o loop + pool limitado; nada novo.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **N8:** flags de `clap` (bool) → **enums** onde representam modo (`max-struct-bools=1`).
- **N2:** `--params`/`--batch` sem `unwrap`; parse com `serde` tipado + `Error` accionável.
- **Determinismo:** a saída de `--batch` é **ordenada pelo índice** (sort estável — N9); nada de
  `HashMap` no despacho (N1).
- **1.99:** `format_into`/`NumBuffer` para formatar exit-codes/logs de alto débito (§2).
- **Gate:** contribui para **G12** (independência do caminho) e **G13** (pipeline).
