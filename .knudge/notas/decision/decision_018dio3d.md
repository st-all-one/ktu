---
id: decision_018dio3d
type: decision
statement: "Ordem de prioridade: 1) funcionar (sessão/contexto/cache), 2) qualidade, 3) performance (terciária)"
created_at: 2026-10-13T00:51:41.325Z
body_hash: 034d570f
schema_version: 2
tags: [ktu, metodo]
anchors: [plan.md]
---
Por quê: a 1.ª tentativa (REF/katu) optimizou antes de funcionar; 70k linhas e zero sessões operáveis.
Evidência: .katu/transcript.md; REF/katu com MVK e Q-21 fechados mas sem sessão estável.
Consequência: o mais simples e óbvio primeiro; complexidade só depois; optimização só com número medido.