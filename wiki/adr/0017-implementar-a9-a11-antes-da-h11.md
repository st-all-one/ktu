# 0017 — Implementar A9–A11 antes da H11

- **Status:** aceite
- **Supersedes:** [0016](0016-h10-adiar-receitas-ricas-tool-search-e-agents-na-leitura.md)
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/command.rs`, `crates/ktu-core/src/tool_search.rs`,
  `crates/ktu-core/src/ports/instructions.rs`, `wiki/convencoes.md`

## Contexto

A [0016](0016-h10-adiar-receitas-ricas-tool-search-e-agents-na-leitura.md) adiou três itens da H10
por YAGNI. Antes de arrancar a H11 (governança), o dono do projecto pediu explicitamente que fossem
implementados: são a parte da H10 que faltava, e o custo de os deixar para a Fase 3 era ficar com
uma superfície documentada e não implementada.

## Decisão

Implementar os três, mantendo a disciplina (gate verde, prova por teste e doc no mesmo passo):

- **A9 — receitas ricas**: `CommandSettings` (`provider`/`base_url`/`model`/`max_steps`), argumentos
  `$1`…`$9`/`$ARGUMENTS`, e `shell: true` **opt-in** cujos segmentos passam pelo **mesmo**
  encadeamento de guardrails da fachada (`facade::evaluate_fact`) antes de correr, com a mesma
  `BashTool`; `ktu run --command <nome> [args]`.
- **A10 — exposição de tools**: `deferred_tools` em `.agents/config.toml` (a exposição é **dado**,
  não permissão), `tool_search` com ranking BM25 determinístico e promoção para o **pedido
  seguinte**, registada em `tool.loaded`.
- **A11 — `AGENTS.md` na leitura**: nova porta `Instructions`; o `AGENTS.md` mais próximo é anexado
  ao resultado de `read_file`/`edit_file`, uma vez por turno e por fonte, com `context.local`.

Continuam **fora** (sem consumidor): `sub_recipes`, `parameters` declarados com validação por tipo,
e `model`/`hidden`/`color` nos agentes declarativos.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Manter a [0016](0016-h10-adiar-receitas-ricas-tool-search-e-agents-na-leitura.md) (adiar) | era o plano; o dono pediu explicitamente o contrário — e o adiamento não tem valor se ninguém o reabre |
| Implementar só o A9 (o mais pedido) | os três partilham a mesma zona (convenções) e a mesma doc; deixar dois por fazer manteria a doc a prometer o que não existe |
| Implementar A9 com um executor de *shell* próprio | seria um segundo motor de shell, fora dos guardrails ([0004](0004-guardrails-em-tres-camadas.md)) |
| `tool_search` sem registo no log | quebraria *model-visible ⟺ logged* (I3): a tool carregada entra no contexto e tem de estar no log |

## Consequências

- A superfície cresceu (3 tools/flags novas) — o teto foi subido explicitamente em `surface.toml`
  ([0011](0011-limite-de-400-linhas-e-firewall-mecanico.md)).
- O adiamento por YAGNI continua a valer como regra ([0015](0015-yagni-com-criterio-de-reabertura.md));
  o que mudou foi o critério de reabertura: aqui foi o **dono do projecto**, não um teste.
- As tarefas da Fase 3 (`task_00mtxl9i`, `task_01moxmoz`, `task_01963li2`) foram fechadas com
  outcome e nota; os factos ficaram no `kd`.
