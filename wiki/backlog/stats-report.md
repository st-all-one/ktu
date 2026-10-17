# Stats / report

- **Estado:** deferido.
- **Origem:** katu `stats`/`report`.
- **Âncora:** `crates/ktu-cli/src/memory_cmd.rs`, `crates/ktu-core/src/ports/metrics.rs`.

## O que é
Comandos de **estatísticas e relatórios**: uso de tokens, custo, latência, nº de turnos/
tools, saúde do projecto — agregados a partir dos logs/métricas.

## O que o ktu tem hoje
- `ktu doctor` (layout + conhecimento), `ktu memory list/export`, `ktu providers doctor`.
- Métricas por função (feature `instrument`, `MetricsSnapshot` serializável).
- Benchmark do núcleo com artefacto (`wiki/benchmarks/ktu-bench.json`).

## Porque não entrou
Não há consumidor de relatórios agregados; a evidência existe no log e nas métricas. O
`stats` do katu era um dos subsistemas "antes de funcionar".

## Pré-condições para reconsiderar
- Necessidade de **acompanhar** uso/custo ao longo do tempo (uso real prolongado).
- Um formato de relatório com consumidor (humano ou CI).

## Custo e risco
- **Esforço:** baixo-médio (agregar do log/métricas).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** mais uma superfície de output sem uso.

## Decisão pendente
Adiar. Se entrar, **derivar** do log/métricas (nunca uma segunda contabilidade).
