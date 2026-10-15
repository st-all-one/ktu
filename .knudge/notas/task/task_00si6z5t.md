---
id: task_00si6z5t
type: task
statement: Tools de filesystem: read/write/edit/list/glob/grep com erros accionáveis e paths resolvidos no root
created_at: 2026-10-13T00:50:48.769Z
body_hash: 96a4be31
schema_version: 2
tags: [task]
revision: 4
outcomes:
  - status: success
    recorded_at: 2026-10-13T01:54:07.825Z
    notes: "FsTools (ktu-tools/src/fs_tools.rs) com read_file/write_file/edit_file/list_dir/glob/grep; erros accionaveis; confinamento em profundidade: resolve_within lexical + canonicalize do caminho real verificado contra o root real (deteta symlink que escapa) em src/path.rs; glob/grep com walk via porta Fs e matcher puro (search.rs, suporta * ** ?). Porta Fs estendida com read_dir/canonicalize/is_dir (StdFs + MemFs). 5 testes de integracao (fs_tools.rs) + 1 unit; make check verde (59 testes)."
anchors: [crates/ktu-tools/src/fs_tools.rs, crates/ktu-tools/src/path.rs, crates/ktu-tools/src/search.rs]
status: closed
scope: task
---
<!-- knudge:parent task_00sv9qlq -->