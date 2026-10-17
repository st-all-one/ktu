# AGENTS.md aninhados (descoberta de instruções)

- **Estado:** candidata (do `opencode`) — **incorporar**.
- **Origem:** `REF/opencode/packages/core/src/instruction-discovery.ts`,
  `.opencode/command/learn.md`; `wiki/revisao-opencode.md` §2.2.
- **Âncora:** `crates/ktu-core/src/prompt.rs`, `crates/ktu-core/src/layout.rs`.

## O que é
`AGENTS.md` pode existir a **qualquer nível** de directório; ao ler um ficheiro, os
`AGENTS.md` dos directórios-pai são carregados no contexto. O comando `/learn` extrai
aprendizagens da sessão para o `AGENTS.md` **mais próximo** do código.

## Porque interessa
O ktu compõe **um** `AGENTS.md` no prompt. Instruções **por directório** são *context
discovery* simples e de alto valor (monorepos/módulos). `/learn` fecha o ciclo
"sessão → conhecimento" no sítio certo.

## O que o ktu tem hoje
`prompt.rs` compõe base + `AGENTS.md` (condensado); `layout.rs` descobre `.agents/`.
`kd write` regista conhecimento no knudge (não em `AGENTS.md` por directório).

## Pré-condições para reconsiderar
- Um monorepo/estrutura onde instruções locais façam diferença.
- Regra de orçamento (não estourar o prompt com N ficheiros).

## Custo e risco
- **Esforço:** baixo-médio (descoberta por nível + injecção no prompt + comando learn).
- **Risco de não fazer:** instruções globais demais.
- **Risco de fazer:** prompt maior (orçamento) e ambiguidade de instruções.

## Decisão pendente
Promover; descoberta determinística (mais próximo primeiro), com tecto de bytes e
deduplicação. `/learn` escreve no `AGENTS.md` mais próximo (nunca em massa).
