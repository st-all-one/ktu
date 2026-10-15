---
id: fact_003y05yn
type: fact
statement: "O ktu corre end-to-end contra o Ornith-1.5-9B real: glob->read_file->resposta, reasoning em campo proprio"
created_at: 2026-10-13T02:39:57.672Z
body_hash: 88ef3a46
schema_version: 2
anchors: [crates/ktu-provider/src/llama.rs]
---
Evidência: ktu run --base-url http://127.0.0.1:8081 --session real1 'le a.txt' devolveu a resposta correta; o log .ktu/sessions/real1.jsonl tem turn_start, assistant com reasoning proprio (101/38/62 chars) e tool_calls (glob, read_file), guard_decision allow, tool_result ok, turn_end end_turn. ktu providers doctor --probe deu estado ok (qwen35, file_type 15, n_ctx 32768).