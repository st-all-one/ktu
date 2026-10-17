# Cancelamento dentro das tools (kill do grupo)

- **Estado:** candidata (do `katu`, WL3-3) — **avaliar**.
- **Origem:** `REF/katu/wiki/_ref/plan/OPTIMIZATION_PLAN.md` (WL3-3); `wiki/revisao-katu.md` §3.8.
- **Âncora:** `crates/ktu-tools/src/bash.rs`, `crates/ktu-core/src/cancel.rs`.

## O que é
`Ports`/`TurnRequest.cancel` → `ExecTool.cancel` → `Process::run(request, cancel)`; o `StdProcess`
mata o **grupo** de processos ao cancelar. Sem isto, cancelar a meio de um `bash` de 30 s deixa o
processo pendurado.

## Porque interessa
O ktu tem `CancelToken` (`cancel.rs`), mas a propagação até ao processo não foi confirmada.
Cancelar deve matar o que está a correr — não só marcar a flag.

## O que o ktu tem hoje
`cancel.rs` (`CancelToken`, injecta resultados sintéticos, `StopReason::Cancelled`); o `bash` foi
confinado (mitigação `risk_01ua5ei2`). Propagar o cancel até ao `kill` do grupo não confirmado.

## Pré-condições para reconsiderar
- Verificar se o `bash` já observa o `CancelToken` durante a execução.

## Custo e risco
- **Esforço:** médio (porta `Process::run(request, cancel)` + kill do grupo).
- **Risco de não fazer:** processos pendurados após cancelar.
- **Risco de fazer:** matar o grupo errado — o confinamento do `bash` limita o escopo; teste com
  processo filho.

## Decisão pendente
Avaliar. Se o `bash` já consulta o cancelamento, fechar; senão, propagar até ao `kill` do grupo
com teste `cancel_kills_the_running_child`.
