# Scheduler (tarefas agendadas)

- **Estado:** candidata (do `goose`) — **não trazer**.
- **Origem:** `REF/goose/crates/goose/src/scheduler*`; `wiki/revisao-goose.md` §3.
- **Âncora:** `crates/ktu-cli/src/main.rs`.

## O que é
Agendar tarefas/recetas para correr em horários (cron-like), dentro do agente.

## Porque não entra
YAGNI: o agente corre a pedido. Agendamento é orquestração externa (cron/systemd) que pode
invocar `ktu run` — não precisa de viver no kernel.

## Pré-condições para reconsiderar
- Uma tarefa que tenha de correr **sem interacção** e com estado próprio (retries,
  backoff, histórico).

## Custo e risco
- **Esforço:** médio (agenda + persistência + supervisão).
- **Risco de não fazer:** usar cron externo (adequado).
- **Risco de fazer (YAGNI):** um agendador dentro do agente.

## Decisão pendente
Manter fora; agendar via SO. Só reconsiderar se o agendamento precisar de estado do agente.
