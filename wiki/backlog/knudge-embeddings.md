# knudge — porta `Embedder` (canal semântico)

- **Estado:** candidata (do `knudge` 0.6.1) — **avaliar**.
- **Origem:** `crates/knudge/wiki/integration/10_embeddings.md`,
  `crates/knudge/wiki/specs/embeddings.md`; `wiki/revisao-knudge.md` §6.2.
- **Âncora:** `crates/ktu-core/src/ports/`, `crates/ktu-memory/src/store.rs`.

## O que é
A porta `Embedder` (`meta()` + `embed(lote)`) com adaptadores prontos (`HttpEmbedder` para servidor
OpenAI-compatible, `LightweightEmbedder` determinístico para testes). O índice vetorial e o cache
(por `body_hash` + modelo) são **derivados**; a fila é lazy; `drain` embute em lotes; `rank_query`
produz os ids do canal `semantic` que entra na fusão **RRF**. Sem provedor, o canal desliga **sem
aviso** (degradação graciosa).

## Porque interessa
O ktu é lexical (BM25 + âncoras). Um canal semântico melhora recall de paráfrases — mas é
**opcional por design**. A porta (não o serviço) encaixa no padrão do ktu (portas + fakes). Liga a
[24 — contexto-por-modelo](./contexto-por-modelo.md) (janela) e à porta `Memory`.

## O que o ktu tem hoje
Sem embeddings; recall lexical.

## Pré-condições para reconsiderar
- Evidência de recall lexical insuficiente (paráfrases).

## Custo e risco
- **Esforço:** médio (porta `Embedder` + fakes + drain off-path).
- **Risco de não fazer:** recall lexical limitado.
- **Risco de fazer:** acoplar a um serviço/worker — adoptar **só a porta**, com fake determinístico;
  o worker/`drain service` fica de fora.

## Decisão pendente
Avaliar. Se entrar, **porta `Embedder` + `FakeEmbedder`** (determinístico, como as outras portas);
o provedor HTTP atrás de config; `drain` off-path; trocar de modelo invalida o índice.
