# PM-0002 — Snippets canónicos da documentação que não compilavam

- **Data:** 2026-10-17/18 (H9 + integração) · **Onde:** `wiki/integration/`, `wiki/fachada-crate.md`

## Contexto

A pedido do dono do projecto, foi escrita documentação de integração da fachada (`KtuBuilder`) para
programadores e para IA — com excertos de código "canónicos" que o leitor deve poder copiar.

## Impacto

Ao compilar os excertos num teste temporário descobriu-se que a forma natural de escrever a cadeia
do construtor **não compila**: `&T::new()` como argumento intermédio provoca `E0716` (valor
temporário eliminado enquanto emprestado) — é preciso **ligar cada dependência a uma variável**
(ou, para unit structs como `NoopLogger`/`NoopMetrics`, deixá-las promover a `'static`). A
documentação, escrita para ser copiada, estava a ensinar código que não compila; e os links
intra-doc do crate só são verificados por `cargo doc`, que **não** está no `make check`.

## Causa

Documentação escrita a partir da memória da API, sem a compilar. O gate do projecto valida código,
não excertos de Markdown — e o `cargo doc` ficou fora do portão por decisão de âmbito.

## Correção

Os excertos foram corrigidos (cada dependência ligada a variável) e o achado ficou registado na
documentação (nota sobre `E0716` e sobre `'static` promovido); o crate de exemplo
`examples/ktu-embed` — que **compila no gate** — passou a ser a referência executável.

## Prevenção

- Todo o excerto "canónico" tem de ter um equivalente **compilado** (o exemplo do workspace, ou um
  teste). O exemplo `examples/ktu-embed` existe para isso e corre em `--backend scripted`.
- O `make check` continua sem `cargo doc` (medição ruidosa/de âmbito), mas a bateria completa inclui
  `cargo doc --workspace --no-deps` — e a decisão de o promover a gate está registada no dossiê.
- Regra de escrita: ao documentar uma API, abrir o código (ou compilar) antes de a descrever.
