---
id: task_015epqsj
type: task
statement: "G5.5 Endurecimento estrutural: isolamento do kernel (T2), disciplina de codigo Rust 1.99 e simplificacao"
created_at: 2026-10-17T00:42:15.612Z
body_hash: b268c888
schema_version: 2
tags: [g5.5]
results_in: [task_016trqis, task_00xdwxvl, task_00js2z5y, task_0039qlfv, task_00ddzsgt]
revision: 8
outcomes:
  - status: success
    recorded_at: 2026-10-17T01:06:55.520Z
    notes: "G5.5 concluida: T2 (isolamento), Deps em 3 grupos, toolchain 1.99 + rustfmt/deny/CI (sniff), deps mortas removidas. make check + cargo machete verdes."
anchors: [plan.md]
status: closed
scope: issue
---
Fase de endurecimento entre G5 e G6. Objectivos: (1) separar de facto a thread de tools T2 (mpsc + oneshot) e tornar I8 verificavel; (2) quebrar a god-struct Deps em portas/telemetria/capacidades; (3) subir a toolchain para Rust 1.99 (rust-toolchain.toml, MSRV) e adoptar o rigor do REF/sniff-css (clippy.toml, rustfmt, deny.toml, CI supply-chain/miri); (4) resolver pendencias (tokio-util morto, task forgotten, cobertura do Logger). Foco: simplicidade, determinismo, testabilidade, seguranca.
<!-- knudge:parent epic_00ynu6q8 -->