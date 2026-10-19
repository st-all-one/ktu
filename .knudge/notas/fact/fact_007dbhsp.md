---
id: fact_007dbhsp
type: fact
statement: F6 fechado sem plano: ktu init ja cobre AGENTS.md+skill; ktu nao tem MCP; G21 fica sem gate
created_at: 2026-10-18T17:20:37.387Z
body_hash: 7c1711c3
schema_version: 2
anchors: [wiki/implementation_phase3/F6-self-setup.md, wiki/pendencias.md, crates/ktu-cli/src/init.rs]
---
F6/D3 fechado sem plano (nao e uma rejeicao da pendencia: D3 fica no dossier com o gatilho afinado). Verificado no codigo: `ktu init` (crates/ktu-cli/src/init.rs:82-145) escreve so dentro do root - layout .agents/ + .ktu/, notes.jsonl, guardrails seed, SKILL.md, config.toml, bloco `<!-- ktu:start -->` no AGENTS.md (git/agents.rs:23, com versao do protocolo) e .git/info/exclude + .gitattributes; nunca sai do projeto nem toca na config global. O `ktu self setup` (F6) escreveria em ficheiros de OUTRO produto (formato/localizacao do cliente). Como o AGENTS.md e a convencao cross-tool (pi, Claude Code, Cursor, Codex leem-no) e a skill ja vai para .agents/skills/, a integracao que o setup venderia ja existe para esses clientes; e o ktu nao tem servidor MCP (so o knudge tem knudge-mcp), pelo que nao ha caminho novo a configurar. O precedente do knudge (crates/knudge/crates/knudge-cli/src/commands/self_cmd/mod.rs:21, CLIENTS = claude/cursor/codex/pi) grava uma receita JSON em .knudge/setup/<cliente>.json e NAO toca nos ficheiros do cliente - ou seja, ate o precedente se reduz a emitir dados. Ficheiros: wiki/implementation_phase3/F6-self-setup.md (reescrito como registo de fecho com a analise), README.md (tabela + seccao 4), GATES.md (G21 sem gate - o numero nao se reutiliza), ADIADOS.md (A3), pendencias.md (D3 com gatilho afinado), backlog/receitas.md, STAGING.md. knudge: task_01oo6bry e task_00odyng2 fechadas como abandoned; epico 11/12 (so a Tab, task_002em5xu).