---
id: fact_014kak9j
type: fact
statement: H0.5 fechada: envelope --json em todos os verbos de dados + exit codes documentados; make check verde
created_at: 2026-10-17T23:27:12.898Z
body_hash: 496045d7
schema_version: 2
anchors: [crates/ktu-cli/src/output.rs]
---
H0.5 (task_012xuf1i) fechada. Envelope --json (crates/ktu-cli/src/output.rs, Format/Success/Failure) aplicado a TODOS os verbos de dados: prime, self version, memory list/export/doctor --memory, guardrails list, plugins list/reload/doctor, trash list/restore, providers doctor. Cada comando recebe format: Format e usa output::emit(format, command, text, data) / emit_error. Codigos de saida documentados no PROTOCOL (prime) e no rodape AFTER_HELP: 0=ok, 1=falha, 2=uso invalido. Verificado: ktu --json guardrails list -> {success:true,command,data:{rules:[...]}}; memory list/trash list/doctor memory idem. Limpeza: junk de teste (2 notas meta_) removido de .agents/memory/notes.jsonl (o commit tinha 0 linhas). Estado: make check VERDE; ktu-cli 27+2+1 testes. Resta na H0: H0.4 (completions/setup).