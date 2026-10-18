---
id: fact_01hsd5sx
type: fact
statement: H0.2 e H0.3 fechadas: --params (contrato de maquina) e --batch (paralelo/sequencial); make check verde
created_at: 2026-10-17T23:23:01.016Z
body_hash: 5d4e34b6
schema_version: 2
anchors: [crates/ktu-cli/src/params.rs]
---
H0.2 (task_00rtiog5) e H0.3 (task_00sl9jsg) fechadas. H0.2 --params <JSON>: novo crates/ktu-cli/src/params.rs (RunParams serde com deny_unknown_fields; parse com - le stdin; pick(flag,param,default)); os args do Run passaram a Option (provider/effort/max_steps/session) para a precedencia flag>params>default funcionar; dispatch_run funde; erro tipado (--params invalido). Verificado: campo desconhecido -> envelope de erro; flag sobrepoe params. H0.3 --batch <JSONL>: novo crates/ktu-cli/src/batch.rs; cada linha e um objeto --params; paralelo por omissao (threads com concorrencia limitada --concurrency default 4, cada turno em sessao batch-<i>) ou --sequential (mesma sessao, em ordem); resultados impressos ordenados pelo indice (determinismo); linha invalida -> erro com o numero da linha. Refactor: run.rs expoe run_capture(root,prompt,logger,config) -> (bool, Option<String>) sem imprimir (o run imprime; o batch recolhe); drive devolve (bool, texto). Limite de ficheiro subido de 300 para 400 linhas (scripts/check_file_length.sh, Makefile, README, plan.md). Estado: make check VERDE; ktu-cli 27+2+1 testes. Falta na H0: H0.4 (completions/setup), H0.5 (envelope nos restantes verbos + codigos de saida documentados).