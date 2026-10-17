# knudge — ciclo de vida (confiança Beta, shelf-life FSRS, decay, demolição)

- **Estado:** candidata (do `knudge` 0.6.1) — **avaliar**.
- **Origem:** `crates/knudge/wiki/integration/09_ciclo_de_vida.md`,
  `crates/knudge/wiki/specs/ciclo-de-vida.md`; `wiki/revisao-knudge.md` §6.2.
- **Âncora:** `crates/ktu-memory/src/report.rs`, `crates/ktu-core/src/memory.rs`.

## O que é
Tudo **derivado** de notas/eventos (nada armazenado):
- **confiança Beta-Bernoulli** (D189) — média posterior + limite inferior 95 %;
- **shelf-life** por curva de esquecimento (FSRS-like, D190) — cada `outcome` estende o prazo;
- **decay de âncoras** e **drift** persistido (D203) — `1 - fração de âncoras válidas`;
- **retenção** por `limiar^(t/ttl)`;
- **clusters/comunidades** (D128/D193) e **demolição soft** por política (D45/D177/D208);
- **renovação por citação** (D154).

## Porque interessa
O ktu não tem curadoria automática: as notas vivem para sempre. O ciclo de vida dá **prioridade**
(confiança no ranking) e **retenção** (expirar o que envelhece) — sem apagar (soft). Liga a
[6 — contexto-rate-distortion](./contexto-rate-distortion.md) (seleção de contexto).

## O que o ktu tem hoje
Recall por similaridade; sem confiança/retenção/demolição.

## Pré-condições para reconsiderar
- Corpus que cresça ao ponto de precisar de priorização/retenção.

## Custo e risco
- **Esforço:** médio (usar `lifecycle::*`; alimentar `outcome`/uso).
- **Risco de não fazer:** corpus sem prioridade.
- **Risco de fazer:** complexidade — tudo derivado e determinístico; nunca armazenar confiança.

## Decisão pendente
Adiar. Se entrar, começar pela **confiança** (alimenta o ranking) e pela **retenção soft**; a
demolição só por acção explícita (`forget`), nunca automática.
