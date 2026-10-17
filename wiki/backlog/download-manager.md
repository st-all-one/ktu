# Download manager (modelos/binários)

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose-download-manager`, `src/download_manager.rs`;
  `wiki/revisao-goose.md` §3.
- **Âncora:** `scripts/setup-model.sh`.

## O que é
Gestor de downloads (retomáveis, verificados, com progresso) para modelos GGUF e binários.

## Porque não entra
O ktu tem `scripts/setup-model.sh` (descarrega o GGUF de referência). Um gestor completo
(retoma, mirrors, verificação) é conveniência de distribuição, não kernel.

## Pré-condições para reconsiderar
- Múltiplos modelos a gerir, com downloads grandes/frágeis.

## Custo e risco
- **Esforço:** baixo-médio.
- **Risco de não fazer:** baixo (o script serve).
- **Risco de fazer (YAGNI):** mais uma superfície de rede.

## Decisão pendente
Manter fora. Se entrar, como comando da borda (`ktu models pull`), nunca no kernel.
