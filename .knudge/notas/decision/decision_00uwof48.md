---
id: decision_00uwof48
type: decision
statement: check-links fiel ao CommonMark e cargo doc no portao (medido ~4 s); knudge em job proprio da CI
created_at: 2026-10-18T15:31:45.698Z
body_hash: 28b6effc
schema_version: 2
anchors: [xtask/src/links.rs, Makefile, .github/workflows/ci.yml]
---
Decisao (F7). (1) O check-links segue o essencial do CommonMark em vez de uma heuristica propria: crases sem fecho do mesmo tamanho sao texto literal e crases escapadas nao abrem codigo. Alternativas descartadas: varrer sem tirar codigo (os exemplos `](...)` na documentacao davam falsos positivos) e truncar o ficheiro na primeira crase solta (a versao inicial: tres ficheiros reais da wiki tem crases soltas/escapadas e ficavam por verificar — um falso negativo silencioso, o pior modo de falha de um gate). (2) O cargo doc entra no make check em vez de alvo separado porque foi medido: ~4 s quente e ~34 s com alvo proprio vazio, contra o limiar declarado de 60 s; a regra do metodo de medicao e medir antes de decidir, e o numero decidiu. (3) O portao do knudge corre num job proprio da CI (rust-cache sobre crates/knudge) porque e um workspace independente com target proprio; o G22 prova as duas metades (wiring no Makefile/workflow e a suite a passar), em vez de so afirmar que o passo existe. (4) A ancora inexistente e aviso, nao erro: o slug do GitHub deriva do titulo e uma mudanca de titulo nao deve quebrar o portao.