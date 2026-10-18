---
id: decision_01o53bco
type: decision
statement: "Aprovado: knudge e o eixo unico de memoria, agent-agnostic, fonte estruturada independente do agente"
created_at: 2026-10-17T22:50:21.042Z
body_hash: 5bd6d1a4
schema_version: 2
anchors: [wiki/knudge-eixo-memoria.md]
---
Aprovada a visao (wiki/knudge-eixo-memoria.md): o knudge e o eixo unico de memoria/conhecimento/tarefas, com um standard independente do agente (schema do knudge: type/scope/statement/body/anchors/tags/provenance). Principio: o agente e substituivel, a memoria nao. Um so standard; estruturado (notas/claims) e nao despejo de log; consulta inteligente (BM25+ancoras+grafo+RRF), nao sequencial; multi-agente por construcao. Tres vias de integracao: nativa (agente escreve notas no schema, como o ktu hoje), adaptada (Source JSONL read-only le o store do agente), projetada (o agente projeta o seu log cru em notas derivadas read-only). Regra de ouro: o knudge indexa (derivado), o agente possui a fonte; nunca duas verdades. Concorrencia multi-agente JA resolvida pelo knudge: lock advisory por ficheiro (D23-D25), ids enderecados por conteudo (D26/D28/D95), commit nota->evento (D21), merge=union no log (D26/D28/D31), proveniencia agent/activity (D207). Falta (pequeno, portas e filtros, sem motor novo): (1) Source JSONL read-only; (2) ask --agent/--activity como filtros; (3) contrato de projecao (doc); (4) kd ai como area fina; (5) registo de agentes (.knudge/agents.toml) so com >=2 consumidores. Nao acoplar o knudge a schemas de agentes (o contrato e o documento {frontmatter,body}, nao o evento). Checklist de integracao de um novo agente documentado. Paginas: wiki/knudge-eixo-memoria.md (nova), wiki/estudo-kd-ai.md atualizado, wiki/backlog/kd-ai.md atualizado.