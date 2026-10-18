# CLI REPL — `ktu session` (conversa longa no terminal)

- **Estado:** **implementado** (H4; propósito: uso #3) — REPL `ktu session` in-process sobre a
  porta; `ls|rm|fork` para gestão; M4 verificado. Ctrl-C/Tab e os `/` sem verbo na porta ficam
  adiados (A5).
- **Origem:** [CLI e configuração](../cli-e-config.md) §2.3.
- **Âncora:** `crates/ktu-cli/src/run.rs` (um turno), `crates/ktu-cli/src/tui.rs` (cliente).

## O que é
O **front primário** do ktu: `ktu session [id]` abre um **REPL** no terminal — multi-turno, mesma
sessão, sem servidor. É o par CLI do `ktu tui` (que passa a **secundário**).

**UX (o requisito é "agradável e extremamente funcional"):**
- prompt com **id + modo** (`plan`/`build`) + estado (`●` a pensar);
- **streaming** dos eventos do turno (texto, tool calls, resultados);
- comandos `/`: `/plan`, `/mode`, `/compact`, `/resume`, `/fork`, `/undo`, `/redo`, `/memory`,
  `/help`, `/quit` — **todos comandos da porta**;
- **multi-linha** (linha em branco ou `\` final), **histórico** persistido;
- **Ctrl-C** cancela o turno, **Ctrl-D** sai, **Tab** completa `/` e caminhos;
- **cor automática** (off quando não é TTY → pipe-friendly).

`ktu session ls|rm|fork|resume` gere sessões (a área `session` é abrir **e** gerir).

## Porque interessa
É o **uso #3** ("longa conversa, vários turnos, **especialmente no cli**"). Hoje `ktu run` corre um
turno e sai; o `tui` exige um `serve` à parte.

## O que o ktu tem hoje
`ktu run` (one-shot) + `ktu tui` (cliente de `serve`). Sem REPL.

## Pré-condições para reconsiderar
- Nenhuma — é o front primário escolhido.

## Custo e risco
- **Esforço:** baixo-médio (loop de leitura + `hub`/actor em processo).
- **Risco de não fazer:** o CLI não serve para conversar.
- **Risco de fazer:** lógica própria no REPL — **proibido** (M4): o REPL fala à **mesma porta**;
  os `/` são comandos da porta. Teste: mesmo prompt por `run` e por REPL ⇒ mesmo log (I3/I6).

## Decisão pendente
Incorporar **primeiro no Eixo 0**. Ordem sugerida: `cli-superficie` (o esqueleto) → `cli-repl`.
