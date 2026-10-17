---
id: fact_01lvl8eo
type: fact
statement: "Benchmark do nucleo (make bench): derive_messages 6us/100, 45us/1k, 534us/10k; turno 8 passos 12us"
created_at: 2026-10-17T11:49:38.056Z
body_hash: 314b0140
schema_version: 2
anchors: [crates/ktu-core/benches/core.rs, wiki/benchmarks/README.md]
---
Harness crates/ktu-core/benches/core.rs (harness=false, sem modelo) escreve wiki/benchmarks/ktu-bench.json com host/cpu/rustc/RSS/tamanho do binario. Mede derive_messages (100/1k/10k), log_build_1k e run_turn com FakeProvider/FakeTools (1/8 passos). Numeros numa corrida de 2026-10-17 (Ryzen 5 5500U): derive_messages_100 6us, _1k 45us, _10k 534us; log_build_1k 45us; turn_1_step 5us, turn_8_steps 12us; RSS 8 MiB; binario 6,0 MB. Fora do make check (ruidoso); make bench a pedido. Interpretacao em wiki/benchmarks/README.md.