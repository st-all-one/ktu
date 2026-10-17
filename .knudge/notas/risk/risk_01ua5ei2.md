---
id: risk_01ua5ei2
type: risk
statement: bash nao confina paths ao root: o modelo pode ler/escrever fora do workspace isolado
created_at: 2026-10-17T07:14:09.154Z
body_hash: e2f2eff3
schema_version: 2
anchors: [crates/ktu-tools/src/fs_tools.rs]
---
O guard estrutural confina path apenas quando a tool tem argumento `path` (read_file/write_file/list_dir). A tool `bash` corre comandos sem confinamento de cwd/paths, pelo que um modelo pode escrever com caminhos absolutos fora do root (observado na bateria: o modelo escreveu exploration/knudge/ktu/*.md no repo em vez do workspace isolado). Impacto: a isolacao da bateria de qualidade depende do comportamento do modelo. Mitigacao futura: confinar o cwd do bash ao root e negar caminhos absolutos/.. nos comandos.