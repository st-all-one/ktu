---
id: fact_01ww33fm
type: fact
statement: H4 concluida: REPL ktu session in-process sobre a porta Host; M4 (run==REPL log) verificado
created_at: 2026-10-18T00:10:15.915Z
body_hash: ddc33a9e
schema_version: 2
anchors: [crates/ktu-cli/src/repl.rs, crates/ktu-cli/src/render.rs, crates/ktu-cli/src/session_cmd.rs]
---
H4 (task_010l8e4e, parcial) concluida. Novo `ktu session [id]` = REPL multi-turno in-process: constroi um SessionHub (a porta Host) e fala-lhe - sem logica propria (M4). Streaming dos eventos do turno via subscribe; stdout = respostas finais, stderr = transcricao (render::describe_styled) e chrome (prompt, /help). Comandos /: /help, /quit, /plan, /build, /mode, /cancel - todos mapeiam verbos da porta (set_mode/cancel/state). Multi-linha (linha com barra final), Ctrl-D sai, cor automatica (off fora de TTY), historico em .ktu/sessions/<id>.history. Gestao: `ktu session ls|rm|fork` (session_cmd.rs). M4 verificado por teste de integracao (tests/session.rs): ktu run --session a e o REPL na sessao b produzem a.jsonl == b.jsonl. Ficheiros novos: render.rs (extraido da tui), repl.rs, session_cmd.rs, actions.rs (enums de accao), run_cmd.rs (split de main.rs); cli.rs/main.rs/tui.rs editados para o limite de 400 linhas. Adiado A5 (task_00dbywbk): Ctrl-C (cancelar) e Tab (completar) exigem raw mode/signal; /compact|/resume|/fork|/undo|/redo|/memory nao tem verbo na porta. Gate: make check + typos + machete verdes; ktu-cli 43 unit + integracao.