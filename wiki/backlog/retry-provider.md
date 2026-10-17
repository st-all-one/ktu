# Retry endurecido do provider

- **Estado:** candidata (do `katu`, ADR 0011/0012) — **incorporar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0011-porta-provider-e-builtin-opencode.md` §7;
  `wiki/revisao-katu.md` §3.9.
- **Âncora:** `crates/ktu-provider/src/llama.rs`.

## O que é
Retry **só antes do primeiro delta** (depois duplicaria texto); `Retry-After` em segundos, ms **ou
data `HTTP`** (descontando o `Date` — o provider não toca relógio); limites de conta/quota são
**permanentes** (não repetir); **sanitização de URLs** (credenciais/query fora do log).

## Porque interessa
O ktu tem retry com backoff (`is_retryable`), mas: (a) sem a distinção "só antes do 1º delta" pode
duplicar output; (b) sem `Retry-After` em data ignora o pedido do servidor; (c) sem classificar
quota como permanente repete inutilmente; (d) uma URL com credenciais pode ir para o log.

## O que o ktu tem hoje
`llama.rs`: `retry_backoff`, `is_retryable`. Sem `Retry-After`, sem permanente-vs-transitório
explícito, sem sanitização de URL.

## Pré-condições para reconsiderar
- Falhas transitórias observadas (429/5xx).

## Custo e risco
- **Esforço:** baixo-médio.
- **Risco de não fazer:** output duplicado; esperas erradas; credenciais no log.
- **Risco de fazer:** retry a mais — só antes do 1º delta e com tecto de tentativas.

## Decisão pendente
Promover. Retry só antes do 1º delta; `Retry-After` (s/ms/data); quota = permanente; URL sanitizada;
testes sem rede (`MockTransport`).
