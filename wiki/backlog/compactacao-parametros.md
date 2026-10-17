# Compactação parametrizada + resumo de branch

- **Estado:** candidata (do `pi`) — **avaliar** (barato).
- **Origem:** `REF/pi/packages/coding-agent/docs/compaction.md`; `wiki/revisao-pi.md` §2.7.
- **Âncora:** `crates/ktu-core/src/compact.rs`, `crates/ktu-core/src/budget.rs`.

## O que é
Gatilho explícito `contextTokens > contextWindow - reserveTokens` (reserve 16 k),
retenção `keepRecentTokens` (20 k), **resumo iterativo** (o resumo anterior entra no
próximo) e **resumo de branch** ao mudar de caminho na árvore de sessão.

## Porque interessa
Enriquece a compactação do ktu (relevante para **Q4** do gate). `reserveTokens`/
`keepRecentTokens` são parâmetros simples e explícitos; o resumo iterativo melhora
resumos sucessivos.

## O que o ktu tem hoje
Compactação **aditiva** (resumo + cru preservado + âncora verificável), com dono único do
tecto (`BudgetGate`, I14). Passou o gate, mas sem parâmetros finos nem resumo iterativo.

## Pré-condições para reconsiderar
- Medição: Q3/Q4 antes/depois. (O resumo de branch depende de `session-tree`.)

## Custo e risco
- **Esforço:** baixo-médio (parâmetros + resumo iterativo; branch só com árvore).
- **Risco de não fazer:** baixo.
- **Risco de fazer:** mexer no **caminho crítico do gate** — só com medição.

## Decisão pendente
Promover os **parâmetros** (`reserveTokens`/`keepRecentTokens`) e o **resumo iterativo**
independentemente da árvore; o resumo de branch fica para quando houver `session-tree`.
