---
id: task_00uz2my4
type: task
statement: Validar se knudge-core indexa corpus JSONL; definir schema físico de .agents/memory/notes.jsonl
created_at: 2026-10-13T00:50:49.021Z
body_hash: e2c98189
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:45:30.431Z
    notes: "Validado por spike (crates/ktu-memory/tests/knudge_spike.rs, 2 testes verdes): (1) o motor puro do knudge-core indexa notas materializadas em memoria — Index::build + Graph::from_notes_ref + recall devolvem hits (id+statement+score) sem Store/Fs; (2) o codec JSON canonico do knudge-core faz ida-e-volta do frontmatter (Frontmatter::to_value/from_value). Conclusao registada: o knudge-core NAO indexa corpus JSONL (usa notas/*.md via Fs); adoptamos JSONL como container + motor knudge em memoria (decisao_01tt4ir5). Schema fisico definido (decisao_010knmoo): uma linha JSON {frontmatter,body}. Dependencia knudge-core (path) adicionada; workspace exclui REF. make check a validar."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_01iz1xh1 -->