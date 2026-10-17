# Sessões versionadas + migração

- **Estado:** candidata (do `pi`) — **avaliar** (barato).
- **Origem:** `REF/pi/packages/coding-agent/docs/session-format.md`; `wiki/revisao-pi.md` §2.10.
- **Âncora:** `crates/ktu-core/src/session/mod.rs`.

## O que é
O formato de sessão tem um campo `version` (o pi: v1 linear → v2 árvore → v3) com
**migração automática** ao carregar. Cada entrada tem `type`, `id`, `parentId`, `timestamp`.

## Porque interessa
O ktu tem eventos com estrutura implícita; tornar a **versão** e a **migração** explícitas
é barato e **protege o formato** (a fonte da verdade). É o pré-requisito de qualquer
evolução do log (ex.: árvore).

## O que o ktu tem hoje
`Event` (serde) com `#[serde(tag = "event")]`; log JSONL. Sem versão explícita nem migração.

## Pré-condições para reconsiderar
- Qualquer mudança futura ao formato do log (ex.: `session-tree`).

## Custo e risco
- **Esforço:** baixo (campo de versão + migração + testes).
- **Risco de não fazer:** evoluções futuras quebram logs antigos.
- **Risco de fazer (YAGNI):** pequeno.

## Decisão pendente
Promover **junto** com a primeira mudança ao formato (árvore). Um header com `version` +
função de migração; testar carregar logs antigos.
