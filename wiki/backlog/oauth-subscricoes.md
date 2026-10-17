# OAuth / subscrições (Claude/ChatGPT/Gemini)

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose/src/oauth`, `documentation/docs/guides/acp-providers.md`;
  `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-cli/src/cli.rs` (`--api-key`), `crates/ktu-provider/`.

## O que é
Autenticação **OAuth** para usar subscrições existentes (Claude/ChatGPT/Gemini) em vez de
API keys — fluxo interactivo de login, tokens, refresh.

## Porque não entra
O ktu usa **API keys** por variável de ambiente (`OPENCODE_GO_API_KEY`). OAuth é um
subsistema (fluxo, armazenamento seguro de tokens, refresh) sem consumidor.

## Pré-condições para reconsiderar
- Necessidade de usar uma **subscrição** (não uma API key) como provider.
- Armazenamento seguro de tokens definido.

## Custo e risco
- **Esforço:** médio-alto (OAuth por provider + segurança de tokens).
- **Risco de não fazer:** baixo (API keys cobrem).
- **Risco de fazer:** superfície de segurança de credenciais.

## Decisão pendente
Manter fora. Preferir API keys; reabrir só com requisito de subscrição.
