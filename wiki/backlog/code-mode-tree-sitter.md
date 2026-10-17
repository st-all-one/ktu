# Code mode / tree-sitter

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** features `code-mode` e `tree-sitter` do crate `goose`; `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs` (grep/glob).

## O que é
**Parsing de código** com `tree-sitter` (multi-linguagem) para views estruturais
(outline/símbolos) e um "code mode" que gera/executa código em vez de tool-calls.

## Porque não entra
O ktu tem tools simples (grep/glob/read) que passaram o gate. `tree-sitter` acrescenta
gramáticas por linguagem (peso) e o "code mode" muda o modelo de execução. É optimização
para leitura de código, sem sintoma.

## Pré-condições para reconsiderar
- Sintoma medido: o agente a ler demasiado código (contexto/tokens) por falta de views
  estruturais.
- Um conjunto de linguagens-alvo fechado.

## Custo e risco
- **Esforço:** médio-alto (gramáticas + views).
- **Risco de não fazer:** leitura menos eficiente em repositórios grandes.
- **Risco de fazer (YAGNI):** gramáticas a manter sem sintoma.

## Decisão pendente
Adiar até haver número. Se entrar, atrás de feature (como no goose), não por omissão.
