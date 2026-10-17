# Configuração de rede (proxy + CA)

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/docs/content/network.mdx`;
  `wiki/revisao-opencode-profunda.md` §1.12.
- **Âncora:** `crates/ktu-provider/src/llama.rs`.

## O que é
`HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY` (loopback **excluído** por ser usado entre CLI e serviço) +
certificados personalizados.

## Porque interessa
O ktu fala HTTP com o provider; em redes corporativas, proxy e CA personalizada são necessários
para funcionar. Baixo esforço; documentar a **exclusão do loopback**.

## O que o ktu tem hoje
Cliente HTTP directo.

## Pré-condições para reconsiderar
- Rede corporativa (proxy/CA).

## Custo e risco
- **Esforço:** baixo (respeitar as variáveis + CA).
- **Risco de não fazer:** não funciona atrás de proxy.
- **Risco de fazer:** proxy mal configurado bloqueia — documentar `NO_PROXY` (loopback).

## Decisão pendente
Avaliar. Respeitar `HTTP(S)_PROXY`/`NO_PROXY` e CA personalizada; documentar a exclusão do
loopback; erro acionável quando o proxy falha.
