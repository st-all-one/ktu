# `SYSTEM.md` / `APPEND_SYSTEM.md`

- **Estado:** candidata (do `pi`) — **incorporar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/configuration.md`;
  `wiki/revisao-pi-profunda.md` §1.5.
- **Âncora:** `crates/ktu-core/src/prompt.rs`.

## O que é
O prompt de sistema pode ser **substituído** (`SYSTEM.md`) ou **estendido** (`APPEND_SYSTEM.md`),
a nível **utilizador** (`<agent-dir>/`) e **projecto** (`.pi/`); o ficheiro de projecto **tem
precedência** (homónimos não se combinam).

## Porque interessa
O ktu compõe o sistema, mas não deixa o utilizador substituí-lo nem acrescentar-lhe instruções
persistentes. São **dois ficheiros** (dado) que dão controlo sem tocar no código — e o
`APPEND_SYSTEM` é o primo estável das instruções persistentes (MOIM, [79](./memoria-de-trabalho.md)).

## O que o ktu tem hoje
`prompt.rs` compõe sistema + `AGENTS.md`; sem override/append.

## Pré-condições para reconsiderar
- Necessidade de substituir ou estender o sistema por projecto/utilizador.

## Custo e risco
- **Esforço:** baixo (dois ficheiros + precedência).
- **Risco de não fazer:** instruções presas no código.
- **Risco de fazer:** prompt maior / sistema quebrado — caps e precedência explícita; fail-open
  (ausência = nada).

## Decisão pendente
Promover. `SYSTEM.md` substitui, `APPEND_SYSTEM.md` acrescenta; projecto > utilizador; fail-open;
teste de precedência.
