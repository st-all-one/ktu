---
id: task_018qh88b
type: task
statement: ktu providers doctor valida o binario do fork PrismML e o ficheiro ternary antes de servir
created_at: 2026-10-13T01:36:51.089Z
body_hash: 910c79f0
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-13T02:04:24.484Z
    notes: "ktu providers doctor valida o ficheiro: ktu-provider/src/gguf.rs le o cabecalho GGUF (magic, versao, tensor_count, KV) e extrai general.architecture e general.file_type sem carregar tensores; 2 testes unit (le file_type; recusa nao-GGUF). O fork e inferido pelo servidor ter carregado o pack ternary (so o fork o aceita) + sonda funcional de tool-calling; doctor imprime 'pack ternary — exige o fork PrismML' e falha se a sonda nao devolver nada. make check verde (71 testes)."
anchors: [plan.md]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->