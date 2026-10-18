# 0016 — H10: adiar receitas ricas, `tool_search` e `AGENTS.md` na leitura

- **Status:** substituída
- **Supersedes:** —
- **Superseded by:** [0017](0017-implementar-a9-a11-antes-da-h11.md)
- **Âncora:** `wiki/implementation_phase2/ADIADOS.md`, `wiki/implementation_phase2/H10-convencoes.md`

## Contexto

A H10 (convenções) tinha três itens que não cabiam no mínimo que a issue pedia: receitas com
`settings`/`parameters`/`sub_recipes` e *shell* embutido, exposição de tools `declared`/`deferred`
com `tool_search`, e injecção do `AGENTS.md` mais próximo na **leitura** de um ficheiro. Nenhum
deles tinha sintoma: o catálogo tinha 13 tools, o mecanismo de templates já existia, e a composição
do `AGENTS.md` no arranque cobria o caso em uso.

## Decisão

Adiar os três (A9, A10, A11 em `ADIADOS.md`), com condição de reabertura registada: *"reabrir quando
o catálogo crescer além do que cabe no prompt / quando houver um workflow repetido / quando a
composição no arranque não chegar"*. As tarefas kd ficaram abertas na Fase 3.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Implementar tudo já | era exactamente o erro simétrico do katu: construir a mais depois de funcionar |
| Implementar só o *shell* embutido das receitas | duplicaria a tool `bash` (que já passa por `Guard::evaluate`) com um segundo caminho de execução |
| `tool_search` por palavras-chave simples | a pré-condição (catálogo grande) não estava satisfeita; o ranking só faz sentido com sintoma |
| Injectar o `AGENTS.md` na leitura sem porta | exigiria um gancho directo no caminho das tools — quebrava a pureza do kernel ([0001](0001-kernel-puro-com-portas.md)) |

## Consequências

- A H10 fechou com o mínimo verificável (templates, agentes declarativos, título de sessão).
- Os três itens ficaram com tarefa própria e critério de reabertura — reversíveis sem re-litígio.
- **Substituída** por [0017](0017-implementar-a9-a11-antes-da-h11.md): o dono do projecto pediu
  explicitamente a implementação antes da H11.
