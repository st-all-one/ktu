# Dossiê de pendências

- **Estado:** **incorporada** (H11) — `wiki/pendencias.md`: estados ⛔🚫⏳🔴⚪ × eixos Q/P/S,
  âncora no código, critério de reabertura por linha e rejeições com número; verificado contra o
  código (nota de verificação na §1).
- **Origem:** `REF/katu/wiki/crates/DOSSIE_PENDENCIAS.md`; `wiki/revisao-katu.md` §1.2.
- **Âncora:** `STAGING.md`, `wiki/`.

## O que é
Um levantamento **verificado no código** do que falta, classificado por estado
(`⛔ Deferido` · `🚫 Rejeitado` · `⏳ Opt-in` · `🔴 Lacuna` · `⚪ Formalismo`) e por eixo
(**Q** qualidade · **P** performance · **S** simplificação), com o **número** que justificou cada
rejeição.

## Porque interessa
O `STAGING.md` diz o que **está feito**; falta o **inverso** — o que falta, porquê, e o que foi
rejeitado com que número. É o antídoto do "fizemos tudo".

## O que o ktu tem hoje
`STAGING.md` (fases concluídas); `wiki/backlog/` (features externas). Falta o espelho interno.

## Pré-condições para reconsiderar
- Fim de fase (momento natural para levantar pendências).

## Custo e risco
- **Esforço:** baixo (`wiki/pendencias.md`, atualizado por fase).
- **Risco de não fazer:** lacunas esquecidas; rejeições repetidas.
- **Risco de fazer:** documento a envelhecer — verificar contra o código a cada revisão.

## Decisão pendente
Promover. Uma página, estados + eixos, cada rejeição com o número e a fonte. Verificar no código
antes de publicar.
