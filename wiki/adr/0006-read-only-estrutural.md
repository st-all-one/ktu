# 0006 — Read-only estrutural (subagentes e modo plano)

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/agent/spec.rs`, `crates/ktu-core/src/subagent.rs`,
  `crates/ktu-core/src/plan.rs`

## Contexto

Explorar um repositório grande exige *fan-out* (vários subagentes a ler) e, muitas vezes, um modo
que **não** toque em nada (planear antes de agir). Se o read-only for uma instrução no prompt, o
agente pode "esquecer-se" — e um subagente com `write_file` é um risco real.

## Decisão

O read-only é **estrutural**, em dois sítios:

- **Subagentes**: cada `AgentSpec` tem um conjunto de tools (`ScopedTools`); a thread de tools
  **recusa por capacidade** uma tool fora do conjunto (I9) — mesmo que o modelo a chame.
- **Modo plano**: o catálogo passa a ter a tool `plan` e não as de escrita; a transição
  `plan → build` exige **aprovação** e é logada (I15). O modelo não escala o próprio modo.

Agentes declarativos (`.agents/agents/*.md`) reutilizam o mesmo `AgentSpec` — não há um segundo
caminho de decisão.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Read-only por prompt | não é garantia; o modelo pode emitir a chamada na mesma |
| Segundo mecanismo de decisão só para subagentes | dois caminhos para manter e para auditar (M4) |
| Modo plano como *flag* do front-end | a thread de tools não a veria: a recusa tem de estar onde a tool executa |
| Subagentes sem tools (só texto) | perdia-se o valor (ler, procurar, resumir) sem ganho de segurança face ao `ScopedTools` |

## Consequências

- Um teste adversarial injecta uma tool de escrita num `explorer` e prova a recusa; outro prova que
  `plan` nega escrita e que `plan → build` sem aprovação é recusado.
- Custo: o conjunto de tools passa a ser dado de primeira classe (e por isso o catálogo pode ser
  diferido — `deferred_tools`/`tool_search`, sem confundir exposição com permissão).
