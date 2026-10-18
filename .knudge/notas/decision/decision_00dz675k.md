---
id: decision_00dz675k
type: decision
statement: "H12: os gates da Fase 2 correm com guiao deterministico (provider scripted), com o modelo real como observacao"
created_at: 2026-10-18T11:33:40.286Z
body_hash: 1ae1a29b
schema_version: 2
revision: 2
anchors: [wiki/adr/0018-gates-da-fase-2-por-guiao-deterministico.md, crates/ktu-cli/src/provider_script.rs]
---
Os gates G10-G15 exigem prova (artefactos) e repetibilidade ("o mesmo gate, duas vezes => o mesmo resultado observavel"). O modelo de referencia (Ornith-1.5-9B Q4_K_M) mediu ~8 min por completion neste host CPU-only: uma cadeia de 15 turnos e inviavel e uma corrida com modelo nao e repetivel. Decisao: os gates correm com providers deterministas (mock para os caminhos triviais; scripted - guiao JSONL, cursor global ao processo - para as cadeias longas), correndo o BINARIO de ponta a ponta; o modelo real fica como OBSERVACAO opcional (--real-model), registada no summary.json e nos artefactos, nunca como criterio (numa das corridas o 1.5B leu o ficheiro mas nao fechou o turno no tecto de passos - comportamento tolerado, mas nao um veredicto). Alternativas descartadas: (a) correr tudo com o modelo de referencia - nao cabe no orcamento nem no determinismo; (b) so o provider mock (eco) - nao executa uma tarefa, o artefacto do G10 nao seria verificavel; (c) fake provider apenas nos testes do nucleo - provaria a biblioteca, nao o produto; (d) gravar uma sessao real e fazer replay (--record) - mais superficie para o que o guiao revisto como codigo ja da (fica no dossier); (e) um guiao por turno - obrigaria a plumbar um indice de turno. Contrapartida assumida: o guiao nao mede QUALIDADE de modelo - essa barra continua a ser a bateria p1-p4 (G4, com modelo real).