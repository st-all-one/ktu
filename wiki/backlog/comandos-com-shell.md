# Comandos com shell embutido

- **Estado:** candidata (do `opencode`) — **incorporar** (unificar com templates/receitas).
- **Origem:** `REF/opencode/.opencode/command/*.md`; `wiki/revisao-opencode.md` §2.3.
- **Âncora:** `crates/ktu-cli/src/cli.rs`, `crates/ktu-tools/src/bash.rs`.

## O que é
Comandos `/nome` (frontmatter: `description`, `model`, `subtask`) cujo corpo pode
**expandir shell** (`` !`git diff` ``) — a saída do comando entra no prompt.

## Porque interessa
É o **prompt template** (pi, backlog 41) **mais** contexto dinâmico: ex.: `/commit` vê o
`git diff`, `/review` vê as mudanças em staging. Usabilidade objectiva e barata.

## Relação com templates/receitas
Há **três** mecanismos sobrepostos: templates (pi), receitas (goose), comandos (opencode).
**Decisão:** implementar **um só** — comandos = templates **+** expansão de shell/contexto
+ (opcional) parâmetros. Evitar três formatos.

## O que o ktu tem hoje
CLI com subcomandos; sem comandos `/` nem expansão de contexto.

## Pré-condições para reconsiderar
- Resolver a unificação (um mecanismo, não três).

## Custo e risco
- **Esforço:** baixo-médio (loader + expansão + integração com o `Guard`).
- **Risco de não fazer:** redigitar contexto.
- **Risco de fazer:** **execução de shell a partir de um `.md`** — tem de passar pelo
  `Guard`/confinamento (não é atalho privilegiado).

## Decisão pendente
Promover como o mecanismo único (substituindo templates/receitas na decisão). A expansão
de shell corre como **tool** (guardrail + timeout), nunca fora do funil.
