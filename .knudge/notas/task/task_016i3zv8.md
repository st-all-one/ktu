---
id: task_016i3zv8
type: task
statement: xtask check-links (wiki + intra-doc) no make check (L3)
created_at: 2026-10-18T13:32:18.610Z
body_hash: 5a1ddc56
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T15:31:27.581Z
    notes: "xtask/src/links.rs (check-links) + make links + make doc no make check; 770 links relativos, 0 quebrados, 0 avisos; o gate G22 prova a falha num link deliberado e o aviso da ancora; o cargo doc -D warnings apanhou um aviso real (redundant_explicit_links em ktu-cli/src/instructions.rs). Medido: ~4 s quente, ~34 s com alvo vazio (limiar 60 s)."
anchors: [wiki/implementation_phase3/F7-travoes.md]
status: closed
scope: task
---
<!-- knudge:parent task_01nopvll -->