---
id: fact_01da31pn
type: fact
statement: A porta Fs do ktu ganhou read_dir/canonicalize/is_dir; StdFs e MemFs implementam-nas
created_at: 2026-10-13T01:54:07.988Z
body_hash: 7c79927f
schema_version: 2
anchors: [crates/ktu-core/src/ports/mod.rs]
---
Evidência: ktu-core/src/ports/mod.rs (trait Fs) + ktu-tools/src/fs.rs (StdFs) + ktu-core/src/ports/fakes/fs.rs (MemFs indexa ficheiros e directórios). Consequência: list_dir/glob/grep e o confinamento por caminho real deixam de tocar std::fs directamente nas tools.