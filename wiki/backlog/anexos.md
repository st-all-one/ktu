# Anexos (`uri` + gamas de linhas)

- **Estado:** candidata (do `opencode`) — **avaliar** (consolidar com [101](./visao-imagens.md)).
- **Origem:** `REF/opencode/docs/content/attachments.mdx`;
  `wiki/revisao-opencode-profunda.md` §1.11.
- **Âncora:** `crates/ktu-core/src/domain.rs`, `crates/ktu-tools/src/fs_tools.rs`.

## O que é
Um anexo é `{uri, name?, description?}`; URLs `file://` absolutos; `start`/`end` (1-based) para
gamas de linhas; imagens/PDF passados ao modelo (até 20 MiB).

## Porque interessa
Formaliza o anexo (ficheiro ou gama de linhas) como **dado** — o utilizador aponta um ficheiro e
o ktu injeta-o no prompt com proveniência. Consolidar com [101 — visao-imagens](./visao-imagens.md)
e a leitura por `offset`/`limit`.

## O que o ktu tem hoje
Leitura por `read`; sem anexos declarativos.

## Pré-condições para reconsiderar
- Anexar ficheiros ao prompt (TUI/CLI).

## Custo e risco
- **Esforço:** baixo-médio (estrutura de anexo + resolução de `file://`).
- **Risco de não fazer:** o utilizador cola conteúdo à mão.
- **Risco de fazer:** acesso a caminhos fora do root — confinamento; tamanho limitado.

## Decisão pendente
Avaliar. Anexo como dado (`uri` + gama); resolução de `file://` respeitando o confinamento; caps
de tamanho; consolidar com [101](./visao-imagens.md).
