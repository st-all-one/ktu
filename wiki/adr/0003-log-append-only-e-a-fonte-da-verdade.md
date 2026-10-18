# 0003 — O log append-only é a fonte da verdade (I3)

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/session/`, `crates/ktu-core/src/project.rs`

## Contexto

O katu perdia contexto a cada turno: o estado vivo estava em memória e o pedido ao modelo era
montado a partir de buffers que ninguém conseguia reconstruir. Uma sessão não retomável não é
depurável — e o utilizador não tem como saber o que o modelo realmente viu.

## Decisão

O **log de eventos** (`SessionLog`, JSONL append-only em `.ktu/sessions/<id>.jsonl`) é a fonte da
verdade. O pedido ao modelo é **derivado** dele (`derive_messages`), e a invariante I3 —
*model-visible ⟺ logged* — vale para tudo o que entra no contexto (incl. *steering*, instruções
locais e resultados de tools, dentro do envelope *taint*). Escrita atómica antes do evento.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Estado vivo em memória + log como *debug* | foi o erro do katu: contexto perdido e sessão não retomável |
| Histórico do provider como fonte | prende o projecto a um dialeto e perde o que não foi enviado (ex.: tools negadas) |
| Base de dados (SQLite) para os eventos | acrescenta dependência e transacções para um formato que já é *append-only* e inspeccionável |
| Guardar as mensagens já renderizadas | duplica a verdade: qualquer mudança de projecção invalidaria o histórico |

## Consequências

- Retomar, compactar e projectar são operações **puras** sobre o log; a compactação guarda resumo
  **e** cru, com âncora verificável.
- O log é a auditoria: uma tool negada, uma instrução local, um `tool.loaded` — tudo lá está.
- Custo: o log cresce e há que o projectar por passo; a projecção é medida (`derive_messages_*` em
  `wiki/benchmarks/`) e custa microssegundos ([0014](0014-medir-antes-de-optimizar.md)).
