# Snapshot no limite de fase (retomada rápida)

- **Estado:** candidata (do `katu`, ADR 0008 / Q-15) — **avaliar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0008-sessoes-identidade-e-retomada.md`;
  `wiki/revisao-katu.md` §3.15.
- **Âncora:** `crates/ktu-core/src/checkpoint.rs`, `crates/ktu-core/src/session/mod.rs`.

## O que é
Snapshot do estado a cada transição de fase com `{seq, offset, …}`; `resume` lê só a **cauda** (não
relê o prefixo). Medido no katu: 20 000 turnos em **321 µs** (−98,7 % vs replay). A invariante
`state_of(replay)` é validada na retomada; o snapshot é reconstruível.

## Porque interessa
Retomar lendo o log todo é O(história). Um snapshot no limite de fase mantém a retomada O(deltas).
O ktu já tem `checkpoint.rs` — avaliar se cobre a retomada rápida ou se relê o log inteiro.

## O que o ktu tem hoje
`checkpoint.rs` (gravado após cada turno, `temp→fsync→rename`); retomada por carregamento do log.

## Pré-condições para reconsiderar
- Sessões longas (retomada lenta observada).

## Custo e risco
- **Esforço:** médio (offset + leitura da cauda + validação da invariante).
- **Risco de não fazer:** retomada O(história).
- **Risco de fazer:** snapshot diverge do log — validar `state_of(replay)` na retomada; fallback
  replay total.

## Decisão pendente
Avaliar. Se a retomada já for rápida o suficiente, fechar; senão, snapshot com `offset` e leitura da
cauda, validado pela invariante.
