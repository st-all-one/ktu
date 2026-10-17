---
id: task_005zfzx3
type: task
statement: Benchmarks com evidência (métrica + artefacto) e otimização guiada por medição
created_at: 2026-10-13T00:51:17.871Z
body_hash: 5e16911f
schema_version: 2
tags: [task]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-17T11:49:37.949Z
    notes: "Harness crates/ktu-core/benches/core.rs (harness=false) mede derive_messages (100/1k/10k eventos), log_build_1k e um turno completo (1/8 passos de tool) com provider/tools falsos; escreve wiki/benchmarks/ktu-bench.json (host, cpu, rustc, RSS, tamanho do binario). make bench. Numeros (Ryzen 5 5500U, rustc 1.99): derive_messages_100 6us, _1k 45us, _10k 534us; log_build_1k 45us; turn_1_step 5us, turn_8_steps 12us; RSS 8 MiB; binario 6,0 MB. Conclusao: nucleo em microssegundos; gargalo e o modelo. make check e make supply-chain verdes."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00fgs8ld -->