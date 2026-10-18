---
id: fact_01b7qqzd
type: fact
statement: H12: provider scripted (guiao JSONL) e os tres defeitos reais que os gates encontraram
created_at: 2026-10-18T11:33:39.703Z
body_hash: a1f4d23a
schema_version: 2
anchors: [crates/ktu-cli/src/provider_script.rs, crates/ktu-cli/src/batch.rs, crates/ktu-core/src/facade/builder.rs, scripts/run-server.sh]
---
Para os gates correrem cadeias longas sem inferencia (o modelo de referencia mediu ~8 min por completion neste host CPU-only), a CLI ganhou `--provider scripted --script <ficheiro.jsonl>` (crates/ktu-cli/src/provider_script.rs): uma Completion por linha, cursor GLOBAL AO PROCESSO (cada passo do modelo consome a linha seguinte, mesmo entre turnos), o que permite --batch --sequential e o REPL percorrerem uma tarefa real deterministica; o parsing puro vive em ktu_core::testing::parse_script e o EchoProvider do CLI passou para ktu_core::testing (partilhado com o exemplo ktu-embed, o que torna a prova M4 do G12 byte-a-byte). Os gates encontraram e corrigiram tres defeitos reais: (1) --batch --sequential NAO acumulava contexto - cada turno reabria a sessao em modo Fresh e so o ultimo turno ficava no log, contra o contrato "uma sessao, N turnos" - corrigido em batch::turn_config (retoma a partir do 2.o item) e travado por crates/ktu-cli/tests/batch.rs; (2) os guardrails GLOBAIS nao eram carregados, pelo que M5 (piso aditivo) era vacuidade - facade::builder e facade::evaluate_fact passam a carregar <user_dir>/guardrails ANTES das regras do projecto (a cadeia so consulta a regra seguinte se a anterior deixar passar) e ha teste em facade/tests/guardrails.rs; (3) scripts/run-server.sh nao encontrava as libllama* (LD_LIBRARY_PATH derivado do directorio do binario).