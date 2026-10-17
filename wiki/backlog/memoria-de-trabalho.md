# MOIM — memória de trabalho (Turn Context)

- **Estado:** candidata (do `goose`) — **incorporar**.
- **Origem:** `REF/goose/crates/goose/src/agents/moim.rs`; `wiki/revisao-goose-profunda.md` §1.1.
- **Âncora:** `crates/ktu-core/src/prompt.rs`, `crates/ktu-core/src/budget.rs`.

## O que é
O goose injecta, **a cada turno**, um bloco `<turn-context>` na memória de trabalho do modelo
(MOIM = *Model-Observed Internal Memory*): hora actual, directório de trabalho, estado de
compactação, **orçamento de turno** e contexto das extensões. O bloco é **regenerado por
turno**; o modelo é instruído a usá-lo para se orientar sem o confundir com o pedido. As
**instruções persistentes** são injectadas no mesmo sítio e **re-lidas a cada turno** (tecto
64 KB), pelo que não são "esquecidas". Só liga acima de ~32 k de contexto.

## Porque interessa
O ktu compõe o sistema a cada turno, mas **não tem** um bloco de estado sempre presente. É
**dado puro** regenerado do log; dá consciência situacional e casa com o `prompt_state` que o
katu mediu (Q-04) e com o `pin_unit` do ktu.

## O que o ktu tem hoje
`prompt.rs` (sistema + AGENTS.md) + `budget.rs` (`pin_last_user`); sem bloco de estado.

## Pré-condições para reconsiderar
- Um consumidor que beneficie de estado situacional (TUI/`run`).

## Custo e risco
- **Esforço:** baixo (bloco de turno + orçamento).
- **Risco de não fazer:** modelo sem orientação temporal/espacial.
- **Risco de fazer:** gastar tokens — manter curto, omitir em contexto pequeno, determinístico.

## Decisão pendente
Promover. Bloco curto e determinístico (hora/cwd/modo/orçamento), omitido abaixo de um limiar;
instruções persistentes re-lidas por turno com tecto.
