# knudge — `Store` + `notas/` + `eventos/` + protocolo `write`

- **Estado:** candidata (do `knudge` 0.6.1) — **incorporar**.
- **Origem:** `crates/knudge/wiki/integration/{03_store_e_persistencia,05_escrita_e_ciclo_de_vida}.md`,
  `crates/knudge/wiki/specs/{persistencia,escrita-e-dedup}.md`; `wiki/revisao-knudge.md` §6.1.
- **Âncora:** `crates/ktu-memory/src/{store,convert,record}.rs`, `crates/ktu-core/src/memory.rs`.

## O que é
O knudge persiste em `notas/<tipo>/<id>.md` (a **verdade**, frontmatter TOON) + `eventos/`
(auditoria append-only) + `.idx/`/`cache/`/`.locks/` (derivado). A escrita é atómica
(`tmp+rename`), com **nota antes do evento**, lock advisory por alvo (stale 30 s, RAII) e rebuild
por double-buffer. O protocolo `write` é idempotente por id → dedup → **create/merge/reject**;
`update` faz revisão no lugar ou **supersede** (novo id + `replaces`/`superseded_by`).

## Porque interessa
O `ktu-memory` **reimplementa** hoje, de forma mais simples, o que o knudge já faz testado:
JSONL append + dedup manual + `temp→fsync→rename`. Adoptar `Store`/`write` dá atomicidade, lock,
auditoria, dedup-on-read e supersede **de graça**, mantendo a porta `Memory` (tipos do ktu).

## O que o ktu tem hoje
`JsonlMemory` sobre `.agents/memory/notes.jsonl`; dedup via `propose`; sem eventos/auditoria/lock.

## Pré-condições para reconsiderar
- Nenhuma (a migração 0.5.4→0.6.1 é uma linha, já validada: `cargo check`/`test` verdes).

## Custo e risco
- **Esforço:** médio (adoptar `Store`/`write_context` + mapear `MemoryDraft`→`Draft`).
- **Risco de não fazer:** durabilidade/auditoria subótimas; supersede ausente.
- **Risco de fazer:** migrar o formato do JSONL — manter leitura tolerante e rebuild.

## Decisão pendente
Promover. Adaptador usa `Store`/`notas/`/`eventos/` + `write`; a porta `Memory` mantém tipos do
ktu; layout configurável (D214) permite `.agents/memory/`; teste de migração do JSONL antigo.
