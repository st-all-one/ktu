---
id: fact_01fvaozi
type: fact
statement: "O host de desenvolvimento do ktu e CPU-only: Ryzen 5 5500U (6c/12t), 17 GiB RAM, iGPU Radeon, sem GPU dedicada"
created_at: 2026-10-13T01:36:14.640Z
body_hash: 3de3f150
schema_version: 2
anchors: [plan.md]
---
Evidência: lscpu/free/lspci (Fedora 44, kernel 7.2.9); ~7 GiB de RAM livre com o sistema em uso; 378 GB de disco livre em /home. Consequência: o decode e limitado por largura de banda de memoria; nao ha VRAM dedicada para offload.