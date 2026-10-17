# Temas (TUI)

- **Estado:** candidata (do `pi`) — **avaliar** (baixo valor).
- **Origem:** `REF/pi/packages/coding-agent/docs/themes.md`; `wiki/revisao-pi.md` §2.8.
- **Âncora:** `crates/ktu-cli/src/tui.rs`.

## O que é
**Temas** (JSON) que definem as cores do terminal, carregáveis de ficheiros.

## Porque interessa
Pouco: a TUI do ktu é mínima. É conveniência visual, não kernel nem métrica. Baixo esforço,
baixo valor.

## O que o ktu tem hoje
TUI mínima (ratatui) como cliente da porta; sem temas.

## Pré-condições para reconsiderar
- Uso interactivo intenso e preferência visual.

## Custo e risco
- **Esforço:** baixo.
- **Risco de não fazer:** nenhum.
- **Risco de fazer (YAGNI):** superfície de configuração cosmética.

## Decisão pendente
Adiar; provavelmente com a TUI completa (backlog 17).
