# `edit` multi-bloco atómico

- **Estado:** candidata (do `katu`, Q-07) — **incorporar**.
- **Origem:** `REF/katu/wiki/_ref/plan/OPTIMIZATION_PLAN.md` (Q-07); `wiki/revisao-katu.md` §3.11.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`.

## O que é
N edições numa só chamada, **atómico** (0 B gravados em falha) — medido no katu: 5 chamadas → 1
(**−80 %**), payload −19,9 %.

## Porque interessa
Cada edição numa chamada separada gasta um turno. Um `edit` multi-bloco atómico faz a mesma mudança
num só passo e nunca deixa o ficheiro a meio.

## O que o ktu tem hoje
O `edit` do agente faz uma edição por chamada.

## Pré-condições para reconsiderar
- Cenários de edição com várias partes no mesmo ficheiro.

## Custo e risco
- **Esforço:** baixo (lista de blocos + aplicação atómica + rollback).
- **Risco de não fazer:** mais turnos; ficheiros a meio em falha.
- **Risco de fazer:** ambiguidade de âncoras — exigir âncoras únicas; 0 B em falha (trava).

## Decisão pendente
Promover. Blocos com âncoras únicas, aplicação atómica (0 B gravados em falha), teste de atomicidade.
