# Permissões por tool (Always/Ask/Never)

- **Estado:** candidata (do `goose`) — **incorporar**.
- **Origem:** `REF/goose/documentation/docs/guides/managing-tools/tool-permissions.md`;
  `wiki/revisao-goose-profunda.md` §1.4.
- **Âncora:** `crates/ktu-core/src/guardrails.rs`, `crates/ktu-core/src/guard.rs`.

## O que é
Além do **modo** global, cada tool tem um nível: **Always Allow** / **Ask Before** / **Never
Allow**. O modo é o default; a permissão por tool **sobrepõe**. (O goose avisa: < 25 tools no
total, por desempenho.)

## Porque interessa
O ktu tem guardrails (por caminho/padrão) e `Approval`, mas **não** uma política declarativa
por **tool**. É o complemento natural — **dado** (nome da tool → nível), fail-closed, e mapeia
directamente nos modos (plano/execução). Casa com a página [21 — permission-store](./permission-store.md)
(memorização) e [62 — motor-de-politica](./motor-de-politica.md).

## O que o ktu tem hoje
Guardrails por caminho/padrão + `Approval`; sem política por tool.

## Pré-condições para reconsiderar
- Tools com risco distinto (ex.: `bash` vs `read`).

## Custo e risco
- **Esforço:** baixo (regra por tool no `.agents/guardrails/`).
- **Risco de não fazer:** política grossa demais.
- **Risco de fazer:** alargar privilégio — **Never Allow** por omissão para tools sensíveis;
  fail-closed.

## Decisão pendente
Promover. Nível por tool como **dado**; `Never Allow` por omissão para sensíveis; sobrepõe ao
modo; teste por tool.
