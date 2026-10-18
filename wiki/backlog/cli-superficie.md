# Superfície do CLI (áreas, `prime`, `help`, `self`, `--params`, `--batch`)

- **Estado:** **nova** (propósito: uso #2 e #4; prioridade CLI) — **incorporar**.
- **Origem:** [CLI e configuração](../cli-e-config.md) §2; modelo: `kd` (knudge) no host.
- **Âncora:** `crates/ktu-cli/src/cli.rs`, `crates/ktu-cli/src/main.rs`.

## O que é
A superfície do CLI reorganizada **por áreas** (como o `kd`), bem documentada e previsível:

- **Áreas:** CONVERSA (`(default)`/`session`/`run`), CONHECIMENTO (`memory`/`guardrails`/`plugins`),
  PROJECTO (`init`/`config`/`doctor`/`mode`), SISTEMA (`self`/`providers`/`prime`/`help`),
  PORTA (`serve`/`tui`).
- **Flags globais:** `--json`, `--log-level`, `--quiet`, `--root`, `--session`, `--ephemeral`.
- **`prime`:** protocolo do agente, **byte-idêntico por versão** (liga a `71`).
- **`self`:** `setup`/`completions`/`upgrade`/`version` (o `upgrade` reusa o `install.sh` com checksum).
- **`help [verbo]`:** ajuda por área, com rodapé (CICLO / ÂNCORAS / veja `ktu help <verbo>`).
- **stdin/heredoc:** `ktu -`, `ktu <<EOF`, e "vazio + pipe".
- **`--params <JSON>`:** objeto com **tudo** (`-` lê stdin); as flags são **açúcar** sobre ele.
- **`--batch <JSONL>`:** lote de `--params`; **paralelo por omissão** (N efémeros, `--concurrency`),
  **`--sequential`** = uma sessão, N turnos; saída JSONL ordenada pelo índice.
- **Envelope JSON + códigos de saída** (como o `kd`): 2 invalid, 3 not_found, 4 conflict, 5 io,
  6 timeout, 7 config, 8 schema, 70 internal.

## Porque interessa
É o que torna o ktu **utilizável em pipelines** (uso #2) e **controlável pelo humano** (uso #4):
`--params`/`--batch` dão o contrato de máquina; as áreas dão a legibilidade. É o **modelo do `kd`**
adaptado a um **agente** (os verbos `ask`/`write` do `kd` são, no ktu, **tools do modelo** — não
comandos do utilizador).

## O que o ktu tem hoje
Subcomandos planos (`run`/`serve`/`tui`/`init`/`mode`/`memory`/`doctor`/`plugins`/`guardrails`/
`trash`/`providers`) sem áreas, sem `--params`/`--batch`/`prime`/`self`, sem stdin documentado.

## Pré-condições para reconsiderar
- Nenhuma — é o front primário escolhido.

## Custo e risco
- **Esforço:** médio (reorganizar o `clap`; `--params`/`--batch` são camadas finas).
- **Risco de não fazer:** CLI difícil de usar em pipelines.
- **Risco de fazer:** `--params` a duplicar as flags — mitigar com **precedência declarada**
  (`flag` > `--params` > config > default) e um **schema versionado**.

## Decisão pendente
Incorporar **primeiro** (é o esqueleto do REPL e da config). Ordem: superfície → REPL → efémero →
config-global.
