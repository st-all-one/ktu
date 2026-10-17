# Instruções como deltas de valor

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/specs/v2/session.md` (Instructions Are Value Deltas);
  `wiki/revisao-opencode-profunda.md` §1.7.
- **Âncora:** `crates/ktu-core/src/prompt.rs`, `crates/ktu-core/src/session/mod.rs`.

## O que é
A sincronização de instruções persiste valores **content-addressed** (SHA-256);
`session.instructions.updated { delta, text? }` mapeia cada fonte alterada a um hash, com
`"removed"` para ausência; os corpos vivem uma vez num store (`instruction_blob`); a projecção
`instruction_state` fornece os valores correntes e iniciais; uma **epoch** atravessa compactações.

## Porque interessa
O ktu recompõe o prompt a cada turno. Persistir instruções como **deltas** (com epoch) evita
reprocessar tudo e dá rasto de **o que mudou** — útil para debug e para o log.

## O que o ktu tem hoje
`prompt.rs` recompõe AGENTS.md/skills a cada turno; sem deltas.

## Pré-condições para reconsiderar
- Muitas fontes de instrução (AGENTS.md aninhados, skills, referências) e necessidade de rasto.

## Custo e risco
- **Esforço:** médio-alto (blobs content-addressed + epoch + projecção).
- **Risco de não fazer:** recomputar tudo; sem rasto.
- **Risco de fazer:** complexidade — só com muitas fontes; o ktu compõe poucas hoje.

## Decisão pendente
Adiar. Se o ktu adotar AGENTS.md aninhados ([52](./agents-md-aninhados.md)) e várias fontes, um
modelo de deltas (content-addressed + epoch) evita recompor tudo; senão, YAGNI.
