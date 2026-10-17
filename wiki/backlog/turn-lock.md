# Lock de turno por sessão (entre processos)

- **Estado:** candidata (do `katu`, L-Q6) — **incorporar**.
- **Origem:** `REF/katu/wiki/_ref/plan/OPTIMIZATION_PLAN.md` (WL1-3); `wiki/revisao-katu.md` §3.2.
- **Âncora:** `crates/ktu-core/src/session/mod.rs`, `crates/ktu-core/src/store.rs`.

## O que é
`turn.lock` (pid + idade; obsoleto após 300 s) impede **dois processos** no mesmo log; recusa com
`conflict`. O katu regista que `turn_open` (estado do **processo**) **não** deteta outro processo —
o lock é a única leitura **entre processos**.

## Porque interessa
O ktu tem `session_store.rs` e várias entradas (CLI, `serve`, TUI). Dois processos na mesma sessão
corrompem o log ou duplicam turnos. O lock é uma trave de robustez barata.

## O que o ktu tem hoje
`turn_lock` = 0 hits. Sessões persistidas em JSONL (`session_store.rs`).

## Pré-condições para reconsiderar
- Um cenário real de concorrência (CLI + serve, ou dois `ktu` no mesmo projecto).

## Custo e risco
- **Esforço:** baixo (ficheiro `turn.lock` + idade + porta `Fs`).
- **Risco de não fazer:** log corrompido/duplicado.
- **Risco de fazer:** lock obsoleto bloqueia — idade de expiração (300 s) + `--force`.

## Decisão pendente
Promover. Lock com pid + idade, expiração documentada, recusa `conflict`, teste determinístico com
`MemFs`/`FakeClock`.
