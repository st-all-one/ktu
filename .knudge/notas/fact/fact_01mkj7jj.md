---
id: fact_01mkj7jj
type: fact
statement: Eixo de memoria: knudge ja tem concorrencia (lock+ids por conteudo+union) e RRF; falta Source JSONL e ask --agent
created_at: 2026-10-17T22:50:21.533Z
body_hash: bba29c8a
schema_version: 2
anchors: [wiki/knudge-eixo-memoria.md]
---
Medicao do knudge v0.6.1 para a visao do eixo agent-agnostic (wiki/knudge-eixo-memoria.md). Ja existe: store/lock.rs (lock advisory por ficheiro, O_CREAT|O_EXCL, stale 30s, reclaim por rename, retry+jitter, ordem container->nota, D23-D25); store/commit.rs (nota primeiro, evento depois, D21); schema/id.rs (id = hash(type+separador+statement normalizado), dedup sob merge, D26/D28/D95); git/attributes.rs (merge=union no log append-only, D26/D28/D31); retrieval/ (BM25+ancoras+grafo+fusao RRF k=60, pesos lexical/anchor/semantic/ppr, D39/D41/D81); schema/provenance.rs (entity/activity/agent PROV-lite, D207; flags --agent/--activity no kd write); corpus/mod.rs (le notas/ uma vez -> Index+Graph, paralelo determinista); knudge.rs (Knudge::builder().knowledge_dir configuravel, D214). Falta: (1) Source JSONL read-only (agentes com store proprio; reusa Note/Frontmatter/jsonl::json); (2) ask --agent/--activity como filtros de consulta (proveniencia ja gravada); (3) contrato de projecao (doc); (4) kd ai (area fina posicional, default consulta); (5) registo .knudge/agents.toml so com >=2 agentes. O standard do contrato: type/scope/statement/body/anchors/tags/provenance/status; uma afirmacao por nota; ask antes de write (dedup <0.75 cria, 0.75-0.92 funde, >=0.92 rejeita).