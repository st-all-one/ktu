# Política de durabilidade do log (`event` vs `turn`)

- **Estado:** candidata (do `katu`, ADR 0024) — **incorporar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0024-durabilidade-do-log.md`; `wiki/revisao-katu.md` §3.1.
- **Âncora:** `crates/ktu-core/src/store.rs`, `crates/ktu-core/src/session/mod.rs`.

## O que é
`behavior.durability` = `event` (default, `fsync` por evento) ou `turn` (*group commit*: escreve
sem sincronizar, barreira no fim do turno). A **janela de perda** é explícita por modo. Complemento
obrigatório: recuperação de **cauda rasgada** (descartar a última linha incompleta + `log.recovered`).
Medido no katu: **−78,1 %** no caminho de anexação.

## Porque interessa
O ktu grava com `temp→fsync→rename` (durável), mas **um `fsync` por evento** custa. Uma política
opt-in permite trocar durabilidade por velocidade sem regressão silenciosa. A cauda rasgada
**já é recuperada** no ktu (`store.rs`), logo falta só a política.

## O que o ktu tem hoje
`store.rs`: JSONL, recuperação do prefixo válido; `fsync` presente; `durability` = 0 hits.

## Pré-condições para reconsiderar
- Um caminho de escrita de log **incremental** (append), não reescrita total.

## Custo e risco
- **Esforço:** baixo-médio (`store.rs` + config + teste de cauda rasgada).
- **Risco de não fazer:** `fsync` a mais.
- **Risco de fazer:** default `turn` perde eventos em crash — **manter `event` por omissão** e a
  janela documentada.

## Decisão pendente
Promover. Default `event`; `turn` opt-in; janela de perda explícita; cauda rasgada recuperada
(trava). Nunca tornar `turn` default sem decisão do dono.
