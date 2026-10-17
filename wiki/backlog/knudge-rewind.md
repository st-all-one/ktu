# knudge — `handoff::rewind` (contexto de sessão com orçamento)

- **Estado:** candidata (do `knudge` 0.6.1) — **incorporar**.
- **Origem:** `crates/knudge/wiki/integration/06_tarefas_e_handoff.md`,
  `crates/knudge/wiki/specs/handoff.md`; `wiki/revisao-knudge.md` §6.1.
- **Âncora:** `crates/ktu-core/src/prompt.rs`, `crates/ktu-core/src/budget.rs`.

## O que é
`rewind` monta o **contexto de início de sessão** respeitando um **orçamento de tokens**
(`ceil(chars/4)`) e grava um **`context_id`** endereçável: retomar com `resume: Some(id)` reproduz
**bytes idênticos** (D88). Modos: `Manifest` (~30 tokens: contadores + recentes + dirty),
`Scope(id)` (working set de um domínio), `Files(paths)` (notas ancoradas a ficheiros) e `Auto`
(deriva dos ficheiros alterados).

## Porque interessa
O ktu monta o prompt com `prompt.rs` + `budget.rs` (com `pin_last_user`). O `rewind` é o primo
directo, mas com **vizinhança** (working set, escopo, ficheiros alterados), **orçamento explícito**
e **reprodutibilidade** por `context_id`. Substitui a montagem manual de contexto no início de
sessão.

## O que o ktu tem hoje
`prompt.rs` (sistema + AGENTS.md) + `budget.rs` (tecto + pin); sem working set nem `context_id`.

## Pré-condições para reconsiderar
- Necessidade de retomar contexto entre sessões com orçamento.

## Custo e risco
- **Esforço:** baixo-médio (usar `rewind` no adaptador/borda).
- **Risco de não fazer:** contexto inicial pobre ou a estourar.
- **Risco de fazer:** acoplar o prompt ao knudge — manter o `rewind` **atrás da porta** (o kernel
  não conhece o knudge).

## Decisão pendente
Promover. `rewind` exposto como dado (contexto + `context_id`); a montagem do prompt do ktu usa o
resultado; teste de reprodutibilidade por `context_id`.
