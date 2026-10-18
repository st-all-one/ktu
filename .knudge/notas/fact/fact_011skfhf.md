---
id: fact_011skfhf
type: fact
statement: Planeados ktu init (marcadores idempotentes + git gerido) e ktu memory drain service (embeddings opcionais)
created_at: 2026-10-17T22:33:53.879Z
body_hash: 4c10b6ed
schema_version: 2
anchors: [wiki/cli-e-config.md]
---
wiki/cli-e-config.md §4/§5 planeia duas pecas novas. ktu init (wiki/backlog/ktu-init.md), modelado no kd init/onboard: idempotente; cria layout .agents/ + .ktu/; copia skill .agents/skills/ktu/SKILL.md (protocolo do agente, como a skill kd D162), template .agents/config.toml e bloco gerido no AGENTS.md; ajusta git SEMPRE em .git/info/exclude e .gitattributes (nunca .gitignore, D30); modos --git-tracked (exclui so o derivado/runtime) e --git-excluded (exclui tudo .agents/+.ktu/, local-only), com reversao ao mudar de modo; --force (so reescreve config, nunca a memoria); --no-prompt; relatorio novo/atualizado/inalterado. Marcadores idempotentes (tatica do knudge): AGENTS.md <!-- ktu:start -->/<!-- ktu:end --> + <!-- ktu:version: N -->; .gitattributes # ktu:start/# ktu:end; upsert casa a linha inteira (apos trim), reexecutar nao duplica. .gitattributes: eol=lf nas notas/eventos, merge=union no log de eventos append-only, binary no derivado. Fora de repo git: no-op. ktu memory drain service (wiki/backlog/memory-drain-service.md): embeddings locais OPCIONAIS e incentivados; mesmo modelo do knudge (embeddinggemma-2 Q8_0, porta 8889); --install/--subscribe/--unsubscribe/--status/--uninstall/--reconcile/--dry-run/--every/--port/--model/--no-deps/--keep-model/--remove-model; --remove-model usa o trash. Superficie do kd medida: areas por verbo, --params <JSON> (D147), --batch JSONL (D147), envelope, exit codes, self, config, prime. Backlog: 132 paginas (2 novas: ktu-init, memory-drain-service).