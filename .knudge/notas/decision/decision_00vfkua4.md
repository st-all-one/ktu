---
id: decision_00vfkua4
type: decision
statement: Tab no REPL adiada: raw mode exige um editor de linha e gestao do terminal em volta dos logs do turno
created_at: 2026-10-18T14:25:16.662Z
body_hash: 0d573678
schema_version: 2
anchors: [crates/ktu-cli/src/repl.rs, wiki/implementation_phase3/F2-repl-interactivo.md]
---
F2 (complemento). Decisao: entregar os comandos / e o Ctrl-C e NAO implementar a Tab agora. Motivo: completar / e caminhos exige raw mode (crossterm) e, com ele, gerir o terminal em volta dos logs do turno - o REPL escreve em stderr durante o turno (transcricao dos eventos, steering, hooks) enquanto o utilizador escreve, pelo que raw mode obriga a guardar/restaurar o cursor e redesenhar a linha; e um editor de linha completo (eco, backspace, historico, redesenho), nao um if. Alternativas descartadas: (a) rustyline/completer como dependencia nova - o projecto evita dependencias e o REPL tem uma thread de stdin propria (a linha-a-linha e o caminho dos gates); (b) completar sem raw mode - impossivel, o terminal so entrega a linha no Enter; (c) tab que so imprime as opcoes (sem editar a linha) - nao completa nada, e ruido. Gatilho mantido: uso interactivo real a pedir o atalho. Tarefa propria: task_002em5xu.