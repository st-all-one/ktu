---
id: fact_01ygtjoy
type: fact
statement: "Tool subagent delega via porta SubagentHost; sessao-filha fresca, relatorio capado (8k), profundidade 1"
created_at: 2026-10-17T00:20:26.133Z
body_hash: 68bebd72
schema_version: 2
anchors: [crates/ktu-core/src/subagent.rs, crates/ktu-cli/src/subagents.rs, crates/ktu-core/src/tools.rs]
---
ktu-core/src/subagent.rs define SubagentHost (porta), SubagentRequest/SubagentReport, spec() e parse_request/cap_report. O loop (agent/delegation.rs::run_subagent) intercepta a tool subagent, chama o host e capa o texto; Deps.subagents Option liga/desliga; LoopConfig.subagent_report_chars default 8192. A borda (ktu-cli/src/subagents.rs::CliSubagents) corre a filha na thread do kernel: AgentSpec do explorer + ScopedTools (filtro de specs + denies por capacidade) + guard for_spec + log proprio persistido em .ktu/sessions/<id>.jsonl com meta do pai; memoria None (read-only) e subagents None (sem netos). Testes: loop subagent_delegation_returns_a_capped_report + scoped_tools_filters_specs_and_denies_off_spec + parse/cap; make check verde.