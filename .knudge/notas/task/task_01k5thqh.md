---
id: task_01k5thqh
type: task
statement: Templates/receitas/comandos com shell (41/G1) + slash commands (20)
created_at: 2026-10-17T22:54:42.101Z
body_hash: 0878ee05
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:58:23.429Z
    notes: "Comandos/templates do projecto (o mecanismo de receitas) + slash: novo ktu-core/src/command.rs (Command{name,description,prompt,path}, parse com frontmatter opcional, load fail-open com problemas reportados, find, render com $ARGUMENTS ou acrescento) + frontmatter partilhado extraido para ktu-core/src/frontmatter.rs (split/parse/body/unquote/required) e guardrails.rs passa a usar-lo (sem duplicar parser); REPL resolve /nome args contra .agents/commands/ e /help lista os comandos do projecto (Slash::Custom{name,args}); novo verbo ktu commands ls (texto + --json com problems). Testes: unit (parse/render/load) + ktu-cli/tests/commands.rs (3: ls, /nome com argumentos renderizados no log, /help + desconhecido). ADIADO com razao: receitas com settings/parameters/sub_recipes e shell embutido (A9/task_00mtxl9i) - o shell duplicaria a tool bash ja guardada, e nenhum workflow repetido exige settings."
anchors: [wiki/backlog/consolidado.md]
status: closed
scope: task
---
<!-- knudge:parent task_01tp3uj0 -->