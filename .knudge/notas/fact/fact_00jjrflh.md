---
id: fact_00jjrflh
type: fact
statement: Plugins de exemplo: input-note (transform user.input) e guard-bash (gate nega bash destrutivo)
created_at: 2026-10-17T07:02:34.108Z
body_hash: 9eadf56e
schema_version: 2
anchors: [examples/plugins/guard-bash/run.sh]
---
examples/plugins/input-note (mode=transform, prefixa o input) e examples/plugins/guard-bash (mode=gate, nega rm -rf/mkfs/push --force/> /dev/). Comandos ktu plugins list|reload|doctor. Provado por crates/ktu-plugin/tests/process.rs e smoke com ktu run --provider mock (turn.start prompt = [exemplo] ...).