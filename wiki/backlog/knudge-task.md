# knudge — hierarquia de trabalho (`epic ⊃ issue ⊃ task`)

- **Estado:** candidata (do `knudge` 0.6.1) — **avaliar**.
- **Origem:** `crates/knudge/wiki/integration/06_tarefas_e_handoff.md`,
  `crates/knudge/wiki/specs/tarefas.md`; `wiki/revisao-knudge.md` §6.2.
- **Âncora:** `crates/ktu-core/src/plan.rs`, `crates/ktu-core/src/plan/`.

## O que é
O trabalho é uma **view derivada**: `epic ⊃ {issue ⊃ task | task}`; o `epic` é `scope=epic` com
`type` omitido. `TaskSpec`/`submit` cria; `context_of`/`progress_of`/`impact`/`subtree`/
`program_of` navegam; `flow` deriva cycle/lead/throughput + **caminho crítico** (PERT/CPM); fechar
exige **evidência** (`close_task`).

## Porque interessa
O ktu tem **modo plano** (`plan.rs`) com artefactos `.ktu/plan/`. A hierarquia do knudge é mais
rica (programa → épico → issue → tarefa, com progresso/impacto/caminho crítico) e **derivada** do
log — sem novo armazenamento. Poderia substituir/complementar o modo plano.

## O que o ktu tem hoje
`plan.rs` + `plan/store.rs` (plano por sessão, gravado atomicamente); sem hierarquia nem métricas.

## Pré-condições para reconsiderar
- Trabalho estruturado (épicos/issues/tarefas) no fluxo do agente.

## Custo e risco
- **Esforço:** médio-alto (adoptar `task::*` + expor na porta/CLI).
- **Risco de não fazer:** plano plano (sem hierarquia).
- **Risco de fazer:** alargar a superfície do agente — só com consumidor concreto (§0).

## Decisão pendente
Avaliar. Se o ktu ganhar trabalho estruturado, adoptar `task::submit`/`context_of`/`flow` como
view derivada; a porta mantém tipos do ktu. Senão, manter o modo plano actual.
