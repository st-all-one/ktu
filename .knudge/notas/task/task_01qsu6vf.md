---
id: task_01qsu6vf
type: task
statement: "Fiabilidade de edicao: fila de mutacao por caminho, remedio por regra, ancoras proximas"
created_at: 2026-10-18T00:31:58.552Z
body_hash: 16a86bdb
schema_version: 2
tags: [fase3]
revision: 5
outcomes:
  - status: success
    recorded_at: 2026-10-18T15:01:51.427Z
    notes: "Fila por caminho + remedio por regra + ancoras proximas. Novo crates/ktu-tools/src/mutation.rs: FileMutationQueue (BTreeMap<PathBuf, Arc<tokio::sync::Mutex<()>>>, chave = realpath com fallback, poda a partir de 256) - write/edit passam por ela e a vez e pedida ANTES da leitura no edit (ler fora/gravar dentro perdia a alteracao). fs_tools write/edit passaram a async. Remedio por regra: o Guard::remedy ja devolvia o remedio da regra (DeclarativeGuard + ChainGuard) - faltava a mensagem nao substituir o estrutural; delegation.rs junta os dois. Ancoras proximas: text.rs::nearby (Levenshtein limitado, 3 melhores em ordem de linha, com numero de linha; ambigua nao sugere). Testes: 2 unit da fila (multi_thread), 2 de integracao (escritas concorrentes + edicoes sem perda), 3 do texto, 2 CLI (editing.rs) + 1 de permissao. Gate G19 verde 7/7."
anchors: [wiki/implementation_phase3/F4-fiabilidade-edicao.md]
status: closed
scope: task
---
Adiados da H5. (a) Fila de mutacao por caminho: as tools correm sequencialmente numa unica thread T2 (WorkerCtx::serve serve um pedido de cada vez; subagentes partilham a T2) - nao ha escrita concorrente ao mesmo ficheiro (YAGNI). (b) Remedio por regra de guardrail: Decision::Deny(DenyReason) nao transporta dados; o remedio estrutural por DenyReason ja cobre todos os motivos. (c) edit a sugerir as ancoras unicas mais proximas: a mensagem actual ja devolve a ancora procurada. Reabrir quando houver paralelismo de tools / uma regra precisar de remedio proprio / sintoma de ancoras erradas.
<!-- knudge:parent task_00sb14zh -->