---
id: fact_01p470ck
type: fact
statement: AGENTS.md aninhados: descoberta deterministica com tectos e nearest() puro para o mais proximo
created_at: 2026-10-18T02:58:59.306Z
body_hash: fe273383
schema_version: 2
anchors: [crates/ktu-core/src/agents_md.rs, wiki/convencoes.md]
---
H10: novo ktu-core/src/agents_md.rs. discover(fs, root) devolve Doc{path,text} com a raiz PRIMEIRO e depois os aninhados por ordem de caminho (BTreeSet + pop_first: ordem lexicografica global, deterministica), saltando dirs ocultos e uma lista de dirs de referencia/artefactos (target, node_modules, vendor, third_party, dist, build, out, REF, ref, __pycache__, Pods). Tectos obrigatorios porque um monorepo com um REF/ grande pagaria o custo em cada arranque: MAX_DEPTH 8, MAX_DIRS 512, MAX_FILE_BYTES 8 KiB, MAX_TOTAL_BYTES 32 KiB (deterministico: para de acrescentar). render(docs, root) condensa cada documento e poe cabecalho ## <caminho relativo> nos aninhados (a raiz fica sem cabecalho, compativel com o comportamento anterior). nearest(fs, root, path) sobe do directorio do ficheiro ate a raiz e devolve o primeiro AGENTS.md (a regra "o mais proximo sobrepoe"), puro e testado - fica pronto para a injecao a pedido (A11). Fail-open: ficheiro ilegivel/vazio/maior que o tecto e ignorado. Testes: 9 unit (ordem, skip, cabecalhos, nearest, tecto de profundidade, ficheiros vazios/grandes). Doc: wiki/convencoes.md §3.