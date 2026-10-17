# knudge — grafo avançado (ontologia, TMS, PageRank/PPR, Louvain)

- **Estado:** candidata (do `knudge` 0.6.1) — **avaliar**.
- **Origem:** `crates/knudge/wiki/integration/07_grafo_e_inferencia.md`,
  `crates/knudge/wiki/specs/grafo.md`; `wiki/revisao-knudge.md` §6.2.
- **Âncora:** `crates/ktu-memory/src/store.rs` (usa só `Graph::from_notes_ref`).

## O que é
Sobre as arestas explícitas do frontmatter, o `graph` do knudge deriva:
- **ontologia leve** (`same_as`/`broader`/`narrower`/`related`, D207) — equivalência e clausura;
- **claims SPO** e **conflitos** precisos (`claim_conflicts`);
- **TMS/defeasible** (D208) — dependentes retratados por `forgotten`/`superseded`;
- **PageRank** global e **PPR** pessoal (D192) — autoridade no grafo;
- **Louvain** (D193) — comunidades (GraphRAG);
- integridade/ciclos (`integrity`, `supersession_cycles`, `dependency_cycles`).

## Porque interessa
O ktu usa hoje o grafo só para `recall`; a ontologia/TMS/PPR/comunidades dão inferência que o ktu
não tem (contradição precisa, demolição por retratação, autoridade). Liga a
[2 — auditoria-densa](./auditoria-densa.md) (consulta do corpus).

## O que o ktu tem hoje
`Graph::from_notes_ref` + `recall`; sem ontologia/TMS/PPR/Louvain.

## Pré-condições para reconsiderar
- Um consumidor que exija inferência (ex.: `ktu map --communities`, contradição no `doctor`).

## Custo e risco
- **Esforço:** médio (expor `graph::*` na porta, se necessário).
- **Risco de não fazer:** inferência ausente.
- **Risco de fazer:** alargar a superfície sem consumidor (§0 YAGNI).

## Decisão pendente
Adiar até haver consumidor. Se entrar, expor como **dado derivado** (nunca armazenado) atrás da
porta; PPR/Louvain são off-path e determinísticos.
