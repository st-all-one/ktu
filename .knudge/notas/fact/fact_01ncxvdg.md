---
id: fact_01ncxvdg
type: fact
statement: "H11: dossie de pendencias verificado no codigo, com estados, eixos e criterio de reabertura"
created_at: 2026-10-18T03:56:51.786Z
body_hash: fe3c090b
schema_version: 2
anchors: [wiki/pendencias.md, STAGING.md]
---
H11 (task_01o2ilbx). wiki/pendencias.md e o espelho do STAGING.md: estados (Deferido/Rejeitado/Opt-in/Lacuna/Formalismo) x eixos (Q qualidade/P performance/S simplificacao), com ancora no codigo e o teste/sintoma que reabre cada item (regra de poda M1). Lacunas verificadas no codigo hoje: L1 o workspace do knudge (crates/knudge) esta excluido do workspace ktu e a CI so corre make check na raiz - nao e verificado por ninguem; L2 ktu serve nao tem autenticacao (default 127.0.0.1:7717, mas --bind 0.0.0.0 expoe sem credenciais); L3 nenhum gate verifica links da wiki nem links intra-doc do Rust (cargo doc fora do make check) - foi assim que dois links quebraram (PM-0003); L4 o limite de 400 linhas nao cobre tests/ (isentos). Deferidos D1-D11 (A1-A7 do ADIADOS + sub_recipes, parameters declarados, model/hidden/color nos agentes, sandbox de SO), rejeitados R1-R7 COM o numero ou com unpriced declarado (micro-optimizacao: projecao de 10k eventos ~0,5 ms vs limiar de 5 ms; GDK: ~167 crates extra sobre 267), opt-in O1-O6 e formalismo F1-F3 (cargo doc fora do gate, envelope JSON sem golden unico, via de integracao do knudge por decidir).