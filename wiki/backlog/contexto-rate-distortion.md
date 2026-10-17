# Contexto por rate-distortion / submodular

- **Estado:** deferido.
- **Origem:** katu `context/select`, `wiki/_ref/plan/`.
- **Âncora:** `crates/ktu-core/src/budget.rs`, `crates/ktu-core/src/compact.rs`.

## O que é
Selecção de contexto como **problema de optimização**: escolher o subconjunto de
unidades (mensagens/tool results) que maximiza utilidade sob um orçamento de tokens —
por rate-distortion ou selecção submodular — em vez de uma política heurística
(suffix/prefix, pin da instrução).

## O que o ktu tem hoje
- `BudgetGate` com **dono único** do tecto (I14).
- **Pin da instrução corrente** (a última `User` nunca sai).
- Compactação **aditiva** (resumo + cru preservado) e checkpoint durável.
- Heurística simples que **passou o gate** (Q4 com `compacted=true`).

## Porque não entrou
A heurística simples atingiu a equivalência de grau no gate. A optimização de selecção
sem um sintoma medido (perda de qualidade por contexto) é prematura.

## Pré-condições para reconsiderar
- Um sintoma medido: quedas de qualidade atribuíveis à selecção de contexto.
- Uma métrica de utilidade por unidade (não trivial) e dados para a calibrar.

## Custo e risco
- **Esforço:** alto.
- **Risco de não fazer:** baixo (o gate já passa).
- **Risco de fazer (YAGNI):** complexidade sem ganho medido — o erro do katu.

## Decisão pendente
Só com sintoma medido no gate ou em uso real. Antes disso, manter a heurística.
