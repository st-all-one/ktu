---
id: task_01p2aal8
type: task
statement: AGENTS.md aninhados (52) + agentes declarativos (51)
created_at: 2026-10-17T22:54:42.808Z
body_hash: 21d702a6
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:58:23.756Z
    notes: "AGENTS.md aninhados + agentes declarativos: novo ktu-core/src/agents_md.rs (discover deterministico - raiz primeiro, aninhados por caminho com cabecalho ## <rel>, tectos: profundidade 8, 8 KiB/ficheiro, 32 KiB total, 512 dirs; saltados dirs ocultos e target/node_modules/vendor/dist/build/out/REF/Pods/__pycache__; nearest() puro para o mais proximo) integrado no system_prompt_in; novo ktu-core/src/agent_file.rs (ToolPolicy com sintaxe {\"*\": false, \"read_*\": true} ou lista, exacto>wildcard>default, ToolAccess/AgentMode como enums (N8), load fail-closed nomeando o ficheiro, apply que substitui o primario e regista subagentes por nome com validacao de tools desconhecidas/dois primaries/sem tools); AgentId ganhou Named(Box<str>) (deixou de ser Copy) e AgentRegistry with_named/with_primary/subagent_names; SubagentHost::agents + subagent::spec_with poem os nomes na descricao da tool; novo verbo ktu agents ls; erro de subagente desconhecido lista os disponiveis. Testes: 6 unit + ktu-cli/tests/agents.rs (2, incluindo a prova estrutural: primario declarativo sem read_file => negado: capability no log). ADIADO: injecao de AGENTS.md na leitura de ficheiro (A11/task_01963li2) e model/hidden/color."
anchors: [wiki/backlog/agents-md-aninhados.md]
status: closed
scope: task
---
<!-- knudge:parent task_01tp3uj0 -->