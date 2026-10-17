# Admissão durável de prompts (inbox) + `steer`/`queue`

- **Estado:** candidata (do `opencode`) — **incorporar**.
- **Origem:** `REF/opencode/specs/v2/session.md`,
  `packages/core/src/session/inbox.ts`; `wiki/revisao-opencode-profunda.md` §1.2.
- **Âncora:** `crates/ktu-core/src/session/mod.rs`, `crates/ktu-port/`.

## O que é
`prompt(...)` publica um **facto durável** de enfileiramento cuja projecção insere uma linha de
inbox **antes** da execução; o item fica **fora do histórico model-visible** até ser entregue.
Modos: **`steer`** (default, entregue no próximo *Safe Step Boundary*) e **`queue`** (pendente
enquanto a sessão pode continuar). Admissão **idempotente** (reusar ID: o primeiro vence).
Compactação e *move* são **itens de controlo**.

## Porque interessa
O ktu tem *steering* ([42](./steering-followup.md)) e uma porta com `Command`, mas **não** um
modelo de **admissão durável** com entrega em fronteiras seguras e idempotência. É **kernel** (o
log é a fonte) e robustez (uma mensagem nunca se perde a meio do turno).

## O que o ktu tem hoje
`Command` na porta + steering; sem inbox durável.

## Pré-condições para reconsiderar
- Mensagens que chegam durante um turno (TUI/`serve`).

## Custo e risco
- **Esforço:** médio-alto (inbox + projecção atómica + fronteiras).
- **Risco de não fazer:** mensagens perdidas/reordenadas.
- **Risco de fazer:** tocar no log — preservar `Model-visible ⟺ logged`; idempotência por ID.

## Decisão pendente
Promover. Admissão durável fora do histórico; entrega `steer`/`queue` em fronteira segura;
idempotência; itens de controlo (compactação/move) com identidade própria.
