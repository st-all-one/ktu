# Voz / ditado

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose/src/{live_voice,dictation}`; `wiki/revisao-goose.md` §3.
- **Âncora:** — (sem equivalente no ktu).

## O que é
Entrada por **voz** e ditado (STT) na interface interactiva.

## Porque não entra
Fora do escopo (agente de código por texto). É uma capacidade de interface sem relação com
o kernel nem com a métrica.

## Pré-condições para reconsiderar
- Uso por voz como alvo explícito.

## Custo e risco
- **Esforço:** médio-alto (STT + integração de UI).
- **Risco de não fazer:** nenhum para o escopo.
- **Risco de fazer (YAGNI):** superfície de UI áudio.

## Decisão pendente
Manter fora.
