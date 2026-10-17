# Mover sessão

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/packages/core/src/session/move.ts`;
  `wiki/revisao-opencode-profunda.md` §1.10.
- **Âncora:** `crates/ktu-cli/src/session_store.rs`, `crates/ktu-core/src/layout.rs`.

## O que é
Move uma sessão para outro directório/location, re-resolvendo os serviços de *placement*; os
eventos da sessão ficam **fora** do contexto da location para que o movimento não os fixe à
antiga.

## Porque interessa
O ktu associa uma sessão a um `--root`. Mover uma sessão entre projectos é usabilidade avançada
— útil quando o trabalho começa num directório e continua noutro.

## O que o ktu tem hoje
Sessão ligada a um root fixo.

## Pré-condições para reconsiderar
- Trabalho que atravessa directórios.

## Custo e risco
- **Esforço:** médio (re-resolver placement + não fixar eventos).
- **Risco de não fazer:** sessão presa ao root.
- **Risco de fazer:** caminhos relativos partidos — o log guarda caminhos; mover pode invalidar
  referências.

## Decisão pendente
Adiar. Se entrar, mover re-resolve o placement e **não** fixa os eventos à location antiga;
verificar que os caminhos do log continuam válidos.
