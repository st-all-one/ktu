# Resposta grande → ficheiro

- **Estado:** candidata (do `goose`) — **incorporar**.
- **Origem:** `REF/goose/crates/goose/src/agents/large_response_handler.rs`;
  `wiki/revisao-goose-profunda.md` §1.3.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`, `crates/ktu-tools/src/toolbox.rs`.

## O que é
Se o texto de um resultado de tool excede `GOOSE_MAX_TOOL_RESPONSE_SIZE` (default **200 000**
chars), é escrito num **ficheiro temporário** e o modelo recebe uma **referência** ("a
resposta era maior e está no ficheiro …, usa outras tools para o examinar"). Protege a janela
de contexto de um resultado gigante.

## Porque interessa
O ktu tem caps de `bash`/ficheiro, mas uma tool pode devolver algo enorme (ex.: `grep` num
repo grande). Guardar em ficheiro + referência é **simples** e evita estourar o orçamento —
melhor que truncar (o modelo pode ir buscar o resto).

## O que o ktu tem hoje
Caps de leitura/execução; sem reencaminhamento de resultados grandes para ficheiro.

## Pré-condições para reconsiderar
- Um resultado de tool que exceda o orçamento no fluxo real.

## Custo e risco
- **Esforço:** baixo (wrapper de resultado + porta `Fs` + limiar).
- **Risco de não fazer:** orçamento estoura; truncagem perde dados.
- **Risco de fazer:** ficheiros temporários acumulam — limpeza e caminho previsível.

## Decisão pendente
Promover. Limiar configurável; escrever em ficheiro + referência legível (com caminho); limpeza
documentada.
