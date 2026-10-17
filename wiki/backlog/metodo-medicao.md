# Método de medição (medir → A/B → adotar-ou-reverter)

- **Estado:** candidata (do `katu`) — **incorporar**.
- **Origem:** `REF/katu/wiki/_ref/plan/OPTIMIZATION_PLAN.md` §0, `bench/published.toml`;
  `wiki/revisao-katu.md` §1.3.
- **Âncora:** `wiki/benchmarks/`, `plan.md` §0, `crates/ktu-core/benches/core.rs`.

## O que é
Todo o item exige **fórmula + artefacto + teste que o trava**; **A/B** com recorte cru;
**adotar-ou-reverter** (≥ 20 % no alvo **ou** remoção de complexidade; senão reverter e **escrever
a rejeição com o número**). `bench/published.toml` publica cada número com uma **base de evidência**
(`measured`/`inferred`/`provider_reported`/`unpriced`).

## Porque interessa
O ktu já mede (`wiki/benchmarks/ktu-bench.json`) e já decidiu por medição (`decision_01e2183g`).
Falta formalizar: (a) a **base de evidência** de cada número; (b) a regra **adotar-ou-reverter**;
(c) o registo das **rejeições com número**.

## O que o ktu tem hoje
`make bench` + `wiki/benchmarks/`; sem coluna `basis`, sem regra de reversão, sem rejeições
registadas.

## Pré-condições para reconsiderar
- Nenhuma (é processo, não código).

## Custo e risco
- **Esforço:** baixo (coluna `basis` no artefacto + regra no `plan.md`).
- **Risco de não fazer:** números sem base; decisões por impressão.
- **Risco de fazer:** burocracia — mitigar com o artefacto como prova, não como formulário.

## Decisão pendente
Promover. `basis` obrigatória em cada métrica; rejeições registadas com o número; regra
adotar-ou-reverter no `plan.md`. **Um lint não substitui uma medição** (lição B-01 do katu).
