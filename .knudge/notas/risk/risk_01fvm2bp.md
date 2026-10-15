---
id: risk_01fvm2bp
type: risk
statement: knudge-core pode não indexar um corpus JSONL directamente
created_at: 2026-10-13T00:52:07.719Z
body_hash: dd652762
schema_version: 2
tags: [ktu, risk, memoria]
supports: [decision_0166fqyj]
revision: 2
anchors: [plan.md]
---
Por quê: o knudge assume notas Markdown como corpus.
Evidência: plan.md §2.7 (risco registado).
Consequência: validar no início de G3; se não, o adaptador projeta registos JSONL -> vista de notas.