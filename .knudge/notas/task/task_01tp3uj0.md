---
id: task_01tp3uj0
type: task
statement: "H10 — Convencoes: templates, slash, nome de sessao, AGENTS aninhados, agentes declarativos"
created_at: 2026-10-17T22:53:40.379Z
body_hash: 3f82a3d6
schema_version: 2
tags: [fase2]
results_in: [task_01k5thqh, task_00n0u20s, task_01p2aal8, task_00n15wb8]
revision: 9
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:58:29.032Z
    notes: "H10 (convencoes) concluida: directriz SYSTEM.md/APPEND_SYSTEM.md com precedencia projecto>utilizador (injectada via KtuBuilder::user_dir, porque o ktu-core e puro), AGENTS.md aninhados com descoberta deterministica e tectos (profundidade/bytes/dirs) + nearest puro, comandos do projecto (.agents/commands/*.md invocaveis por /nome args com $ARGUMENTS, ktu commands ls), agentes declarativos (.agents/agents/*.md com mode primary|subagent e tools allow/deny, fail-closed, enforcement estrutural via AgentSpec existente, nomes na descricao da tool subagent, ktu agents ls), titulo de sessao heuristico no checkpoint (ktu session ls) e frontmatter partilhado (ktu-core/src/frontmatter.rs, reutilizado pelos guardrails). Novo verbo ktu prompt para ver o prompt efectivo. Contrato em wiki/convencoes.md; plano em wiki/implementation_phase2/H10-convencoes.md (com Estado da implementacao + Prova G10/G11). Desvios com razao: receitas com settings/shell (A9), exposicao de tools + tool_search (A10) e AGENTS.md na leitura de ficheiro (A11) - os tres em ADIADOS.md com condicao de reabertura e tarefa kd propria. Gate make check verde; typos e cargo doc limpos."
  - status: success
    recorded_at: 2026-10-18T02:58:32.779Z
    notes: fecho confirmado
anchors: [wiki/backlog/consolidado.md]
status: closed
scope: issue
---
<!-- knudge:parent epic_01j7ou2p -->