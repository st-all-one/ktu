---
id: task_01xdbyx0
type: task
statement: "Preparar o modelo Ornith-1.5-9B-GGUF Q4_K_M e arrancar o llama-server stock (CPU, -c 32768)"
created_at: 2026-10-13T01:36:51.178Z
body_hash: 381b693a
schema_version: 2
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:39:50.325Z
    notes: "Modelo e servidor reais: descarregado Ornith-1.5-9B-Q4_K_M.gguf (5,78 GB) e construido o llama-server stock (cmake/gcc-c++) em .ktu/llama.cpp. Servidor na 8081 (a 8080 estava ocupada por outro modelo): -ngl 0 -fa on -c 32768 --cache-type-k/v q8_0 --jinja, sampling do modelo. ktu providers doctor --probe: reachable true, model_path Ornith, n_ctx 32768, GGUF version 3, architecture qwen35, file_type 15, probe_ok true, estado ok. Turno real (ktu run --base-url 8081): o agente fez glob -> read_file -> resposta correta sobre a.txt; log da sessao registou reasoning em campo proprio (101/38/62 chars), guard_decision allow, tool_result ok; persistido em .ktu/sessions/real1.jsonl. make check verde (71 testes)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->