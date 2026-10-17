# Checks de revisão (`.agents/checks/*.md`)

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/crates/goose/src/checks/mod.rs`;
  `wiki/revisao-goose-profunda.md` §1.10.
- **Âncora:** `crates/ktu-core/src/subagent.rs`, `.agents/`.

## O que é
Descobre `.agents/checks/*.md` e `**/.agents/REVIEW.md` (frontmatter `name`/`description`/
`model`/`turn-limit`/`tools`/`severity-default`); cada check corre como um **subagente** e
emite achados atribuídos ao ficheiro de origem.

## Porque interessa
É uma forma **declarativa** de definir verificações (lint/review) que correm como subagentes —
reutiliza o fan-out existente e o padrão "dado + loader fail-closed" dos guardrails.

## O que o ktu tem hoje
Subagentes (`explorer` + fan-out); sem checks declarativos.

## Pré-condições para reconsiderar
- Verificações recorrentes (ex.: revisão de segurança de um diff).

## Custo e risco
- **Esforço:** médio (frontmatter + execução como subagente + relatório).
- **Risco de não fazer:** verificações ad-hoc repetidas.
- **Risco de fazer:** checks mal-definidos gastam turnos — `turn-limit` + `tools` mínimos.

## Decisão pendente
Avaliar. Se entrar, frontmatter mínimo, `turn-limit` e `tools` (mínimo privilégio), achados
atribuídos ao ficheiro de origem.
