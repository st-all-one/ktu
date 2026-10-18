# Deteção de eco

- **Estado:** **implementado** (H5; `wiki/implementation_phase2/H5-fiabilidade-loop.md`).
- **Origem:** `REF/katu/wiki/_ref/plan/OPTIMIZATION_PLAN.md` (WL2-3, KS-G5);
  `wiki/revisao-katu.md` §3.3.
- **Âncora:** `crates/ktu-core/src/agent/mod.rs`.

## O que é
Comparar o texto final do modelo com os `delta` recentes (trim + colapso de whitespace;
igualdade/prefixo longo). Eco → *nudge* + **1 retry**; esgotado o orçamento, o eco é **substituído**
por nota visível — **nunca aceite** (G5).

## Porque interessa
Modelos fracos (locais) devolvem o prompt ou a resposta anterior como resposta final; sem deteção,
o turno "termina com sucesso" com lixo. O ktu tem `StopReason` e estagnação, mas não eco.

## O que o ktu tem hoje
`echo` só no `EchoProvider` de teste; deteção de estagnação em `agent/support.rs` (repeats), que
não cobre o eco do próprio texto.

## Pré-condições para reconsiderar
- Evidência de eco num modelo em uso.

## Custo e risco
- **Esforço:** baixo (comparação no fecho do turno + 1 retry + nota).
- **Risco de não fazer:** resposta-lixo aceite.
- **Risco de fazer:** falso positivo (resposta legítima igual a um delta) — usar igualdade/prefixo
  longo + controlo negativo no teste.

## Decisão pendente
Promover. Eco nunca aceite; nudge + 1 retry; substituição por nota visível; teste com controlo
negativo (resposta legítima não é marcada).
