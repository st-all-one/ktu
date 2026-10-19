---
id: decision_019o5bcj
type: decision
statement: "Steering: admissao durvel na fronteira do loop (entrega no passo seguinte), nao persistencia no steer() da borda"
created_at: 2026-10-18T14:00:43.736Z
body_hash: 6868e6de
schema_version: 2
anchors: [crates/ktu-core/src/agent/steering.rs, wiki/gates/G16-steering.md]
---
F1/M7. Decisao: a admissao do steering fica no log como Event::SteerQueued e a entrega (Event::Steer) acontece na fronteira SEGUINTE; o id vem do front, pelo que repetir o pedido nao duplica. Alternativas descartadas: (a) persistir a admissao no steer() da borda - exige dois escritores do mesmo log de sessao (a borda e o actor), com risco de lost update quando o actor grava no fim do turno; (b) guardar a fila pendente num ficheiro proprio - segundo store, contra I3 (o log e a fonte da verdade); (c) manter a entrega no mesmo passo e nao persistir nada - a mensagem admitida nao sobrevive a um crash nem ao fim do turno, e o item perde o sentido; (d) gerar o id no kernel (Rng/tempo) - quebra I6 e torna o log nao reproduzivel. Custo aceite: a mensagem entra no passo seguinte ao da admissao (antes entrava no mesmo), porque so assim existe um estado pendente que pode ser gravado e recuperado. Limite conhecido: a janela coberta e admissao registada -> morte antes da entrega; fechar a janela anterior exigiria (a).