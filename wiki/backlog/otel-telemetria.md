# OpenTelemetry / telemetria

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose/src/{otel,tracing,posthog.rs}`; feature `otel`;
  `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-core/src/ports/metrics.rs`, `crates/ktu-core/src/agent/logging.rs`.

## O que é
Exportar traces/métricas para OpenTelemetry (OTLP), e telemetria de produto (PostHog).

## Porque não entra
O ktu tem **métricas por função** (feature `instrument`, `MetricsSnapshot` JSON) e logs
estruturados — suficientes e sem dependências. OTel acrescentaria uma stack de observabilidade
sem consumidor; telemetria de produto não é aceitável por omissão.

## Pré-condições para reconsiderar
- Operação em produção que exija correlação distribuída (traces OTLP).

## Custo e risco
- **Esforço:** médio (deps OTel + export).
- **Risco de não fazer:** baixo (métricas locais bastam).
- **Risco de fazer (YAGNI):** stack pesada; telemetria de produto = privacidade.

## Decisão pendente
Manter fora. As métricas actuais são o contrato; OTel seria um export opcional, nunca por
omissão.
