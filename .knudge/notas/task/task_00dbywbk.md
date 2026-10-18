---
id: task_00dbywbk
type: task
statement: "A5 - REPL: Ctrl-C, Tab e comandos / sem verbo na porta (pos-H4)"
created_at: 2026-10-18T00:10:04.206Z
body_hash: 1238f19b
schema_version: 2
tags: [fase3]
anchors: [crates/ktu-cli/src/repl.rs]
scope: task
---
Adiados da H4. (a) Ctrl-C para cancelar o turno e (b) Tab para completar exigem raw mode (crossterm) e/ou stdin assincrono (tokio features signal + io-std) - o REPL actual le linha a linha sem raw mode. (c) /compact, /resume, /fork, /undo, /redo, /memory nao tem verbo na porta Host, e o REPL nao inventa logica (M4). Reabrir quando a porta ganhar os verbos e houver raw mode/signal.
<!-- knudge:parent task_016wnwcs -->