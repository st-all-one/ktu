# Precedência de credenciais

- **Estado:** candidata (do `pi`) — **incorporar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/models.md` (Authenticate);
  `wiki/revisao-pi-profunda.md` §1.8.
- **Âncora:** `crates/ktu-cli/src/providers.rs`, `crates/ktu-provider/`.

## O que é
Ordem explícita de resolução de credenciais: `--api-key` (runtime) **>** credencial guardada
(`auth.json`) **>** `apiKey` da config (`models.json`) **>** variáveis de ambiente / credenciais
de cloud ambiente. Extensões podem definir o seu próprio comportamento.

## Porque interessa
O ktu lê a chave do ambiente. Uma **ordem** explícita (runtime > guardada > config > env) evita
ambiguidade, permite CI sem escrever credenciais e dá precedência previsível. É **regra**, não
subsistema.

## O que o ktu tem hoje
Chave via variável de ambiente (`OPENCODE_GO_API_KEY`); sem ordem explícita.

## Pré-condições para reconsiderar
- Múltiplas fontes de credencial (runtime + guardada + env).

## Custo e risco
- **Esforço:** baixo (ordem documentada + resolução).
- **Risco de não fazer:** ambiguidade; credencial errada.
- **Risco de fazer:** guardar credenciais em disco — só opt-in e fora do log.

## Decisão pendente
Promover. Ordem explícita e documentada; **nunca** logar credenciais; armazenamento em disco só
opt-in. Consolidar com [31 — oauth-subscricoes](./oauth-subscricoes.md).
