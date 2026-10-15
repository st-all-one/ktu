---
id: risk_01a6bzxd
type: risk
statement: O arranque real do modelo fica pendente: cmake esta ausente no host
created_at: 2026-10-13T02:05:12.567Z
body_hash: 1e13fa1d
schema_version: 2
revision: 2
anchors: [scripts/setup-model.sh]
status: forgotten
---
Evidência: command -v cmake => ausente; git/hf/nproc presentes e a rede para huggingface.co responde. Consequência: o build do fork PrismML exige 'sudo dnf install cmake' (ou binario pre-compilado) antes de ./scripts/setup-model.sh poder construir o llama-server.