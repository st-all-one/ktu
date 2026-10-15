---
question_id: Q2
question_issue: "sugira uma atualização para o /REF/knudge visando melhorar a usabilidade quando operado pelo humano"
expected_answer_ideia: "sugerir um novo subcomando 'human' e/ou flag '--human' que entregue o texto em uma estrutura mais amigável, como markdown com frontmatter ou texto colorido com header line indicando o que cada cor é, ou algo equivalente"
provider: opencode-go
model: longcat-2.5-preview-free
thinking-level: xhigh
cache: true
same_session: true
compacted: false
agent: ktu
---
Aqui vai uma sugestão de atualização para melhorar a usabilidade humana, baseada no que encontrei no código:

---

## Sugestão: Modo de saída legível para humanos (`--human`)

### Problema

A saída do `kd` é desenhada para **máquinas e agentes de IA**, não para humanos. Exemplos:

- `kd ask` produz colunas como `id|statement|score|why` com valores técnicos (`why=lexical`, `why=ppr`, `why=stars`).
- `kd task list` produz `id|scope|status|statement` — ilegível sem decorar o formato.
- `kd doctor` lista achados com códigos de erro e caminhos internos.
- Não há nenhum resumo em linguagem natural do que está na memória.

### Proposta

Adicionar uma flag global `--human` (ou `--pretty`) que reformata a saída para consumo humano:

```bash
# Antes (padrão, para agentes):
kd ask "rate limit" --brief
# fact_01abc|O gateway limita 100 rps por chave

# Com --human:
kd ask "rate limit" --human
# 📝 O gateway limita 100 rps por chave
#    📌 fact_01abc · tipo: fact · tags: gateway · âncora: src/gateway.rs
```

### O que mudaria

| Comando | Hoje | Com `--human` |
|---|---|---|
| `kd ask` | `id\|statement\|score\|why` | Lista legível com ícones, tags e âncoras destacadas |
| `kd task list` | `id\|scope\|status\|statement` | Tabela alinhada com emojis de estado (✅ ⏳ 🔒) |
| `kd doctor` | Lista técnica de achados | Resumo em linguagem natural: "3 notas com âncoras quebradas. Execute `kd doctor --fix` para reparar." |
| `kd rewind` | Bloco de contexto estruturado | Resumo narrativo: "Na última sessão você trabalhou em X, decidiu Y, e ficou pendente Z." |

### Implementação sugerida

1. **Novo módulo** `crates/knudge-cli/src/output/human.rs` com funções de renderização paralelas às existentes.
2. **Flag global** `--human` no `Cli` (em `crates/knudge-cli/src/cli/mod.rs`).
3. **Branching no `Output`**: se `--human`, usa o renderizador humano; senão, mantém o comportamento atual.
4. **Zero impacto em agentes**: o comportamento padrão não muda; `--json` continua idêntico.

### Benefícios

- Humanos conseguem **explorar e auditar** a memória sem decorar formato de máquina.
- Reduz a barreira de entrada para quem não é agente de IA.
- Mantém o contrato de máquina intacto (agentes continuam usando `--json` ou o formato pipe).

---

### Bónus: outras melhorias de usabilidade humana

1. **`kd explore`** — modo interativo (TUI simples) para navegar pela memória com setas do teclado.
2. **Confirmação interativa em `kd forget --purge`** — pedir "tem certeza?" antes de apagar permanentemente.
3. **`kd stats`** — resumo do corpus: "42 notas, 8 decisões, 3 tarefas abertas, última atualização há 2 dias."

---

Quer que eu implemente alguma destas sugestões?
