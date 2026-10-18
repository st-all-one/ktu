---
id: task_00n0u20s
type: task
statement: Nome automatico de sessao (84)
created_at: 2026-10-17T22:54:42.461Z
body_hash: 40e3be6d
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:58:23.586Z
    notes: "Nome automatico de sessao: novo ktu-core/src/session/title.rs (title_from: <=8 palavras/60 chars, tira tags XML, cercas de codigo e pontuacao de extremidade; title_of: primeiro TurnStart do log; MAX_WORDS/MAX_CHARS publicos) + checkpoint.title (serde default, retro-compativel) + session_store deriva o titulo ao gravar o checkpoint + ktu session ls mostra-o (ultima coluna; --json em title). O titulo e indice DERIVADO (o log continua a ser a verdade, I3) - recalculavel. Testes: 5 unit + ktu-cli/tests/session.rs::ls_shows_the_title_derived_from_the_first_prompt (inclui retoma que nao muda o titulo)."
anchors: [wiki/backlog/nome-sessao.md]
status: closed
scope: task
---
<!-- knudge:parent task_01tp3uj0 -->