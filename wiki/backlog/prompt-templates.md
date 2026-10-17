# Prompt templates (Markdown → `/` comando)

- **Estado:** candidata (do `pi`) — **incorporar** (barato).
- **Origem:** `REF/pi/packages/coding-agent/docs/prompt-templates.md`;
  `wiki/revisao-pi.md` §2.2.
- **Âncora:** `.agents/`, `crates/ktu-cli/src/cli.rs`.

## O que é
Ficheiros **Markdown** em `prompts/` tornam-se comandos `/nome` reutilizáveis, com
`description`, `argument-hint` e substituição de argumentos (`$1`, `$@`,
`${1:-default}`). Sem código de execução — só texto expandido antes de virar mensagem.

## Porque interessa
- **Simplicidade máxima:** um `.md` + frontmatter; zero execução.
- **Usabilidade objectiva:** repetir prompts sem os redigitar.
- **Kernel:** é dado; loader fail-open (como as skills).

## Relação com as receitas (backlog 19)
Sobrepõe-se às **receitas** do goose (que têm parâmetros + settings + sub-receitas).
Decidir: **um** mecanismo só (templates, leves) ou **dois** (templates leves + receitas
com settings). Sugestão: começar por **templates** (o mais simples) e ver se as receitas
se tornam necessárias.

## O que o ktu tem hoje
`.agents/skills/` (instruções de apoio, carregadas on-demand) e o prompt composto
(`AGENTS.md`). Não há templates de prompt.

## Pré-condições para reconsiderar
- Um prompt que se repita.

## Custo e risco
- **Esforço:** baixo (loader + expansão de argumentos + `/nome`).
- **Risco de não fazer:** redigitar prompts.
- **Risco de fazer (YAGNI):** mais um formato declarativo (junto com receitas/guardrails).

## Decisão pendente
Promover já; resolver a sobreposição com receitas antes de implementar as duas.
