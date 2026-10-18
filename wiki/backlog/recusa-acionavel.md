# Recusa acionável (`remedy` por regra)

- **Estado:** **implementado** (H5) — remédio **estrutural** por `DenyReason`; o remédio *por regra* fica adiado (A6).
- **Origem:** `REF/katu/wiki/_ref/plan/OPTIMIZATION_PLAN.md` (Q-07/Q-08);
  `wiki/revisao-katu.md` §3.5.
- **Âncora:** `crates/ktu-core/src/guardrails.rs`, `crates/ktu-core/src/guard.rs`.

## O que é
Cada regra negada devolve um **`remedy`** (dado, não código) — o que fazer a seguir. O `edit`
devolve as **âncoras únicas mais próximas**. *"Um erro que ensina vale um turno."*

## Porque interessa
Quando um guardrail nega uma ação, o modelo não sabe o que fazer e repete (até o guard cortar). Um
`remedy` por regra transforma a recusa em próximo passo — menos turnos falhados.

## O que o ktu tem hoje
`guardrails.rs` + `guard.rs` (recusa fail-closed); `remedy` = 0 hits.

## Pré-condições para reconsiderar
- Regras com remediação conhecida (ex.: escrever fora do root → "usa um caminho sob o root").

## Custo e risco
- **Esforço:** baixo (campo `remedy` por regra, dado).
- **Risco de não fazer:** turnos repetidos a bater no mesmo muro.
- **Risco de fazer:** `remedy` errado induz o modelo — manter curto e verificado por teste.

## Decisão pendente
Promover. `remedy` como **dado** por regra; recusa sem `remedy` é defeito (trava). O `edit` devolve
as âncoras únicas mais próximas.
