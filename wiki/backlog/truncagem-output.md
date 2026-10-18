# Truncagem sistemática de output (linhas + bytes)

- **Estado:** **implementado** (H5; `wiki/implementation_phase2/H5-fiabilidade-loop.md`).
- **Origem:** `REF/pi/packages/coding-agent/src/core/tools/{truncate,output-accumulator}.ts`;
  `wiki/revisao-pi-profunda.md` §1.2.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`, `crates/ktu-tools/src/bash.rs`.

## O que é
Dois limites **independentes**, o que vier primeiro: **2000 linhas** e **50 KB**; nunca devolve
linhas parciais (excepto o *tail* do bash). O acumulador trunca e guarda o **output completo**
num ficheiro, devolvendo o **caminho** ao modelo. O `grep` limita cada linha a 500 chars.

## Porque interessa
É a política de truncagem **canónica** — mais completa que caps soltos. A ideia é a mesma da página
[81 — resposta-grande-ficheiro](./resposta-grande-ficheiro.md) (goose); esta acrescenta o **duplo
limite** e o **caminho do ficheiro** (o modelo pode ir buscar o resto).

## O que o ktu tem hoje
Caps de leitura/execução; sem política unificada de truncagem nem output completo em ficheiro.

## Pré-condições para reconsiderar
- Output de tool que exceda o orçamento no fluxo real.

## Custo e risco
- **Esforço:** baixo (helper partilhado + ficheiro + caminho).
- **Risco de não fazer:** orçamento estoura; truncagem perde dados.
- **Risco de fazer:** ficheiros acumulam — caminho previsível + limpeza documentada.

## Decisão pendente
Promover. Duplo limite (linhas + bytes), sem linhas parciais, output completo em ficheiro com
caminho devolvido; **uma** política para todas as tools (consolidar 81).
