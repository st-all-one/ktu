---
id: task_01moxmoz
type: task
statement: A10 Exposicao de tools declared/deferred + tool_search BM25 apos H12
created_at: 2026-10-18T02:58:07.565Z
body_hash: f7afc4b8
schema_version: 2
tags: [fase3]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T03:20:14.581Z
    notes: "A10 implementado: exposicao de tools declared/deferred + tool_search. ToolExecutor::deferred() (default vazio: retro-compativel) e ScopedTools::deferred() (cruza a exposicao com o conjunto do agente - uma diferida fora do conjunto nunca e oferecida); novo ktu-core/src/tool_search.rs com spec() (tool_search), parse_query e rank() BM25 (k1=1.2, b=0.75, nome pesa o dobro, idf com ln_1p, empates desfeitos pelo nome - deterministico) + render() com o schema das tools encontradas; driver_specs passa a oferecer tool_search SO quando ha diferidas (nada de superficie morta) e o Driver promove as encontradas para o catalogo do pedido seguinte, logando tool.loaded por cada uma (nunca em silencio); a fachada (e o host de subagentes) acrescentam tool_search ao conjunto do agente quando ha diferidas - a exposicao e de CATALOGO, nao de permissao (o guard conhece o conjunto todo); Config::deferred_tools (array TOML ou lista por virgulas, com merge local>global) + Toolbox::with_deferred(names) + wiring no turn_runtime. Testes: 6 unit (ranking, empates, top_k, render), ktu-core/tests/loop.rs::tool_search_promotes_a_deferred_tool_to_the_next_request (prova o gate: tool_counts == [2,3,3] - a tool aparece no pedido SEGUINTE), ktu-tools/tests/plan_mode.rs::deferred_tools_leave_the_catalog_but_stay_executable."
anchors: [wiki/implementation_phase2/ADIADOS.md]
status: closed
scope: task
---
<!-- knudge:parent task_016wnwcs -->