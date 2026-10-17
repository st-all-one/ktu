# Hooks (eventos de ciclo de vida)

- **Estado:** candidata (do `goose`) — **avaliar**.
- **Origem:** `REF/goose/documentation/docs/guides/context-engineering/hooks.md`;
  `wiki/revisao-goose-profunda.md` §1.9.
- **Âncora:** `crates/ktu-core/src/plugin.rs`, `crates/ktu-plugin/`.

## O que é
Scripts do utilizador em eventos (`SessionEnd`, edição de ficheiro, comando de shell),
descobertos de **plugins em disco** (`<plugin>/hooks/hooks.json`), com o payload em JSON no
stdin. Segue a *Open Plugins hooks specification*. Ex.: formatar após edição, notificar no fim.

## Porque interessa
O ktu tem plugins (que transformam/negam) mas **não** hooks de notificação/ciclo de vida. É
uma forma leve de extensibilidade — **dado** (evento → comando) + processo, como os plugins.

## O que o ktu tem hoje
Plugins externos (processo) com guardrails; sem hooks de ciclo de vida.

## Pré-condições para reconsiderar
- Um caso de uso (ex.: formatter após edição, notificação).

## Custo e risco
- **Esforço:** médio (eventos + descoberta + execução com timeout).
- **Risco de não fazer:** integrações locais exigem plugins completos.
- **Risco de fazer:** **executa comandos locais** — só de fontes confiáveis; timeout e caps;
  nunca bloquear o turno.

## Decisão pendente
Avaliar. Se entrar, eventos fechados (poucos), payload JSON, timeout, e opt-in explícito;
consolidar com a página [formatter](./formatter.md) (auto-formatação) que é o caso de uso óbvio.
