# Pesquisa no histórico de sessões

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose/src/session/chat_history_search.rs`;
  `wiki/revisao-goose.md` §3.
- **Âncora:** `.ktu/sessions/*.jsonl`, `crates/ktu-core/src/session/mod.rs`.

## O que é
Pesquisar em **sessões antigas** (histórico de chat) por conteúdo, para retomar contexto
de conversas passadas.

## Porque não entra
O ktu tem `resume` por id e a **memória** (knudge) para conhecimento durável. Pesquisar
logs de sessão é uma conveniência; o log é a fonte da verdade, mas não há consumidor de
pesquisa.

## Pré-condições para reconsiderar
- Muitas sessões e necessidade de as encontrar por conteúdo.
- Um índice derivado (descartável) definido.

## Custo e risco
- **Esforço:** baixo-médio (índice + comando).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** índice derivado a manter.

## Decisão pendente
Adiar. Se entrar, derivar do log (nunca fonte de verdade própria) e ser descartável.
