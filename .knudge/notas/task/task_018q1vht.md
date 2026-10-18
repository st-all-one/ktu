---
id: task_018q1vht
type: task
statement: ktu memory drain service (install/subscribe/status/uninstall/reconcile) systemd+launchd
created_at: 2026-10-17T22:54:40.352Z
body_hash: ddb8d544
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T01:49:37.335Z
    notes: "ktu memory drain service [--install|--subscribe|--unsubscribe|--status|--uninstall|--reconcile|--dry-run] (H8): wrapper fino sobre o worker EMBUTIDO scripts/ktu-idle.sh (include_str!, sem supply-chain de rede), materializado em <config>/ktu/ktu-idle.sh no install (copia-se para ~/.local/bin/ktu-idle). systemd --user (ktu-embed.service persistente + ktu-idle.service/.timer) ou launchd; sem agendador = linha de cron. Multi-projecto (idle.projects), --reconcile alinha memory.toml, --remove-model move para o trash (nunca rm), --dry-run so mostra o plano. Testes: worker_defines_every_action (um heredoc mal terminado apagava funcoes em silencio - bug real apanhado), service_status_reports_the_worker_without_errors, service_dry_run, uninstall_moves_the_model_to_the_trash."
anchors: [wiki/backlog/memory-drain-service.md]
status: closed
scope: task
---
<!-- knudge:parent task_01t8rm0l -->