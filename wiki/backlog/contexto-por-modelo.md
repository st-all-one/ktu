# Limite de contexto por modelo

- **Estado:** candidata (do `goose`) — a avaliar.
- **Origem:** `REF/goose/crates/goose/src/{context_limit,model_catalog}.rs`;
  `wiki/revisao-goose.md` §2.6.
- **Âncora:** `crates/ktu-core/src/budget.rs`, `crates/ktu-cli/src/providers.rs`.

## O que é
Derivar o tecto de contexto do **modelo** (ex.: `n_ctx` do `llama-server`) e disparar a
compactação a um limiar (o goose usa **0.8** da janela), em vez de um `max_tokens` fixo.

## Porque interessa
O ktu já lê `n_ctx` no `ktu providers doctor`; usar esse valor como tecto é uma melhoria
**pequena e determinística** do `BudgetGate`.

## O que o ktu tem hoje
`BudgetGate::new(max_tokens)` fixo, definido na borda.

## Pré-condições para reconsiderar
- Providers com janelas diferentes em uso.
- Um limiar de compactação definido (0.8? outro?).

## Custo e risco
- **Esforço:** baixo (ler o limite + derivar o tecto + limiar).
- **Risco de não fazer:** contexto subutilizado ou a estourar.
- **Risco de fazer (YAGNI):** catalogar modelos sem necessidade.

## Decisão pendente
Promover quando houver ≥2 janelas de contexto distintas; começar pelo `n_ctx` do
`llama-server` (já disponível) e um limiar explícito.
