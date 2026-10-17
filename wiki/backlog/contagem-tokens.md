# Contagem de tokens precisa

- **Estado:** candidata (do `goose`) — a avaliar.
- **Origem:** `REF/goose/crates/goose/src/token_counter.rs`; `wiki/revisao-goose.md` §2.5.
- **Âncora:** `crates/ktu-core/src/budget.rs`.

## O que é
Contagem de tokens fiel (tokenizer real) em vez da estimativa **≈4 bytes/token**, incluindo
o custo dos esquemas de tools.

## Porque interessa
O `BudgetGate` decide compactar/pinar com base numa estimativa grosseira. Uma contagem mais
fiel melhora essa decisão — **se** houver sintoma.

## O que o ktu tem hoje
`BudgetGate::estimate` (≈4 bytes/token), determinístico e sem dependências.

## Pré-condições para reconsiderar
- Sintoma medido: orçamento a errar (compactar cedo/tarde, contexto a estourar).
- Um tokenizer adequado ao **modelo local** (o `llama.cpp` usa outro tokenizer, não o do
  GPT).

## Custo e risco
- **Esforço:** médio (dependência de tokenização + cache).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** dependência nova e mismatch de tokenizer entre modelos.

## Decisão pendente
Manter a estimativa até haver sintoma. Se entrar, preferir o tokenizer do **próprio
modelo** (via `llama-server`) a um tokenizer de terceiros.
