# Normalização de line-endings/BOM no `edit`

- **Estado:** **implementado** (H5; `wiki/implementation_phase2/H5-fiabilidade-loop.md`).
- **Origem:** `REF/pi/packages/coding-agent/src/core/tools/edit-diff.ts`,
  `utils/text.ts` (`splitBom`); `wiki/revisao-pi-profunda.md` §1.4.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`.

## O que é
Detecta o **line-ending** (`\r\n`/`\n`), normaliza para LF para calcular o diff e **restaura** o
ending original; `splitBom` trata o BOM.

## Porque interessa
Editar um ficheiro CRLF num ambiente Unix (ou vice-versa) corrompe o ficheiro inteiro num diff
(todas as linhas mudam). Preservar o ending é **correcção**, não feature.

## O que o ktu tem hoje
`edit` faz substituição de texto; sem normalização de ending/BOM.

## Pré-condições para reconsiderar
- Edição de ficheiros CRLF (Windows) ou com BOM.

## Custo e risco
- **Esforço:** baixo (detectar + normalizar + restaurar).
- **Risco de não fazer:** diffs falsos; ficheiros corrompidos.
- **Risco de fazer:** baixo.

## Decisão pendente
Promover. Detectar o ending, calcular o diff em LF, restaurar o ending e o BOM; teste com ficheiro
CRLF e com BOM.
