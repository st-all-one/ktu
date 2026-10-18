---
id: fact_00ezedll
type: fact
statement: "Fase 2 criada: epico epic_01j7ou2p, 12 issues (H0-H11), 48 tarefas, ancoradas ao backlog"
created_at: 2026-10-17T22:56:19.312Z
body_hash: 5a4621dd
schema_version: 2
anchors: [wiki/implementation_phase2/README.md]
---
Nova rodada de planos em wiki/implementation_phase2/ (README + H0-H11), com o epico epic_01j7ou2p no knudge. Tese: a Fase 1 provou usabilidade (G4); a Fase 2 garante minimo/kernel/seguro/independente-do-caminho/deterministico/project-bound, servindo 4 usos (crate, pipeline, conversa longa CLI, config do projeto); o CLI e o front primario. Sequencia estritamente ordenada: H0 superficie do CLI (task_00qn6s14, 5 tarefas); H1 ktu init (task_01v41q8n, 4); H2 config global/local (task_00tx0uvu, 5); H3 efemero+lock (task_017rrpwn, 3); H4 REPL ktu session (task_010l8e4e, 4); H5 fiabilidade do loop (task_00htbspv, 5); H6 controlo deterministico (task_01s9n99d, 3); H7 eixo de memoria no knudge (task_00104ugh, 5); H8 embeddings opcionais (task_01t8rm0l, 4); H9 crate facade (task_00dkc6dk, 3); H10 convencoes (task_01tp3uj0, 4); H11 governanca (task_01ehkt9h, 3). Total 12 issues + 48 tarefas (tag fase2). Cada doc Hn-*.md tem objetivo, ancora ao backlog, pre-condicoes, passos, tarefas kd (IDs), gate/testes e riscos. Invariantes novos travados por teste: M4 dois-caminhos-uma-sessao, M3 auth-nao-ecoa, M5 guardrail-global-nao-removivel. Gate por issue: make check + make supply-chain. Metodo: simples primeiro, atraves da porta, um so standard, teste antes de fechar, ancora no backlog.