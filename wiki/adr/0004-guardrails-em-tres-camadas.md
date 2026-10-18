# 0004 — Guardrails em três camadas, fail-closed

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/guard.rs`, `crates/ktu-core/src/guardrails.rs`,
  `crates/ktu-plugin/src/`

## Contexto

Um agente autónomo escreve ficheiros e corre comandos. As tentativas anteriores tentaram resolver
isso com um **motor de política** genérico (regras, prioridades, *rate-distortion*, *containment*),
que nunca chegou a operar. Ao mesmo tempo, um projecto real precisa de regras que o utilizador
possa escrever sem recompilar o agente.

## Decisão

Três camadas, com poderes **assimétricos** (I10/I11):

1. **Invariantes em código** (`StructuralGuard`): o read-only do `explorer`, o modo plano, a
   fronteira do *root*. Não são configuráveis e nenhum ficheiro os desliga.
2. **Guardrails declarativos** (`.agents/guardrails/*.md`): regras de projecto, *fail-closed* — um
   ficheiro inválido impede o arranque e diz qual (I16).
3. **Plugins/hooks**: podem **acrescentar** gates e transformar dados; nunca transformam `Deny` em
   `Allow`. Timeout ou erro num gate = `Deny` (I11).

Nenhuma tool produz efeito antes de `Guard::evaluate` devolver `Allow`; `Deny`/`Escalate` não são
ignorados por nenhum caminho (incl. subagentes e receitas com `shell`).

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Motor de política completo (regras + prioridades + *containment*) | YAGNI explícito: entrou antes do loop no katu e nunca operou; o ktu só o admite com sintoma |
| Regras só em código | o utilizador não consegue proteger o próprio repositório sem recompilar |
| Plugins com poder de `Allow` | um plugin passaria a poder libertar uma escrita estruturalmente negada (quebra I9/I10) |
| Permissões só por prompt ("o modelo é bem comportado") | não é uma garantia: o teste adversarial por regra do catálogo existe precisamente para isso |

## Consequências

- Há **um** ponto de decisão (`Guard::evaluate`) e uma ordem de enforcement testada; a fachada
  reutiliza-o fora de um turno (`facade::evaluate_fact`) em vez de compor uma segunda cadeia.
- A recusa é accionável: diz a regra e o remédio (`ktu permission set <tool> …`).
- Custo: três camadas para explicar e testar; em troca, o que é estrutural nunca é negociável.
