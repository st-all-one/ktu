---
id: fact_00q8h9e8
type: fact
statement: "Skills Rust vs principios do ktu: 9 conflitos (HashMap, unwrap, should_panic, unsafe, tracing, algebraic)"
created_at: 2026-10-17T23:01:24.178Z
body_hash: f71a402b
schema_version: 2
anchors: [.agents/skills/rust/SKILL.md]
---
Revisao de .agents/skills/rust/ (Rust 1.99.1/Edition 2024) contra os principios do ktu (simplicidade, determinismo, 1.99+, gate). Alinhados: lib+thin binary, injecao por traits, proptest, release profile (lto=fat/codegen-units=1/panic=abort), ADRs, cargo deny, thiserror-em-lib. Conflitos onde o ktu ganha (NOTAS-RUST.md): N1 a skill diz HashMap default, o ktu PROIBE HashMap/HashSet (so BTreeMap/BTreeSet, determinismo); N2 exemplos usam unwrap, o ktu proibe unwrap/expect/panic em src/ e testes; N3 should_panic nao se aplica (o ktu nunca panica, panic=abort, erros sao valores); N4 capitulo unsafe/FFI nao se aplica (forbid(unsafe_code)); N5 tracing so na borda (Logger e porta zero-cost); N6 Error proprio em vez de anyhow; N7 fakes em vez de mockall; N8 flags viram enums; N9 sort() estavel onde a ordem e contrato. Rust 1.99+: adoptar let-else/gen/RPIT, String::from_utf8_lossy_owned, Vec::into_parts, format_into/NumBuffer; EVITAR f32/f64::algebraic_* (nao-deterministico, o ktu faz orcamento com floats reprodutiveis). Notas de execucao adicionadas a cada plano H0-H11. Doc: wiki/implementation_phase2/NOTAS-RUST.md.