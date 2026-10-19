---
id: fact_005izxga
type: fact
statement: "Fase 3 criada: epico epic_00j1cgo3 com 7 issues (F1-F7), 11 tarefas e gates G16-G22"
created_at: 2026-10-18T13:40:42.339Z
body_hash: f4bc9086
schema_version: 2
revision: 2
anchors: [epic.md]
---
Apos fechar a Fase 2, os adiados por YAGNI foram reavaliados CONTRA O CODIGO e a Fase 3 foi criada: epico epic_00j1cgo3 (wiki/implementation_phase3/README.md). 7 issues: F1 steering duravel/idempotente (M7) task_00ly3qez, F2 REPL interativo task_01mrz94g, F3 recursos globais + piso visivel (M8) task_004bvdzb, F4 fiabilidade de edicao task_00sb14zh, F5 migracoes de config task_00g0us7a, F6 ktu self setup task_00odyng2, F7 travoes (links/doc + knudge na CI) task_01nopvll; 11 tarefas. Cada item tem plano (wiki/implementation_phase3/Fn-*.md com objetivo, estado verificado datado, passos, tarefas kd, gate e riscos), GATILHO declarado e gate proprio (G16-G22, corridos pelo mesmo runner xtask gates / make gates da Fase 2, com artefacto em wiki/gates/). Invariantes novos: M7 (uma mensagem admitida e entregue exactamente uma vez, mesmo apos crash/resume) e M8 (nada que o runtime usa e invisivel - todo o recurso efectivo aparece num ls com origem). Premissas expiradas corrigidas na reavaliacao: A1 dizia que o runtime nao carregava guardrails (falso desde a H12) e A5 dizia que /compact,/fork,/memory nao tinham verbo na porta (falso: os verbos existem na CLI, faltava o mapeamento). Ficaram FORA: A4 (.ktu/logs, fechado como abandoned - e um caminho, nao trabalho), A9-A11 (ja implementados) e D8-D12 (sem consumidor). O epico antigo epic_00ey9lmk foi absorvido e fechado; a lista verificada e uma so (wiki/pendencias.md secao 2).