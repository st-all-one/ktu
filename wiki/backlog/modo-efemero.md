# Modo efémero (default) + lock de turno

- **Estado:** **implementado** (H3; propósito: uso #2) — feito.
- **Origem:** [CLI e configuração](../cli-e-config.md) §1.3/§1.4.
- **Âncora:** `crates/ktu-cli/src/cli.rs` (`Run { session }`), `crates/ktu-cli/src/session_store.rs`.

## O que é
1. **Efêmero por omissão:** `ktu` (nu) e `ktu <prompt>` correm **1 turno sem persistir** sessão.
   `ktu run --session <id>` persiste; `--resume` implica persistente. É a inversão "pipeline-first".
2. **Lock de turno por sessão** (`64`): `.ktu/sessions/<id>.lock` impede dois processos no **mesmo**
   id; o segundo espera ou recusa com `StopReason` claro.

## Porque interessa
É o **uso #2**: "CLI em pipelines, 1 turno, vários ktus independentes em paralelo". E o lock
protege o caso do §1.4: um **REPL de sessão** e um **turno efémero** a coexistir no mesmo projecto
(ids diferentes ⇒ sem colisão; recursos partilhados são append-only).

## O que o ktu tem hoje
`ktu run --session <id>` (persiste); default `default` (colidente); sem efémero; sem lock.

## Pré-condições para reconsiderar
- Nenhuma.

## Custo e risco
- **Esforço:** baixo (flag + lock por ficheiro).
- **Risco de não fazer:** corrupção de log em paralelo; impossível CI sem `--session` único.
- **Risco de fazer:** **mudança de comportamento** (hoje `run` persiste) — mitigar documentando no
  `prime`/`help`; `--session` explícito mantém o comportamento antigo.

## Decisão pendente
Incorporar. O lock é obrigatório assim que dois processos possam partilhar um `id`. Teste: dois
`run` concorrentes no mesmo `id` ⇒ um espera/recusa, log íntegro (I1/I2).
