# Postmortems — incidentes reais, com prevenção

Um incidente que aconteceu uma vez volta a acontecer se a causa não ficar escrita. Cada postmortem
descreve **o que se perdeu**, **porquê** e **que travão ficou** — sem culpa e sem teatro: o
objectivo é a próxima pessoa não repetir.

## Contrato (imposto por `make check` → `xtask check-adr`)

1. Nome `PM-NNNN-slug.md`, id único.
2. Título `# PM-NNNN — <título>`.
3. Cinco secções obrigatórias: `## Contexto`, `## Impacto`, `## Causa`, `## Correção`,
   `## Prevenção`.
4. Este índice lista todos os postmortems.

## O que **não** é um postmortem

- Um *bug* já coberto por teste (vive no teste e no *commit*).
- Uma decisão de arquitectura (vive numa [ADR](../adr/README.md)).
- Uma ideia por fazer (vive no [dossiê de pendências](../pendencias.md)).

## Índice

| # | Incidente | Prevenção que ficou |
|---|---|---|
| [PM-0001](PM-0001-modulo-de-testes-apagado-pelo-script.md) | Módulo de testes apagado por um script de extracção | escrever o ficheiro novo antes de reescrever o antigo; correr o teste logo depois |
| [PM-0002](PM-0002-snippets-da-doc-que-nao-compilavam.md) | Snippets canónicos da documentação que não compilavam (`E0716`) | excertos canónicos compilados num teste do workspace |
| [PM-0003](PM-0003-links-da-doc-a-apodrecer.md) | Links internos da wiki a apodrecer em silêncio | verificação de links no `make check` + `cargo doc` na bateria completa |
