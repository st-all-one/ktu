# Catálogo de diagnóstico extenso (122 eventos)

- **Estado:** deferido.
- **Origem:** katu `diag` (catálogo de 122 eventos), `wiki/_ref/plan/20-instrumentacao-transversal.md`.
- **Âncora:** `crates/ktu-core/src/events.rs`, `crates/ktu-core/src/agent/logging.rs`.

## O que é
Um **catálogo tipado e exaustivo** de eventos de diagnóstico (o katu tinha 122),
verificado por `xtask` (nenhuma macro de output fora do sink), com custo zero por
omissão e ligação em runtime.

## O que o ktu tem hoje
- Um conjunto **estável e pequeno** de ids de evento (`turn.start/end`, `tool.call/result`,
  `guard.decision`, `plugin.dispatch`, `plugin.lifecycle`, `server.start`, …).
- Porta `Logger` (custo zero por omissão) e a feature `instrument` (spans por função).
- `xtask check-layers` para o firewall de camadas.

## Porque não entrou
122 eventos sem consumidor é ruído; o ktu mantém só os que têm uso (gate, depuração,
métricas). A verificação "só pelo sink" ainda não existe como gate.

## Pré-condições para reconsiderar
- Uma necessidade de observabilidade que exija eventos novos.
- Um `xtask` que valide o catálogo (a parte **barata** e útil — pode entrar isolada).

## Custo e risco
- **Esforço:** baixo (acrescentar eventos) a médio (gate `check-diag`).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** catálogo que cresce sem uso.

## Decisão pendente
Se houver interesse, avaliar **primeiro** o gate `xtask check-diag` (útil e barato),
independentemente do número de eventos.
