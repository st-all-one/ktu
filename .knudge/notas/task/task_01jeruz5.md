---
id: task_01jeruz5
type: task
statement: Compactação aditiva + checkpoint tipado durável (temp->fsync->rename) + recuperação de crash
created_at: 2026-10-13T00:50:49.196Z
body_hash: 7373f1b8
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T03:02:23.728Z
    notes: "Compactacao aditiva + checkpoint tipado duravel. ktu-core/src/compact.rs: summarize (resumo deterministico dos turnos descartados) + keep_from_for_turns (mantem os ultimos KEEP_TURNS). SessionLog ganhou compaction_from/set_compaction/is_compacted; derive_messages substitui os eventos anteriores ao corte por um resumo (log cru intacto; prefixo estavel). O loop compacta uma vez quando o orcamento nao cabe (antes falhava com Error) e emite session.compact. ktu-core/src/checkpoint.rs: Checkpoint (session/turns/compacted/keep_from/updated_ms) gravado temp->fsync->rename; store::load_recovering tolera a ultima linha parcial (crash); run.rs retoma o corte do checkpoint e grava-o apos cada turno. Testes: compact (2), checkpoint (3), loop::a_tight_budget_compacts_instead_of_failing. make check verde."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01iz1xh1 -->