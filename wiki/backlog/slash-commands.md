# Slash commands (atalhos interactivos)

- **Estado:** **incorporada** (H10) — `/help`, `/plan`, `/build`, `/mode`, `/cancel`, `/quit`
  (embutidos) + `/nome args` resolvido contra `.agents/commands/*.md` (comandos do projecto).
- **Origem:** `REF/goose/crates/goose/src/slash_commands`; `wiki/revisao-goose.md` §2.2.
- **Âncora:** `crates/ktu-cli/src/tui.rs`, `crates/ktu-port/src/protocol.rs`.

## O que é
Comandos `/...` na sessão interactiva, resolvidos por nome, com origem
(`Builtin`/`Recipe`/`Skill`): ex.: `/compact`, `/mode`, `/recipe`, `/skill`, `/cancel`.

## Porque interessa
O ktu **já tem** os comandos na porta (`SetMode`, `cancel`, `continue`); os atalhos são
apenas **fronteira** (front-end), com superfície pequena e fechada. Usabilidade objectiva
sem tocar no kernel.

## O que o ktu tem hoje
CLI com subcomandos e `ktu mode`; a TUI é mínima e não tem atalhos de linha.

## Pré-condições para reconsiderar
- Uso interactivo (TUI) como alvo; lista de atalhos estável.

## Custo e risco
- **Esforço:** baixo (parser de linha + registo).
- **Risco de não fazer:** baixo.
- **Risco de fazer (YAGNI):** duplicar comandos que já existem.

## Decisão pendente
Promover junto com a TUI/interacção. Mapear cada atalho a **um** comando da porta
(sem lógica própria).
