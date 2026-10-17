# Snapshots + `/undo` / `/redo`

- **Estado:** candidata (do `opencode`) — **incorporar**.
- **Origem:** `REF/opencode/packages/core/src/{snapshot.ts,session/revert.ts}`,
  `docs/content/snapshots.mdx`; `wiki/revisao-opencode-profunda.md` §1.1.
- **Âncora:** `crates/ktu-core/src/session/mod.rs`, `crates/ktu-tools/src/fs_tools.rs`.

## O que é
Por cada passo do modelo, tira-se um **snapshot** imediatamente **antes** da chamada e outro no
sucesso/falha; a mensagem do assistente registra os **caminhos alterados**. O `/undo` **encena**
um rollback (restaura os ficheiros **e** devolve o prompt ao composer), o `/redo` cancela;
submeter o prompt editado **compromete** o rollback. Também se reverte para uma mensagem
anterior. A captura é *best-effort*; o escopo é o directório activo.

## Porque interessa
O ktu tem `trash`/`restore` e checkpoint, mas **não** um undo/redo **de conversa + ficheiros**
encenado. É usabilidade objectiva e encaixa no log (a reversão é uma operação registada). Casa
com [40 — session-tree](./session-tree.md) e [85 — editar-em-sessao](./editar-em-sessao.md).

## O que o ktu tem hoje
`trash`/`restore` + `checkpoint.rs`; sem undo/redo encenado.

## Pré-condições para reconsiderar
- Necessidade de rever trabalho recente sem git.

## Custo e risco
- **Esforço:** médio (stage/commit/clear + snapshot de ficheiros).
- **Risco de não fazer:** sem undo coordenado.
- **Risco de fazer:** tocar em ficheiros — snapshot *best-effort*, nunca bloquear; o log fica
  intacto (a reversão é um evento).

## Decisão pendente
Promover. Estágio explícito (`/undo` → `/redo` → commit ao submeter); snapshot *best-effort*;
caminhos alterados no evento; teste de rollback de conversa + ficheiros.
