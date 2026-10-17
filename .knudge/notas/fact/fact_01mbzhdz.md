---
id: fact_01mbzhdz
type: fact
statement: AgentSpec + AgentRegistry definem o conjunto de tools por agente; explorer read-only e estrutural (max_depth 0)
created_at: 2026-10-17T00:14:36.848Z
body_hash: 643186f3
schema_version: 2
anchors: [crates/ktu-core/src/agent/spec.rs, crates/ktu-tools/src/agents.rs]
---
ktu-core/src/agent/spec.rs define AgentSpec (id, BTreeSet de tools, max_depth) e AgentRegistry (primario sempre, explorer opcional) com get/get_by_name. ktu-tools/agents.rs constroi default_registry(): primario ktu com tool_names() e max_depth 1, explorer com EXPLORER_TOOLS (read_file/list_dir/glob/grep/memory_ask) e max_depth 0. StructuralGuard::for_spec(root, spec) deriva allowed_tools e max_depth da spec. O read-only e estrutural (conjunto de tools), nunca prompt. Testes: 3 em spec.rs + 3 em agents.rs; make check verde.