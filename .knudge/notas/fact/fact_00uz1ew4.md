---
id: fact_00uz1ew4
type: fact
statement: Revisão: G0-G3 fechados com evidência e G4 (task_010xrtzt) verde; make check verde
created_at: 2026-10-16T08:21:23.673Z
body_hash: 0c06fb48
schema_version: 2
anchors: [epic.md]
---
Verificação por evidência: as 28 tasks de G0-G3 estão closed com outcome e âncoras; o issue G4 task_010xrtzt está closed com veredicto de equivalência (wiki/quality_test/ktu-veredicto.md) e artefactos p*/ktu-*.md com frontmatter de condições (Q1-Q3 same_session, Q4 compacted=true, sonda de persistência=passe). make check (fmt + clippy -D warnings + test + check-layers + file-length) verde nesta revisão. O alvo xtask quality --dry-run reproduz o plano p1-p4. Caveats declarados: a corrida usou provider remoto opencode-go/longcat em vez do llama.cpp local (por indisponibilidade de fundos) e a Q2 propôs REPL em vez de --human.