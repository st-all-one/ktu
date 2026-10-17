# Inferência local (candle)

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose-local-inference` (10 k linhas); `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-provider/src/`, `scripts/setup-model.sh`.

## O que é
Correr o modelo **dentro do processo** com `candle` (HuggingFace), sem `llama-server`
externo; o goose tem uma feature `local-inference` com suporte HF Hub.

## Porque não entra
O ktu decidiu **`llama.cpp` built-in** (`llama-server`, SSE) — um processo externo simples,
que separa o transporte do kernel e reaproveita o melhor runtime de inferência local.
Trazer `candle` duplicaria o caminho de inferência.

## Pré-condições para reconsiderar
- Necessidade de **embedding** de inferência (sem processo externo) — ex.: distribuição
  single-binary.
- Modelo-alvo que o `llama.cpp` não sirva.

## Custo e risco
- **Esforço:** alto (runtime de inferência + kernels + quantização).
- **Risco de não fazer:** baixo (o llama-server funciona).
- **Risco de fazer (YAGNI):** reimplementar o que o `llama.cpp` já faz melhor.

## Decisão pendente
Manter fora. O `llama.cpp` é a fronteira de inferência do ktu.
