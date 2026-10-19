---
id: decision_00ks8pbk
type: decision
statement: "Fase 3 planeada mas condicionada ao gatilho (cada item so arranca com sintoma), com gates proprios"
created_at: 2026-10-18T13:40:43.479Z
body_hash: 21d2a1de
schema_version: 2
anchors: [wiki/implementation_phase3/README.md, wiki/adr/0019-fase-3-planeada-mas-condicionada-ao-gatilho.md]
---
Decisao (ADR 0019 em wiki/adr/): ao fechar a Fase 2 ficaram 11 itens adiados por YAGNI, em duas listas da mesma verdade (ADIADOS.md e pendencias.md secao 2) com premissas ja expiradas. Optou-se por PLANEAR tudo o que vale a pena e EXECUTAR so com gatilho: wiki/implementation_phase3/ (epic_00j1cgo3) da a cada item plano, gate (G16-G22) e tarefa kd, e cada item declara o gatilho que o abre (F1 front multi-cliente ou requisito de nao perder mensagem; F2 uso interactivo; F3 segundo projecto; F4 paralelismo de tools; F5 schema v2; F6 receitas de cliente; F7 ja disparou - PM-0003/L1). A lista VERIFICADA e uma so (pendencias.md secao 2); a Fase 3 e o plano de execucao, nao uma segunda lista. Alternativas descartadas: (a) executar ja os onze itens porque a fase fechou - contradiz o YAGNI do plan.md secao 0 (a fila de mutacao, por exemplo, so faz sentido depois do paralelismo de tools); (b) nao planear ate aparecer sintoma - o sintoma chega sob pressao e o desenho sai apressado; (c) manter as duas listas - ja divergiram uma vez (A1 vs D1); (d) reabrir o epico antigo - misturava adiados com implementados (A9-A11) e itens sem valor (A4); (e) gates so por teste unitario - a Fase 2 provou o valor do artefacto arquivado. Custo assumido: sete documentos escritos antes do sintoma (curtos, verificados no codigo e datados); o gatilho impede a execucao prematura.