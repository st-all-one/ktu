---
id: decision_00btk0j5
type: decision
statement: Gate inegociável: alcançar o quality_test antes de prosseguir para qualquer fase seguinte
created_at: 2026-10-13T00:51:41.184Z
body_hash: 2fc55b47
schema_version: 2
tags: [ktu, gate, quality]
anchors: [plan.md]
---
Por quê: o objectivo do ktu é atingir o mesmo grau de qualidade dos agentes de referência (goose/pi/opencode) na bateria p1-p4, e persistência entre sessões.
Evidência: wiki/quality_test/ (p1-p4, isolados por sujeito) e plan.md §3/F5.
Consequência: G4 bloqueia G5-G8. Sem equivalência, não se avança — corrige-se qualidade, não se acrescenta sistema.