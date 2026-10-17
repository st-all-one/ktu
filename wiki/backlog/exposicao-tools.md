# Exposição de tools (declared/deferred) + `tool_search`

- **Estado:** candidata (do `pi`) — **incorporar**.
- **Origem:** `REF/pi/packages/coding-agent/src/extensions/tool-search/tool.ts`,
  `core/extensions/types.ts` (`ToolExposure`); `wiki/revisao-pi-profunda.md` §1.1.
- **Âncora:** `crates/ktu-core/src/tools.rs`, `crates/ktu-core/src/prompt.rs`.

## O que é
Cada tool tem uma **exposição**: `declared` (vai ao modelo), `deferred` (não vai) ou via
`codemode`. O `tool_search` é um **ranker BM25** sobre os metadados das tools; devolve os `top-8`
e **carrega** os que faltam, ficando declarados no **próximo** pedido — registado no transcript
(sobrevive a `/tree`, resume e fork).

## Porque interessa
O ktu declara **todas** as tools no catálogo. Com muitas tools, o catálogo gasta tokens e confunde
o modelo. A exposição por **dado** (declared/deferred) + `tool_search` (BM25 determinístico)
mantém o catálogo pequeno sem perder tools. **Distinto** do que o katu rejeitou (A4, retrieval
semântico, 14 % < 20 %): aqui o ganho é **não declarar** até ser preciso.

## O que o ktu tem hoje
Catálogo fixo com todas as tools (`tools.rs`).

## Pré-condições para reconsiderar
- Catálogo que cresça além do que cabe confortavelmente no prompt.

## Custo e risco
- **Esforço:** médio (exposição por tool + ranker BM25 + carregamento no próximo pedido).
- **Risco de não fazer:** prompt gasto em tools não usadas.
- **Risco de fazer:** o modelo não descobre a tool — `tool_search` sempre disponível + nomes claros.

## Decisão pendente
Promover. Exposição por tool (dado), `tool_search` determinístico (BM25, `BTreeMap`), carregamento
registado no log; teste de que a tool carregada aparece no pedido seguinte.
