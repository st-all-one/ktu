---
id: fact_006ilnqq
type: fact
statement: "bash confinado ao root: recusa ~/$HOME, .. e absolutos fora do root (risk_01ua5ei2 mitigado)"
created_at: 2026-10-17T09:40:59.016Z
body_hash: 418b4777
schema_version: 2
anchors: [crates/ktu-tools/src/bash.rs]
---
ktu-tools/src/bash.rs::analyze_paths(script, root) recusa expansao de home, `..`/`../` e caminhos absolutos fora do root, permitindo so prefixos de sistema read-only (/bin,/sbin,/usr,/lib,/lib64,/dev/null,/dev/stdout,/dev/stderr,/dev/fd,/proc,/sys). Aplicado em BashTool::run depois da analise de destrutivos (fail-closed). Testes unitarios (denies_paths_outside_the_root, allows_commands_inside_the_root_and_system_tools) + integracao (tests/bash.rs).