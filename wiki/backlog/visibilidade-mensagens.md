# Visibilidade das mensagens (`User` vs `Agent`)

- **Estado:** candidata (do `katu`, KS-G3) — **incorporar**.
- **Origem:** `REF/katu/wiki/_ref/plan/KERNEL_SURFACE.md` (KS-G3); `wiki/revisao-katu.md` §3.7.
- **Âncora:** `crates/ktu-core/src/domain.rs`, `crates/ktu-core/src/session/mod.rs`.

## O que é
`Visibility::{User, Agent}` no log/projeção: o *nudge* interno chega ao **modelo** mas **não** à
transcrição do utilizador.

## Porque interessa
O ktu injecta *nudges* (ex.: lembrete de memória, aviso de truncagem). Sem visibilidade, ou o nudge
não chega ao modelo, ou polui o que o utilizador vê. A distinção é barata e mantém a projeção fiel
ao log.

## O que o ktu tem hoje
`visibility` = 0 hits; o log tem mensagens sem distinção de origem.

## Pré-condições para reconsiderar
- Um nudge que deva chegar ao modelo sem aparecer na transcrição.

## Custo e risco
- **Esforço:** baixo (`Visibility` no `Event`/`Message`, `serde default`; projeção inclui `Agent`,
  transcrição salta-o).
- **Risco de não fazer:** nudge polui a transcrição ou não chega.
- **Risco de fazer:** mensagem marcada errada desaparece da vista — `default = User` (fail-safe).

## Decisão pendente
Promover. `Visibility::{User, Agent}` com `default = User`; `derive_messages` inclui ambos; a
transcrição salta `Agent`; testes em projeção e transcrição.
