# Receitas reutilizáveis (recipes)

- **Estado:** candidata (do `goose`) — a decidir.
- **Origem:** `REF/goose/crates/goose/src/recipe`, `documentation/docs/guides/recipes/`;
  `wiki/revisao-goose.md` §2.1.
- **Âncora:** `.agents/`, `crates/ktu-cli/src/run.rs`, `crates/ktu-core/src/plan.rs`.

## O que é
Definições **declarativas e versionadas** de tarefas reutilizáveis: `title`, `description`,
`instructions`, `prompt`, `parameters`, `settings` (provider/modelo/max_turns), `sub_recipes`.
Lançadas por `ktu run --recipe <nome>` (ou `/recipe` na TUI). No goose são YAML/JSON; no ktu
pode ser **TOML** (sem dependência nova).

## Porque interessa
Repetir/reproduzir workflows é usabilidade objectiva — a bateria `quality_test` já é isto à
mão. Casa com o **modo plano** (uma receita pode semear o plano e as tarefas). É **dado**,
não efeito.

## O que o ktu tem hoje
`ktu run` com prompt livre; modo plano (`.ktu/plan/*.json`); guardrails declarativos como
precedente de "dado em `.agents/`".

## Pré-condições para reconsiderar
- Um workflow que se repita (ex.: a própria bateria, ou revisões periódicas).
- Decisão sobre o formato (TOML vs YAML).

## Custo e risco
- **Esforço:** baixo-médio (schema + loader fail-closed + `--recipe` + `--param`).
- **Risco de não fazer:** repetir instruções longas à mão.
- **Risco de fazer (YAGNI):** mais um formato declarativo a manter.

## Decisão pendente
Promover se houver ≥2 workflows repetidos. Começar por TOML, fail-closed, e mapear
sub-receitas para subagentes.
