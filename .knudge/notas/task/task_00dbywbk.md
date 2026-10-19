---
id: task_00dbywbk
type: task
statement: "Comandos / que faltam no REPL: /compact, /fork, /memory, /resume (mapeamento de verbos existentes)"
created_at: 2026-10-18T00:10:04.206Z
body_hash: 452cfe91
schema_version: 2
tags: [fase3]
revision: 5
outcomes:
  - status: success
    recorded_at: 2026-10-18T14:25:04.285Z
    notes: "Comandos / que faltam no REPL: /compact (Cmd::Compact no actor: corta o contexto, grava log + checkpoint), /fork (session_cmd::copy, byte-a-byte), /memory (memory_cmd::list_text), /resume (create_session + subscribe + state). Novo repl_slash.rs (delegacao, sem logica no REPL); /help actualizado; SessionState ganhou keep_from. Gate G17 verde (checks 2-7)."
anchors: [wiki/implementation_phase3/F2-repl-interactivo.md]
status: closed
scope: task
---
Adiados da H4. (a) Ctrl-C para cancelar o turno e (b) Tab para completar exigem raw mode (crossterm) e/ou stdin assincrono (tokio features signal + io-std) - o REPL actual le linha a linha sem raw mode. (c) /compact, /resume, /fork, /undo, /redo, /memory nao tem verbo na porta Host, e o REPL nao inventa logica (M4). Reabrir quando a porta ganhar os verbos e houver raw mode/signal.
<!-- knudge:parent task_01mrz94g -->