# Permission store (aprovações memorizadas)

- **Estado:** **implementado** (H6) — `.agents/permissions.toml` (`Always/Ask/Never`), `ktu permission ls|set|unset`; `Ask` → `Escalate`. TTL implícito não entrou (permissões explícitas no ficheiro).
- **Origem:** `REF/goose/crates/goose/src/permission`; `wiki/revisao-goose.md` §2.3.
- **Âncora:** `crates/ktu-core/src/guard.rs`, `crates/ktu-core/src/plan.rs` (`Approval`).

## O que é
Memorizar uma **aprovação** (por tool/caminho) para não a pedir de novo, com modos de
permissão (`auto`/`approve`/`chat`) e expiração.

## Porque interessa
O ktu tem `Approval`/`Mode`/`Escalate`, mas **cada** escrita fora do root volta a pedir
aprovação. Memorizar é usabilidade objectiva; a decisão continua sobre **factos tipados**
(o seam do `Guard`), e o store é um `BTreeMap` determinístico e logado.

## O que o ktu tem hoje
`StructuralGuard` (capacidade + confinamento) e `Approval` por turno; sem memória de
aprovações.

## Pré-condições para reconsiderar
- Fricção real com aprovações repetidas num fluxo de trabalho.
- Escopo e TTL definidos (o quê e por quanto tempo memorizar).

## Custo e risco
- **Esforço:** baixo-médio (store + regra + expiração + log).
- **Risco de não fazer:** aprovações repetitivas.
- **Risco de fazer:** **alargar privilégio** — mitigar com escopo mínimo, TTL e log.

## Decisão pendente
Promover só com um caso de uso concreto; a aprovação memorizada deve ser **explícita**
e **expiráve**l, nunca implícita.
