# Provider policy (declarações ordenadas)

- **Estado:** candidata (do `opencode`) — **avaliar** (consolidar com [1](./motor-de-politica.md)/[21](./permission-store.md)).
- **Origem:** `REF/opencode/specs/v2/provider-policy.md`,
  `docs/content/policies.mdx`; `wiki/revisao-opencode-profunda.md` §1.9.
- **Âncora:** `crates/ktu-core/src/guardrails.rs`, `crates/ktu-core/src/guard.rs`.

## O que é
Declarações `{effect: allow|deny, action: provider.use|tool.use|integration.use, resource}`;
wildcard; **última correspondência vence**; utilizador > repositório > organização. `tool.use`
**nunca** concede acesso (as permissões decidem `allow`/`ask`).

## Porque interessa
O ktu exclui um *policy engine* (§0), mas tem guardrails. A ideia "declaração ordenada + wildcard
+ última vence" é **dado** simples e expressivo — e separa **configuração** (endpoints) de
**política** (permissão de uso). Consolida com [1](./motor-de-politica.md) e
[21](./permission-store.md).

## O que o ktu tem hoje
Guardrails declarativos (por caminho/padrão); sem `provider.use`/`tool.use`.

## Pré-condições para reconsiderar
- Necessidade de negar o uso de um provider/tool por política.

## Custo e risco
- **Esforço:** baixo-médio (lista ordenada + wildcard + avaliação).
- **Risco de não fazer:** política grossa.
- **Risco de fazer:** ambiguidade de precedência — **última vence** documentado e testado; `deny`
  sempre vence o `allow` no mesmo recurso? (o katu prefere deny-by-default).

## Decisão pendente
Avaliar. Se entrar, `{effect, action, resource}` ordenado, wildcard, última vence; `tool.use`
**nunca** concede (só nega); teste de precedência.
