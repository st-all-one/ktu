# Keybindings configuráveis

- **Estado:** candidata (do `pi`) — **incorporar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/keybindings.md`,
  `core/{keybindings,keybindings-schema}.ts`; `wiki/revisao-pi-profunda.md` §1.6.
- **Âncora:** `crates/ktu-cli/src/tui.rs`.

## O que é
Acções **nomeadas** (`app.session.new`) mapeadas para teclas num `keybindings.json` (uma ou lista;
lista vazia desliga); `/hotkeys` mostra os activos; `/reload` aplica. Sintaxe `modifier+key`
(`ctrl`/`shift`/`alt`/`super`).

## Porque interessa
A TUI do ktu tem teclas fixas. Acções nomeadas + config é usabilidade objectiva e **dado**
(mesmo padrão dos guardrails declarativos). Baixo esforço.

## O que o ktu tem hoje
Teclas fixas na TUI.

## Pré-condições para reconsiderar
- Preferências de teclas divergentes entre utilizadores.

## Custo e risco
- **Esforço:** baixo (registo de acções + mapa configurável).
- **Risco de não fazer:** teclas rígidas.
- **Risco de fazer:** conflito de teclas — validação + lista vazia desliga.

## Decisão pendente
Promover. Acções nomeadas, mapa em ficheiro, validação de conflitos, `/hotkeys` mostra os activos.
