---
id: decision_008xccqv
type: decision
statement: "Duas threads: T1 kernel (loop/sessão) e T2 tools; front-end desconectado, só fala pela porta única"
created_at: 2026-10-13T00:52:06.746Z
body_hash: bf1241f0
schema_version: 2
tags: [ktu, arquitetura]
anchors: [plan.md]
---
Por quê: isolamento de estado e determinismo; o front-end nunca partilha estado com o kernel.
Evidência: plan.md §2.2/§2.3.
Consequência: comunicação só por mpsc/oneshot; mesma porta (Channel embedded ou HTTP/1.1+2) para CLI/TUI/web.