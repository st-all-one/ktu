---
question_id: Q2
same_session: true
question_issue: "sugira uma atualização para o /REF/knudge visando melhorar a usabilidade quando operado pelo humano" 
expected_answer_ideia: "sugerir um novo subcomando 'human' e/ou flag '--human' que entregue o texto em uma estrutura mais amigável, como markdown com frontmatter ou texto colorido com header line indicando o que cada cor é, ou algo equivalente"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: pi 
---

# PI_SUGGESTION — atualização de usabilidade do `knudge` para operação humana

> **Alvo:** `/REF/knudge` (workspace Rust, versão 0.5.2, binários `kd` + `knudge-mcp`)
> **Tema:** tornar o `kd` utilizável por uma **pessoa** no terminal, sem quebrar o contrato
> de máquina que o agente já consome.
> **Base:** auditoria existente `plan/proposals/usabilidade_humana.md` (achados H1–H8) —
> esta proposta a consolida, prioriza em ondas e acrescenta o que faltava (H9–H11).

---

## 1. O problema, em uma frase

O `kd` foi desenhado com **o usuário = o LLM** (`plan/00_panorama.md` §1: *"não há humano lendo ou
editando; legibilidade humana é irrelevante"*), mas o `init`, o `doctor`, o `AGENTS.md` e toda a
`wiki/usage/` **são escritos para humanos**. Onde os dois papéis se cruzam, hoje, o humano leva
erro no primeiro comando, perde dado sem aviso e não tem caminho para *ver, ler e corrigir* uma
nota.

O objetivo desta atualização **não** é transformar o `kd` num app interativo. É fechar o
**loop humano mínimo** — `achar → ler → corrigir → apagar` — e parar de punir quem ousa usar a
ferramenta na mão, **mantendo `stdout`/`--json` byte-a-byte estáveis** para o agente.

---

## 2. A sugestão: uma passada "humano em primeiro lugar"

Uma única atualização, com escopo controlado e três frentes, nesta ordem:

### Onda 1 — "primeiro contato e não perder dado" (bloqueante, baixo risco)

Corrige as duas falhas que mais machucam e não tocam o contrato de bytes.

| # | Mudança | Verificado em | Aceite |
|---|---|---|---|
| **H1** | `kd init` e o bloco do `AGENTS.md` passam a ensinar a sintaxe **correta** (`--summary`), não a pré-D140. | `commands/init.rs:183` (`próximos: kd write "<afirmação>" …`) e `core/git/agent_md.rs:41` | copiar o comando sugerido roda **exit 0**; golden `tests/golden/init.txt` atualizado |
| **H2** | `doctor --fix` **para de remover âncora em silêncio**: (a) renderiza a lista `fixed:` no **texto** (hoje só sai no `--json`, `doctor/render.rs:158`); (b) âncora para path inexistente vira `warn` (não `fail`) e a remoção pede aceite/`--force`. | `core/health/doctor/fix.rs:195` (`fix_broken_anchors`) | nenhuma âncora some sem aparecer no stdout; teste de regressão da nota ancorada em arquivo futuro |
| **H3** | Separar os rótulos que hoje se contradizem: `âncoras stale` (check) × `âncoras órfãs` (auditoria), e o resumo surfa **qualquer** achado da auditoria mesmo com check `ok`. | `core/health/doctor` (checks vs `audit.rs`) | `kd doctor` e `kd doctor --explain` concordam |

### Onda 2 — "fechar o loop ler/corrigir" (alto valor, médio risco)

| # | Mudança | Verificado em | Aceite |
|---|---|---|---|
| **H8** | `ask --id` e o `--json` passam a expor o **caminho do arquivo** (`path`), e o `ask` termina com `próximos: kd write --update <ID> --summary "…"` (convenção D165). | `commands/ask/render.rs` (`hit_json`) | `kd ask --id <ID>` mostra onde a nota vive; humano corrige sem decorar id |
| **H4** | Novo check `id`: derivar `id = hash(type + U+001F + normalize(statement))` e comparar com o id em disco; `--fix` re-renomeia + reescreve arestas (ou `superseded` + novo id). | `core/health/doctor` (hoje só checa `body_hash`) | editar o `statement` à mão deixa de gerar id-mentira silencioso |
| **H7** | `write` deixa claro "criado" × "rejeitado como duplicata": `rejected`/`merged` ganham texto inequívoco (mostrar statement anterior/novo) e `próximos:` com `--update`. Exit code distinto só se a matriz de aceitação permitir. | `core/write/mod.rs` | humano entende, no terminal, se algo foi gravado |
| **H6** | `config get` consulta a config **efetiva** (`session.config()`, default+global+projeto) e marca a origem; `did_you_mean` nunca sugere a própria string digitada. | `commands/config_cmd.rs:95`; `schema/suggest.rs` | `kd config get --key recall.semantic` devolve valor + origem |

### Onda 3 — "leitura humana de verdade" (novos achados H9–H11)

| # | Mudança | Situação hoje | Aceite |
|---|---|---|---|
| **H9** | **Saída amigável em TTY, sem mexer no pipe.** Quando `stdout` é terminal, o `ask`/`task list`/`doctor` renderizam em colunas legíveis (id curto, statement, tipo, âncora); com pipe/`--json`, continua o formato atual. `--json` nunca muda. | hoje o default é `id\|statement\|score\|why`, formato de máquina, em qualquer contexto | humano lê sem decodificar `\|`; scripts intactos |
| **H10** | **Comando de leitura direta:** `kd show <ID>` (corpo completo + frontmatter legível + `path`) e `kd log`/`timeline` (notas por data/tipo). Não muda a filosofia "nota é a verdade", só dá um ponto de entrada. | não existe `show`/`ls`/`log` (`unrecognized subcommand`); `ask --id` é o único caminho e não traz o `path` | humano navega o corpus sem saber `id` de antemão |
| **H11** | **Higiene de mensagens:** remover os sufixos `(D93)`, `(D84)`, `(D54)`… dos erros visíveis (`write/mod.rs:141`, `task/mutate.rs:145`, `forget_sync.rs:76`, `task/spec.rs:80`, `task/hierarchy.rs:16,38`); o rastro da decisão fica em `DIVERGENCES.md`/testes, não no terminal. | 7 mensagens vazam ids internos do plano | erros acionáveis, sem ruído de mantenedor |

---

## 3. Como executar (alinhado às regras do projeto)

- **Contrato intacto.** `stdout` de dados e `--json` permanecem o contrato de máquina (`16_cli_surface.md`
  §3, D71/R20). A Onda 3 só ramifica por `IsTerminal` — o mesmo sinal já usado em `ask/mod.rs:83`.
- **Decisão formal.** Abrir `D215` em `plan/03_decisoes-fechadas.md` ("passada de usabilidade
  humana") e atualizar `17_matriz_aceitacao.md` sempre que mudar pipe/exit (H7, H9).
- **Regressão obrigatória.** Cada achado entra com teste (AGENTS.md §6: "bug encontrado ⇒ adicione
  o teste de regressão"); H1 e H5 exigem atualizar goldens.
- **Arquivos ≤ 300 linhas, sem `unwrap`/`panic`** — a passada não introduz exceções.
- **Ordem de merge sugerida:** H1 → H11 → H6 (triviais) → H2 (perda de dado) → H3/H7 → H8 →
  H4 → H9/H10. H2 não deve esperar a Onda 3, porque é a única que **perde dado**.

---

## 4. Guardrails (o que esta atualização NÃO faz)

- **Nada de TUI interativo, fuzzy-finder, editor embutido ou `watch` ao vivo.** Isso contraria a
  natureza "um binário, sem servidor, sem daemon" e é onde a proposta perde o foco.
- **Nada de ACL, colaboração ou sync com nuvem** — fora do escopo declarado do knudge.
- **Nada de cor obrigatória** — quando H9 colorizar, respeitar `NO_COLOR` e `--json`.
- **Nada de renomear verbos.** A superfície congelada (`16_cli_surface.md`) fica; só ganha
  flags/subcomandos de leitura (`show`, `log`, `--path`).

---

## 5. Por que vale a pena

- **Primeiro contato consertado (H1):** hoje o próprio `init` ensina um comando que falha — péssima
  primeira impressão e barreira para adoção.
- **Sem perda de dado (H2):** o único achado que **destrói** informação (liga nota↔código) sem avisar.
- **Loop humano completo (H8/H9/H10):** transforma o `kd` de "ferramenta só de robô" em memória de
  projeto auditável por gente — que o README já promete (*"a memória que você consegue auditar depois"*).
- **Risco contido:** as Ondas 1 e 2 são correções pequenas e localizadas; o contrato do agente não muda.

> **Resumo executivo:** uma atualização em 3 ondas que ensina a sintaxe certa no `init`, para de
> apagar âncoras em silêncio, dá `path` e leitura amigável às notas, e limpa ruído das mensagens —
> tudo preservando byte-a-byte o `stdout`/`--json` que o agente consome hoje.
