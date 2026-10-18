---
id: decision_00vvp7k6
type: decision
statement: Reverter: drain service (embeddings locais) passa a opcional e incentivado no ktu
created_at: 2026-10-17T22:33:53.441Z
body_hash: d444c5f9
schema_version: 2
anchors: [wiki/backlog/memory-drain-service.md]
---
A decisao anterior "nao adoptar o servico de embeddings (drain service)" fica revertida a pedido do dono. Justificacao: o ganho de recall com embeddings e expressivo (medido no knudge); o ktu e hoje lexical (BM25+ancoras). Nova decisao: o canal semantico passa a parte OPCIONAL e INCENTIVADA do ktu, com ktu memory drain service --install (equivalente ao kd drain service --install): agendador systemd/launchd + servidor de embeddings persistente (llama.cpp + GGUF) + cadastro do projeto; mesmo modelo do knudge (google/embeddinggemma-2, embeddinggemma-2-Q8_0.gguf), porta 8889, --every 1h. Ciclo de vida: --status (default), --subscribe/--unsubscribe, --reconcile, --uninstall (--keep-model default; --remove-model move para trash, nunca rm), --dry-run. Fila derivada: --digest drena em lotes; --force apaga .idx/ e reindexa. Degradacao graciosa: sem worker, o canal semantico desliga sem aviso; o recall lexical continua. Uma so verdade: memory.toml projeta no knudge. O que NAO muda: o binario kd e o knudge-mcp continuam fora como deps de runtime (o ktu tem a sua porta; MCP e YAGNI §0). Pagina: wiki/backlog/memory-drain-service.md; knudge-embeddings.md (G11/121) atualizado.