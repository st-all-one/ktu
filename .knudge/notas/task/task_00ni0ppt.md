---
id: task_00ni0ppt
type: task
statement: Recursos globais: skills/plugins/receitas em ~/.config/ktu/ com precedencia por nome
created_at: 2026-10-17T23:51:50.638Z
body_hash: c311d3ea
schema_version: 2
tags: [fase3]
revision: 5
outcomes:
  - status: success
    recorded_at: 2026-10-18T14:48:32.556Z
    notes: "Recursos globais (skills/plugins/receitas + precedencia). Origin::{Global,Local} (novo origin.rs). Skills: novo ktu_core::skill (dois niveis, nome = directoria, local ganha, tolerante ao YAML do pi/knudge, tecto do AGENTS.md) + seccao no prompt via compose_layers. Receitas: command::load_layered (local ganha). Plugins: PluginHost::layered + entries() com origem (aditivos). guardrails::load_layered partilhado pela fachada e pelo evaluate_fact. CLI: ktu skills ls (novo; surface 47->49/88->89), origem em guardrails/commands/plugins. Testes: 6 skill + 2 guardrails + 1 command + 1 plugin + 5 CLI. Gate G18 verde 10/10."
anchors: [wiki/implementation_phase3/F3-recursos-globais.md]
status: closed
scope: task
---
<!-- knudge:parent task_004bvdzb -->