# Diagnóstico empacotado (`doctor`)

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/crates/goose/src/session/diagnostics.rs` + `doctor.rs`;
  `wiki/revisao-goose-profunda.md` §1.11.
- **Âncora:** `crates/ktu-cli/src/` (`doctor`?), `crates/ktu-core/src/layout.rs`.

## O que é
Recolhe um **pacote de diagnóstico**: sistema (versão/OS/arquitetura), provider, config e
**tail de logs**, com caps explícitos (`LLM_LOG_MAX_BYTES` 2 MB, `CONFIG_MAX_BYTES` 256 KB,
`CLI_LOG_TAIL_LINES` 400), em níveis `Summary`/`Full`.

## Porque interessa
Reportar um bug exige juntar contexto à mão. Um pacote com **caps** (nunca vaza tudo, nunca
cresce sem limite) é usabilidade objectiva e barata — e o ktu já tem `layout.rs` e logs.

## O que o ktu tem hoje
Logs (`logging.rs`) e `layout.rs`; sem pacote de diagnóstico.

## Pré-condições para reconsiderar
- Um canal de report de bugs.

## Custo e risco
- **Esforço:** baixo (recolha + caps + níveis).
- **Risco de não fazer:** reports incompletos.
- **Risco de fazer:** vazar segredos — caps + redação antes de escrever (o katu usa
  `diag::redact`).

## Decisão pendente
Avaliar. Pacote com caps, níveis Summary/Full e **redação**; nunca inclui segredos.
