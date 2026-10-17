---
id: risk_00zsx9hy
type: risk
statement: "Arquitetura de 2 threads (T1 kernel/T2 tools, mpsc) do plan §2.2 nao esta implementada nem tem task no epico"
created_at: 2026-10-17T00:35:51.326Z
body_hash: 75a0f7e6
schema_version: 2
anchors: [crates/ktu-core/src/agent/mod.rs, plan.md]
---
Evidencia: grep em crates/ nao encontra thread::spawn/mpsc/oneshot/tokio::spawn/JoinHandle; o loop chama deps.tools.execute(call).await na mesma task assincrona. tokio-util (feature rt) esta declarado no workspace mas nenhum crate o usa. Existe a fronteira de abstracao (ToolExecutor), nao a separacao de execucao. I8 (kernel e thread de tools nunca partilham estado com o front-end) fica parcialmente inaplicavel. Consequencia: adicionar task explicita (ex.: G6 ou G5.5) que corra as tools numa task/thread dedicada via mpsc, com teste de que um panic/timeout de tool nao derruba o kernel.