# `/reload` (recarregar recursos sem reiniciar)

- **Estado:** candidata (do `pi`) — **avaliar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/configuration.md` (`/reload`);
  `wiki/revisao-pi-profunda.md` §1.10.
- **Âncora:** `crates/ktu-cli/src/plugins_cmd.rs`, `crates/ktu-core/src/guardrails.rs`.

## O que é
`/reload` recarrega settings, keybindings, instruções, extensões, skills e templates **sem
reiniciar** a sessão. A extensão recarregada **não** pode reusar estado do runtime antigo.

## Porque interessa
O ktu tem hot reload de **plugins** por geração; estender o conceito a recursos
(skills/guardrails/prompt) é usabilidade objectiva — editar um guardrail ou skill sem matar a
sessão.

## O que o ktu tem hoje
Hot reload de plugins por geração; recursos carregados no arranque.

## Pré-condições para reconsiderar
- Edição de recursos durante uma sessão.

## Custo e risco
- **Esforço:** baixo-médio (recarregar loaders + invalidar cache).
- **Risco de não fazer:** reiniciar para aplicar.
- **Risco de fazer:** estado inconsistente a meio do turno — recarregar só em repouso; geração
  (como os plugins).

## Decisão pendente
Avaliar. Recarregar recursos em repouso, por geração (como os plugins); nunca a meio de um turno;
teste de reload.
