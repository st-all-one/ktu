---
id: fact_005rfd2l
type: fact
statement: "Revisao do opencode: 4 a incorporar (agentes declarativos, AGENTS.md aninhados, comandos com shell, formatter)"
created_at: 2026-10-17T12:46:26.852Z
body_hash: aeafb63e
schema_version: 2
anchors: [wiki/revisao-opencode.md]
---
wiki/revisao-opencode.md analisa o REF/opencode (monorepo Bun/TS v2.0.27; 394 600 linhas src; ~37 pacotes) com o filtro simplicidade/kernel/usabilidade. Incorporar: agentes declarativos (.opencode/agent/*.md: mode/model/tools allow-deny), AGENTS.md aninhados (instruction-discovery + /learn), comandos com shell embutido (.opencode/command/*.md com !`cmd`), formatter (auto-formatar apos editar). Avaliar: PTY persistente, referencias externas, simulation (testes deterministicos), custom tools (.opencode/tool/*.ts). Consolidam-se com permission-store (permissoes declarativas + PermissionSaved) e codemode (dois referenciais o implementam). Nao trazer: Effect-TS, SQLite/effect-drizzle, server multi-instancia/location/worktree, desktop/GUI/web, enterprise/managed-policy/SSO, ai/merman/latex, OAuth complexo, MCP. As 8 candidatas ficaram como paginas 51-58 no wiki/backlog/.