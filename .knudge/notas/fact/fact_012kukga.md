---
id: fact_012kukga
type: fact
statement: H12: gates G10-G15 verdes com runner proprio (xtask gates) e artefactos em wiki/gates
created_at: 2026-10-18T11:33:38.970Z
body_hash: 8c53299c
schema_version: 2
revision: 2
anchors: [crates/ktu-cli/src/provider_script.rs]
---
H12 (task_00h7sxsu). Novo `xtask gates` (make gates, na CI como passo proprio): prepara um espaco de trabalho isolado por gate em .ktu/gates/<gate>/, corre o BINARIO ktu de ponta a ponta, verifica os criterios de GATES.md e arquiva artefactos em wiki/gates/artifacts/<gate>/ + summary.json (contrato de maquina). Quadro final: G10 cadeia longa 7/7 (15 turnos numa sessao com o mesmo lote, todos end_turn, sem compactacao, artefacto correto - cargo test verde no alvo, soma corrigida e modulo media ligado com testes), G11 multi-modelo 2/2 + observacao (tool call declarada em TEXTO decodificada contra o catalogo fechado), G12 tres caminhos 4/4 (run, REPL e crate com log BYTE-IDENTICO - fnv64 igual nos tres - e o mesmo caminho duas vezes igual, I6), G13 concorrencia 6/6 (batch paralelo ordenado, sessoes isoladas, I1/I2 limpos, lock vivo recusa e lock largado corre), G14 project-bound 12/12 (init idempotente, marcadores unicos, --git-excluded/--git-tracked reversiveis, precedencia local>global/flag>config/params>config, M3 auth nao ecoa, M5 guardrail global nao desligavel), G15 memoria 9/9 (escrita automatica no fim do turno com proveniencia agent=ktu, recall em sessao nova, apagar .idx nao perde nada, projecao read-only, testes do knudge ask --agent e fonte JSONL). 40 verificacoes no total. Determinismo: os gates usam mock/scripted (sem inferencia) e duas corridas completas dao o mesmo quadro. Artefactos versionados em wiki/gates/ (README + 6 relatorios por gate + fixtures).