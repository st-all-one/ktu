# Prompt de `MaxSteps`

- **Estado:** **implementado** (H5; `wiki/implementation_phase2/H5-fiabilidade-loop.md`).
- **Origem:** `REF/opencode/packages/core/src/session/runner/max-steps.ts`;
  `wiki/revisao-opencode-profunda.md` §1.4.
- **Âncora:** `crates/ktu-core/src/agent/mod.rs`, `crates/ktu-core/src/domain.rs`.

## O que é
Ao atingir o máximo de passos, o runner injecta um prompt que **desactiva as tools** e obriga o
modelo a responder **só com texto**: resumo do que foi feito, tarefas restantes e recomendações.

## Porque interessa
O ktu termina com `StopReason::MaxSteps` **sem** dar ao modelo a chance de resumir. Um passo final
textual transforma um corte abrupto num desfecho útil — e deixa o utilizador com um ponto de
partida.

## O que o ktu tem hoje
`StopReason::MaxSteps` sem passo textual final.

## Pré-condições para reconsiderar
- Turnos que atinjam o tecto de passos.

## Custo e risco
- **Esforço:** baixo (prompt final + tools desactivadas nesse passo).
- **Risco de não fazer:** corte abrupto.
- **Risco de fazer:** o modelo ignora a instrução e tenta tools — desactivar as tools no passo.

## Decisão pendente
Promover. Um passo final com tools desactivadas e prompt de resumo; `StopReason::MaxSteps`
preservado; teste de que o passo final não executa tools.
