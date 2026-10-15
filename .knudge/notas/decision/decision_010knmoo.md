---
id: decision_010knmoo
type: decision
statement: "Schema fisico de notes.jsonl: uma linha JSON por nota com {frontmatter:<mapa de chaves canonicas>,body:<texto>}"
created_at: 2026-10-13T02:45:17.282Z
body_hash: bde59a22
schema_version: 2
anchors: [plan.md]
---
Por quê: preserva o contrato de bytes do knudge-core (Frontmatter/Value) sem reimplementar o schema. As chaves canonicas sao as CANONICAL_KEYS do knudge (id,type,statement,created_at,schema_version,tags,anchors,...); opcionais omitidos (nunca null — o codec JSON do knudge rejeita null). Consequência: reconstrucao exacta via Frontmatter::from_value + Note::new; eventos em .agents/memory/events/*.jsonl; indice derivado reconstruivel.