---
id: decision_01b0xy52
type: decision
statement: "Fila de mutacao do processo (Arc no hub), nao por turno; lock entre processos e escrita atomica deferidos"
created_at: 2026-10-18T15:14:48.018Z
body_hash: ef2fb816
schema_version: 2
anchors: [crates/ktu-tools/src/mutation.rs, crates/ktu-cli/src/hub.rs, wiki/pendencias.md]
---
Decisao (pos-F4). A fila de mutacao e um recurso DO PROCESSO: um Arc<FileMutationQueue> criado no hub (por servico de sessoes) e um por lote, injectado em execute_turn. Motivo: a concorrencia dentro de um processo vem de varias T2 (uma por turno) - sem partilha, a fila dava uma garantia que nao se aplicava ao caso mais provavel (duas sessoes do serve, workers do batch paralelo). Alternativas descartadas: (a) fila estatica/global (OnceLock) - estado escondido, contra a DI do projecto, e tornaria os testes dependentes uns dos outros (tracked() partilhado); (b) manter uma fila por turno e aceitar o defeito - duas sessoes a editar o mesmo ficheiro perdiam alteracoes hoje; (c) lock de ficheiro em disco ja agora (flock) - resolve o caso entre processos mas nao o caso em processo, e nao tem consumidor observado. Deferidos com gatilho: D13 lock de ficheiro entre processos (gatilho: dois processos no mesmo projecto), D14 escrita atomica temp+rename (gatilho: ficheiro truncado por queda a meio - o log de sessao e reescrito inteiro a cada turno).