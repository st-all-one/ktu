---
id: decision_01yhrzu0
type: decision
statement: O G2 entrega scripts idempotentes de setup/arranque do modelo de referencia e flags CPU-only
created_at: 2026-10-13T02:05:12.445Z
body_hash: cc5c54cf
schema_version: 2
superseded_by: decision_009zx5ir
revision: 2
anchors: [scripts/run-server.sh]
status: superseded
---
Evidência: scripts/setup-model.sh descarrega Ternary-Bonsai-2-27B-PTQ1_0.gguf (hf) e constroi o llama-server do fork PrismML-Eng/llama.cpp; scripts/run-server.sh arranca com -ngl 0 -fa on -c 65536 --jinja e sampling do thinking mode. Consequência: o arranque real depende de cmake instalado e ~6 GB de download.