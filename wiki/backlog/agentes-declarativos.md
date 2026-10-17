# Agentes declarativos

- **Estado:** candidata (do `opencode`) — **incorporar**.
- **Origem:** `REF/opencode/.opencode/agent/*.md`; `wiki/revisao-opencode.md` §2.1.
- **Âncora:** `crates/ktu-core/src/agent/spec.rs`, `crates/ktu-core/src/subagent.rs`.

## O que é
Agentes definidos como **dados** (Markdown + frontmatter): `mode` (`primary`/`subagent`),
`hidden`, `model`, `color`, `tools` (allow/deny por wildcard, ex.:
`{"*": false, "github-triage": true}`). O opencode traz **build** (acesso total) e **plan**
(read-only) embutidos e um subagente **general**.

## Porque interessa
O ktu tem `AgentSpec` **em código** (primário + `explorer`). Torná-lo declarativo é
**kernel** (dado) e usabilidade (definir agentes próprios) — e já há precedente
(guardrails declarativos em `.agents/guardrails/`). O modo plano read-only (I15) é a
mesma ideia do agente `plan`.

## O que o ktu tem hoje
`AgentSpec` + registo (primário + `explorer` read-only estrutural); modo plano; subagentes
com fan-out e profundidade máx. 1.

## Pré-condições para reconsiderar
- Necessidade de agentes próprios (além do primário/explorer).
- Um schema de frontmatter e um loader **fail-closed**.

## Custo e risco
- **Esforço:** baixo-médio (schema + loader + integração com `AgentSpec`/guard).
- **Risco de não fazer:** agentes fixos no código.
- **Risco de fazer:** agentes mal-definidos alargarem capacidades — mitigar com allow/deny
  e validação.

## Decisão pendente
Promover; colocar em `.agents/agents/*.md`, fail-closed, com `mode`/`tools`/`model` e
`tools: {"*": false, ...}` por omissão (mínimo privilégio).
