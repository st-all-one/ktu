# Resumo estruturado de compactação

- **Estado:** candidata (do `goose`) — a avaliar com medição.
- **Origem:** `REF/goose/crates/goose-context-management/src/structured.rs`;
  `wiki/revisao-goose.md` §2.4.
- **Âncora:** `crates/ktu-core/src/compact.rs`.

## O que é
Que a compactação produza um resumo **estruturado** com campos estáveis: `user_intent`,
`technical_concepts`, `files` (path + resumo + código-chave), `errors_and_fixes`,
`problem_solving`, `pending_tasks`, `current_work`, `next_step` — com **parsing leniente**
(campos extra não descartam o resumo).

## Porque interessa
O gate G4 tem **Q4 com `compacted=true`**. Campos como `pending_tasks`/`next_step` podem
melhorar a qualidade **pós-compactação**. O ktu já compacta de forma aditiva; isto
**enriquece o resumo**, sem mudar o modelo nem o loop.

## O que o ktu tem hoje
Compactação aditiva (resumo + cru preservado + âncora verificável) — passou o gate, mas o
resumo é texto.

## Pré-condições para reconsiderar
- Medição: comparar Q3/Q4 **antes/depois** do resumo estruturado.
- Um template de resumo + parsing leniente com teste de ouro.

## Custo e risco
- **Esforço:** médio (schema + template + parsing + teste de ouro).
- **Risco de não fazer:** baixo (o gate já passa).
- **Risco de fazer:** mexer no **caminho crítico do gate** — só com medição.

## Decisão pendente
Avaliar com a bateria; promover apenas se o resumo estruturado **melhorar** Q4 sem
regressão. Não substituir a compactação aditiva — enriquecê-la.
