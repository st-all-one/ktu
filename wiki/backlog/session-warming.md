# Session warming

- **Estado:** candidata (do `opencode`) — **avaliar** (consolidar com [73](./cache-prefixo.md)).
- **Origem:** `REF/opencode/packages/core/src/plugin/warming.ts`,
  `docs/content/warming.mdx`; `wiki/revisao-opencode-profunda.md` §1.6.
- **Âncora:** `crates/ktu-provider/`.

## O que é
Envia pedidos *keep-alive* periódicos para preservar as **caches de prefixo** do provider durante
pausas; **desligado por omissão**; configurável (`prompt`/`interval`/`duration`). Aquece uma
sessão após 4 min sem pedido, a cada 4 min, até 30 min desde o último pedido real. O pedido é
uma mensagem transitória ("faz nada, responde OK").

## Porque interessa
Complementa a cache de prefixo declarativa ([73](./cache-prefixo.md)) e o cache warming do pi:
não só declarar, mas **manter quente**. Avançado (custo/TTL) — só se o modelo declarar cache.

## O que o ktu tem hoje
Projeção prefix-stable; sem warming.

## Pré-condições para reconsiderar
- Um modelo que declare `cache_read` e pausas longas entre turnos.

## Custo e risco
- **Esforço:** baixo-médio (timer + pedido transitório).
- **Risco de não fazer:** cache expira; paga-se input.
- **Risco de fazer:** gastar tokens sem retorno — só quando a poupança esperada compensa; off por
  omissão; nunca entra no contexto.

## Decisão pendente
Avaliar. Off por omissão; só com cache declarada; pedido transitório **fora** do contexto e do
log model-visible; consolidar em [73](./cache-prefixo.md).
