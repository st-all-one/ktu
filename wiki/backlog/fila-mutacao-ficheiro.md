# Fila de mutação de ficheiro

- **Estado:** candidata (do `pi`) — **incorporar**.
- **Origem:** `REF/pi/packages/coding-agent/src/core/tools/file-mutation-queue.ts`;
  `wiki/revisao-pi-profunda.md` §1.3.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`.

## O que é
Uma fila **por caminho** (chave = `realpath`, com fallback ao `resolve` quando o ficheiro não
existe) serializa as mutações ao mesmo ficheiro — evita escritas concorrentes a partir de tool
calls paralelas.

## Porque interessa
O ktu tem `write`/`edit` e pode correr tool calls em paralelo (lote `Shared`). Sem serialização,
dois `edit` ao mesmo ficheiro **perdem dados**. É **kernel** (uma fila determinística por caminho)
e uma correcção, não uma feature.

## O que o ktu tem hoje
`write`/`edit` sem fila por ficheiro; paralelismo de tool calls existe.

## Pré-condições para reconsiderar
- Paralelismo de tool calls que possam tocar o mesmo ficheiro.

## Custo e risco
- **Esforço:** baixo-médio (fila por caminho + chave canónica).
- **Risco de não fazer:** escritas perdidas.
- **Risco de fazer:** *deadlock* se a fila for reentrante — mutex por caminho, sem reentrância.

## Decisão pendente
Promover. Fila por caminho (chave canónica), não reentrante; teste de duas mutações concorrentes
ao mesmo ficheiro.
