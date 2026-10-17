# Formatter (auto-formatação após edição)

- **Estado:** candidata (do `opencode`) — **incorporar**.
- **Origem:** `REF/opencode/packages/core/src/formatter.ts`; `wiki/revisao-opencode.md` §2.4.
- **Âncora:** `crates/ktu-tools/src/fs_tools.rs`, `crates/ktu-tools/src/tool_thread.rs`.

## O que é
Após editar um ficheiro (`write`/`edit`), correr o **formatter do projecto** (builtins por
linguagem/extensão), mantendo o código formatado sem o modelo gastar passos.

## Porque interessa
Usabilidade objectiva e barata: o resultado sai formatado; o modelo não gasta turnos a
corrigir formatação. É **efeito**, logo atrás de uma porta/tool.

## O que o ktu tem hoje
`write`/`edit` escrevem o conteúdo tal como o modelo o dá; sem formatter.

## Pré-condições para reconsiderar
- Projectos com formatter definido (rustfmt, prettier, …).
- Configuração em `.agents/` (quais comandos por extensão).

## Custo e risco
- **Esforço:** baixo (correr o comando após a edição + timeout + caps).
- **Risco de não fazer:** formatação a cargo do modelo (desperdício).
- **Risco de fazer:** correr comandos do projecto — passar pelo `Guard`/confinamento e
  ter timeout; nunca bloquear o turno.

## Decisão pendente
Promover; opt-in por configuração, com timeout e sem efeito no log de eventos (a não ser
o resultado da tool).
