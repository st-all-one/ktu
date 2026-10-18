---
id: task_00dkc6dk
type: task
statement: H9 — Crate facade: embutir o ktu noutro projeto Rust
created_at: 2026-10-17T22:53:40.078Z
body_hash: ae9a0468
schema_version: 2
tags: [fase2]
results_in: [task_01jalrhj, task_00ghiyx7, task_01nj66mb, task_01ka615s]
revision: 7
outcomes:
  - status: success
    recorded_at: 2026-10-18T02:11:38.483Z
    notes: "H9 (crate facade) concluida: Ktu/KtuBuilder/TurnOutcome em ktu-core com doc test executavel e teste de incorporacao; a CLI deixou de ter composicao propria (turn_runtime::execute_turn monta os adaptadores reais e delega na fachada) - uma so composicao, logo o caminho crate e os caminhos da CLI nao podem divergir (M4). Gates: make check + typos + supply-chain verdes; cargo doc limpo. Prova em wiki/implementation_phase2/H9-crate-facade.md; contrato em wiki/fachada-crate.md. Adiado (YAGNI, com razao): fachada que constroi adaptadores std (crate nova), run_turn/Deps privados, Session propria, Env/Rng na fachada."
anchors: [wiki/backlog/crate-facade.md]
status: closed
scope: issue
---
<!-- knudge:parent epic_01j7ou2p -->