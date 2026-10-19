---
id: task_01jzwiv3
type: task
statement: Projecao idempotente por id (dedup) + recuperacao de SteerQueued no resume
created_at: 2026-10-18T13:32:17.243Z
body_hash: 9150052f
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T14:01:07.068Z
    notes: "Projecao idempotente por id (dedup) + recuperacao de SteerQueued no resume: SessionLog::{pending_steers, knows_steer}; derive_messages descarta um Steer repetido; steering::inject entrega as pendentes (inclui as retomadas do disco) antes de admitir as novas. Testes: a_queued_steer_is_pending_until_delivered, a_repeated_steer_id_is_projected_once, a_repeated_queued_steer_id_is_pending_once, a_pending_steer_is_delivered_in_the_next_turn."
anchors: [wiki/implementation_phase3/F1-steering-duravel.md]
status: closed
scope: task
---
<!-- knudge:parent task_00ly3qez -->