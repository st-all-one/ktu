# Cache de prefixo declarativo + cabeçalhos de afinidade

- **Estado:** candidata (do `katu`, ADR 0013) — **avaliar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0013-cache-de-prefixo-e-compressao-de-pedido.md`;
  `wiki/revisao-katu.md` §3.10.
- **Âncora:** `crates/ktu-provider/src/llama.rs`, `crates/ktu-core/src/session/mod.rs`.

## O que é
`prompt_cache`/`prompt_cache_retention` são **campos declarativos por modelo**; o `prompt_cache_key`
deriva da sessão; `affinity_headers` (`x-client-request-id`, `x-session-affinity`) são declarativos.
Medido no katu: `cached=896/1004` no 2.º turno.

## Porque interessa
O ktu já tem a condição necessária — a projeção é **prefix-stable** byte-a-byte. Falta declarar, por
modelo, que o endpoint tem cache e enviar os cabeçalhos de afinidade. É latência ganha de graça.

## O que o ktu tem hoje
Projeção prefix-stable (teste) + cabeçalhos de sessão no provider; sem campos declarativos por modelo.

## Pré-condições para reconsiderar
- Um endpoint/modelo que declare `cache_read` (ex.: `deepseek-v4.1-flash`).

## Custo e risco
- **Esforço:** baixo (campos no catálogo + cabeçalhos).
- **Risco de não fazer:** paga-se input a mais.
- **Risco de fazer:** enviar campos que o endpoint rejeita — só declarativo, por modelo.

## Decisão pendente
Avaliar. Só os modelos que o declaram recebem `prompt_cache`; afinidade alinhada com o gateway;
artefacto com `cached`/`input` (base `provider_reported`).
