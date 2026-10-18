---
id: fact_010tjxi4
type: fact
statement: "Titulo automatico de sessao: heuristico determinístico no checkpoint, mostrado por ktu session ls"
created_at: 2026-10-18T02:59:13.296Z
body_hash: 66a4763c
schema_version: 2
anchors: [crates/ktu-core/src/session/title.rs, crates/ktu-core/src/checkpoint.rs, crates/ktu-cli/src/session_store.rs, wiki/convencoes.md]
---
H10: novo ktu-core/src/session/title.rs - title_from(prompt) tira tags XML, cercas de codigo e pontuacao de extremidade, colapsa espacos e corta em <= 8 palavras (MAX_WORDS) e <= 60 caracteres (MAX_CHARS) na fronteira de palavra; title_of(log) usa o PRIMEIRO TurnStart (deterministico: retomar nao muda o titulo). O titulo e guardado no CHECKPOINT (campo title, #[serde(default)] para retro-compatibilidade), que e um indice DERIVADO - o log continua a ser a unica verdade (I3) e o titulo e recalculavel a qualquer momento. ktu session ls passou a mostrar o titulo como ultima coluna (id, turns, mode, bytes, title) e --json ganhou "title". Sem pedido utilizavel => None e o consumidor usa o id. Testes: 5 unit + ktu-cli/tests/session.rs::ls_shows_the_title_derived_from_the_first_prompt.