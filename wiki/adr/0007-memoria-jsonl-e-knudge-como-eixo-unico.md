# 0007 — Memória JSONL e o knudge como eixo único

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-memory/src/`, `crates/ktu-core/src/memory.rs`,
  `wiki/knudge-eixo-memoria.md`

## Contexto

Um agente que trabalha em várias sessões precisa de memória que sobreviva a elas, mas há dois papéis
a não confundir: a memória do **projecto** (o que a equipa sabe) e o **motor** de memória (o que o
agente grava e consulta). O knudge (`.knudge/`, `kd`) já é a memória de desenvolvimento deste
projecto, com contrato próprio (registo é a verdade, índice derivado, *dedup* por limiar).

## Decisão

A memória do agente é **JSONL append-only** em `.agents/memory/notes.jsonl`, com o contrato do
`knudge-core` adoptado como motor **puro em memória** (`ktu-memory`), sem projectar a porta `Fs`:
o mesmo schema físico, o mesmo *dedup* (cria < 0,75 / *merge* 0,75–0,92 / rejeita ≥ 0,92) e a mesma
busca determinística `(score desc, id asc)`. O índice (`.idx/`) é **derivado e descartável** (I17).
O knudge é o **eixo único** de memória do ktu: não se adoptam `kd` nem `knudge-mcp` como
dependências do binário.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| SQLite/FTS para as notas | acrescenta dependência e transacções; o registo já é *append-only* e inspeccionável |
| Adoptar `kd` como biblioteca | traz CLI/terminal/relógio/FS para dentro do domínio (viola [0001](0001-kernel-puro-com-portas.md)) |
| `knudge-mcp` (servidor MCP) | MCP está na lista YAGNI; o protocolo seria uma segunda fronteira sem consumidor |
| Memória só em memória (por sessão) | perde exactamente o que se quer: persistência entre sessões (Q3/Q4 da bateria de qualidade) |
| Dois motores (um para o projecto, outro para o agente) | duas verdades para reconciliar; o `kd` continua a ser a memória de desenvolvimento |

## Consequências

- Apagar `.idx/` e reconstruir não perde nada e dá o mesmo *recall* (teste de conformação).
- O canal semântico é **opcional**: `Embedder` é porta, o índice é derivado, e `ktu memory drain`
  funciona por passos (nunca bloqueia o turno).
- Custo: dois papéis para explicar (`kd` para desenvolver o ktu, `ktu-memory` para o agente) — está
  documentado em `wiki/knudge-eixo-memoria.md`.
