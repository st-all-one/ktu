# Consolidação das duplicatas do backlog

> **Ficheiro único** que funde os grupos de páginas que descrevem o **mesmo mecanismo**
> visto de referenciais diferentes. Cada grupo passa a ter **um mecanismo canónico**; as
> páginas-membro ficam referidas aqui (não apagadas — registo da decisão). O backlog activo
> deixa de ter ~40 páginas redundantes e passa a ter **15 decisões**.
>
> Base: [análise crítica](../backlog_reviews.md) §2.1 e [revisão por propósito](../revisao-proposito.md) §5.

## Como ler

Cada grupo tem: **canónico** (a página que fica), **membros** (as que se fundem), **o mecanismo
unificado**, **o que muda face às páginas isoladas** e **a decisão**. "Fundir" ≠ "implementar":
a decisão de cada grupo mantém-se a mesma que a das suas partes.

---

## G1 — Templates, receitas e comandos
**Canónico:** [41 — prompt-templates](./prompt-templates.md) ·
**Membros:** [19 — receitas](./receitas.md), [53 — comandos-com-shell](./comandos-com-shell.md)

**Mecanismo unificado.** Um **template** é Markdown com frontmatter (`description`, `args`),
invocável por `/nome`; um template com **shell embutido** executa comandos antes de enviar o
prompt (substituição de `!`cmd``). "Receita" (goose) e "comando com shell" (opencode) são o
**mesmo objecto** com dois graus de riqueza: só-prompt → prompt+shell.

**O que muda.** Uma só descoberta (`.agents/templates/*.md` ou `.agents/commands/*.md`), um só
parser, um só ponto de invocação (CLI `/`, TUI, porta). O shell embutido é a variante rica —
**gated** por guardrail (um comando de template passa por `Guard::evaluate` como qualquer efeito).

**Decisão:** **incorporar** o mecanismo único; o shell embutido entra depois, com gate.

## G2 — Compactação
**Canónico:** [22 — resumo-estruturado](./resumo-estruturado.md) ·
**Membro:** [48 — compactacao-parametros](./compactacao-parametros.md)

**Mecanismo unificado.** Compactação com **resumo estruturado** (secções fixas) e **parâmetros**
(`raw_min`/`raw_max`, gatilho, `keep`). O `resumo-estruturado` é o *conteúdo*; a
`compactacao-parametros` é o *controlo*. Um só `BudgetGate` (I14) decide **quando**; o resumo
estruturado decide **o quê**.

**Decisão:** **avaliar com medição** (Q4) — o ktu já compacta; o resumo estruturado só entra se o
teste Q3→Q4 mostrar perda.

## G3 — Output grande / truncagem
**Canónico:** [94 — truncagem-output](./truncagem-output.md) ·
**Membro:** [81 — resposta-grande-ficheiro](./resposta-grande-ficheiro.md)

**Mecanismo unificado.** Uma só política de **truncagem sistemática** (linhas + bytes) para
**toda** a saída de tool; quando o resultado é grande, o corpo vai para **ficheiro** e o contexto
recebe um **ponteiro + prévia** (o `resposta-grande-ficheiro` é o caso "muito grande").

**O que muda.** Um só ponto (o wrapper de tool), não dois. Consistente com o teto de bytes (I14) e
com o taint (I13).

**Decisão:** **incorporar** — é o que evita o thrashing de contexto do katu.

## G4 — Cache de prefixo
**Canónico:** [73 — cache-prefixo](./cache-prefixo.md) ·
**Membro:** [108 — session-warming](./session-warming.md)

**Mecanismo unificado.** Prefixo **declarativo** e estável (system prompt + histórico fixo) para
aproveitar o cache do provider; *warming* é a variante que pré-aquece o prefixo entre sessões. Ambos
dependem da **estabilidade do prefixo** (determinismo, §2.10).

**Decisão:** **avaliar** — só com um provider que cobre cache; medir antes.

## G5 — Saída estruturada / gramática
**Canónico:** [9 — gramatica-restringida](./gramatica-restringida.md) ·
**Membro:** [74 — saida-estruturada](./saida-estruturada.md)

**Mecanismo unificado.** Forçar o output do modelo (GBNF no llama.cpp; `json_schema` num gateway)
para tornar a tool-call **fiável**. É a **mesma alavanca**: restringir a gramática da resposta.

**Decisão:** **avaliar** — opt-in; com o modelo de referência, medir a queda de tool-calls inválidas.

## G6 — Codemode
**Canónico:** [45 — codemode](./codemode.md) ·
**Membro:** [36 — code-mode-tree-sitter](./code-mode-tree-sitter.md)

**Mecanismo unificado.** Compor tools com **código** (o modelo escreve um script que chama tools),
com `tree-sitter` como variante de parsing estrutural. Um só motor de execução sandboxada.

**Decisão:** **avaliar (alto custo)** — dois referenciais implementam-no; grande alavanca, grande
superfície. Só com um teste que o exija.

## G7 — Permissões de tools
**Canónico:** [21 — permission-store](./permission-store.md) ·
**Membro:** [82 — permissoes-por-tool](./permissoes-por-tool.md)

**Mecanismo unificado.** Aprovação por tool com política `Always/Ask/Never` **memorizada**
(o `permission-store` é a memória; o `por-tool` é a granularidade). Integra com `Escalate` (I10) e
com a política-dado (G8).

**Decisão:** **incorporar** — é o controlo determinístico "a pedido", project-bound.

## G8 — Política (dado)
**Canónico:** [1 — motor-de-politica](./motor-de-politica.md) ·
**Membro:** [110 — provider-policy](./provider-policy.md)

**Mecanismo unificado.** Política como **dado** sobre factos tipados (`{effect, action, resource}`,
wildcard, última vence). O `provider-policy` é um **caso** (que provider/modelo por contexto). O
`Guard::evaluate` já usa o tipo de factos — o motor entra sem tocar no loop (§2.12-B).

**Decisão:** **deferido (seam)** — só com consumidor; o provider-policy sozinho é mais barato.

## G9 — Configuração
**Canónico:** [76 — config-precedencia](./config-precedencia.md) ·
**Membros:** [83 — migracoes-config](./migracoes-config.md), [99 — settings-schema](./settings-schema.md), [122 — knudge-config](./knudge-config.md), [100 — credenciais-precedencia](./credenciais-precedencia.md), [config-global](./config-global.md)

**Mecanismo unificado.** Config em **dois níveis** (projecto > global), com precedência
`flag > --params > local > global`, **schema** (JSON Schema / TOML validado), **migrações**
versionadas e **segredos só no global** (`auth.toml`, 0600). O global (`~/.config/ktu/`) traz
`auth.toml`, `config.toml`, `memory.toml` e as pastas `skills/`/`guardrails/`/`plugins/`/`recipes/`
(defaults sobreponíveis; guardrails = **piso** aditivo). O knudge já o implementa — modelo a copiar.

**O que muda.** Uma só página decide o formato, a precedência, a validação e a migração. É a base
do "project-bound config" (uso #4).

**Decisão:** **incorporar** — é o eixo que falta para o ktu ser personalizável por projecto.

## G10 — Git
**Canónico:** [16 — git-versionado](./git-versionado.md) ·
**Membro:** [123 — knudge-git-sync](./knudge-git-sync.md)

**Mecanismo unificado.** Exclusões geridas em `.git/info/exclude` (nunca `.gitignore`),
`.gitattributes` com `merge=union` nos eventos, e `sync` por evento. O `git::sync` do knudge é a
implementação de referência.

**Decisão:** **deferido (seam)** — só quando a memória do agente tiver de ser versionada.

## G11 — Contexto / recall
**Canónico:** [24 — contexto-por-modelo](./contexto-por-modelo.md) ·
**Membro:** [121 — knudge-embeddings](./knudge-embeddings.md)

**Mecanismo unificado.** A **janela de contexto** por modelo (orçamento correcto) e o **canal
semântico** (embeddings) são a mesma família: "o que cabe e o que se recupera". Ambos alimentam o
`BudgetGate` e o recall.

**Decisão:** **avaliar** — o limite por modelo é barato e útil; as embeddings só com evidência de
recall lexical insuficiente.

## G12 — Sessão: fork / snapshot / undo
**Canónico:** [40 — session-tree](./session-tree.md) ·
**Membros:** [78 — snapshot-retomada](./snapshot-retomada.md), [85 — editar-em-sessao](./editar-em-sessao.md), [103 — snapshots-undo](./snapshots-undo.md)

**Mecanismo unificado.** Ramificar/retomar uma sessão a partir de um ponto: a **árvore** é a
estrutura, o **snapshot** é o ponto, o **fork/editar** é a operação, o **undo/redo** é a
navegação. Uma só primitiva de "ponto de retomada".

**Decisão:** **incorporar (Tier 2)** — é a usabilidade de conversa longa (uso #3).

## G13 — Sessão: versões / mover
**Canónico:** [49 — sessoes-versionadas](./sessoes-versionadas.md) ·
**Membro:** [111 — mover-sessao](./mover-sessao.md)

**Mecanismo unificado.** Versionar o **formato** da sessão (com migração) e **mover** a sessão entre
raízes/projectos. Ambos são "a sessão tem identidade e formato próprios".

**Decisão:** **avaliar** — só quando o formato mudar ou houver multi-projecto.

## G14 — Admissão / steering
**Canónico:** [42 — steering-followup](./steering-followup.md) ·
**Membro:** [104 — admissao-inbox](./admissao-inbox.md)

**Mecanismo unificado.** Mensagens do utilizador **durante** um turno: `steer` (redirecciona no
próximo limite seguro) e `queue` (enfileira); a **inbox durável** garante que nenhuma se perde e que
é idempotente. É o controlo determinístico em tempo real.

**Decisão:** **incorporar (Tier 2)** — par do controlo determinístico.

## G15 — Tool calls declaradas / reparação
**Canónico:** [67 — tool-calls-declaradas](./tool-calls-declaradas.md) ·
**Membro:** [107 — tool-input-repair](./tool-input-repair.md)

**Mecanismo unificado.** Modelos fracos emitem a tool-call como **texto** e/ou com **JSON
quebrado**; o parser tem de (a) reconhecer a forma declarada em texto e (b) **reparar** o input
quando a reparação é inequívoca (profundidade limitada). Um só parser + reparador.

**Decisão:** **incorporar** — é o que torna modelos locais utilizáveis (fiabilidade do loop).

---

## Resumo

| Grupo | Canónico | Membros fundidos | Decisão |
|---|---|---|---|
| G1 Templates/receitas/comandos | 41 | 19, 53 | incorporar |
| G2 Compactação | 22 | 48 | avaliar (Q4) |
| G3 Output/truncagem | 94 | 81 | incorporar |
| G4 Cache de prefixo | 73 | 108 | avaliar |
| G5 Structured output | 9 | 74 | avaliar |
| G6 Codemode | 45 | 36 | avaliar (alto custo) |
| G7 Permissões de tools | 21 | 82 | incorporar |
| G8 Política (dado) | 1 | 110 | deferido (seam) |
| G9 Configuração | 76 | 83, 99, 122, 100, config-global | incorporar |
| G10 Git | 16 | 123 | deferido (seam) |
| G11 Contexto/recall | 24 | 121 | avaliar |
| G12 Sessão fork/snapshot | 40 | 78, 85, 103 | incorporar (T2) |
| G13 Sessão versões/mover | 49 | 111 | avaliar |
| G14 Admissão/steering | 42 | 104 | incorporar (T2) |
| G15 Tool calls/reparação | 67 | 107 | incorporar |

**Efeito:** 36 páginas → **15 decisões** (o backlog activo encolhe sem perder informação).
