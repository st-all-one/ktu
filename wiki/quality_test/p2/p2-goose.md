---
question_id: Q2
same_session: true
question_issue: "sugira uma atualização para o /REF/knudge visando melhorar a usabilidade quando operado pelo humano" 
expected_answer_ideia: "sugerir um novo subcomando 'human' e/ou flag '--human' que entregue o texto em uma estrutura mais amigável, como markdown com frontmatter ou texto colorido com header line indicando o que cada cor é, ou algo equivalente"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: goose 
---

# Sugestão de melhoria de usabilidade para o `knudge` (`kd`)

> **Escopo:** tornar o `kd` agradável quando **operado por uma pessoa** no terminal, **sem
> quebrar** o contrato de máquina que os LLMs já consomem (`pipe`, `--json`, exit codes, goldens).
> **Base:** análise do repo em `REF/knudge` (v0.5.2) + execução real do binário
> (`target/debug/kd`) num projeto de teste.
> **Lema a preservar:** *"stdout = dados; stderr = logs"* (R20) e *"determinismo é requisito"*.

---

## 0. Resumo executivo

O `kd` foi desenhado primeiro para **LLMs**: a saída padrão é um pipe (`id|statement|score|why`),
o help é denso e cheio de referências internas (`D147`, `D209`…), e nada de cor/TTY. Para uma
pessoa, a experiência mostra atritos claros:

| Sintoma observado hoje | Por que incomoda o humano |
|---|---|
| `kd ask "rate limit"` → `decision_003y9k3n\|Rate limit é 100 rps por chave\|0.51\|semantic` | Não entrega a **resposta/corpo**, e `0.51`/`semantic` não dizem nada a quem não conhece o formato |
| `ked write` → `created\|decision_003y9k3n\|r1` | `r1` é sigla interna; falta uma frase legível ("nota criada…") |
| Help com `Print help`/`Print version` (inglês) no meio de tudo em português | Quebra visual e de idioma |
| Help recheado de `(D147)`, `(D209)`, `(D163)`, `(D170/D186)`… (~65 ocorrências só no `cli/`) | IDs de decisão **internos** vazando para o usuário final |
| IDs opacos (`decision_003y9k3n`) exigidos em `--id`, `forget --id`, `task show --id` | Humano não memoriza nem digita hash |
| Sem cor, sem “você quis dizer”, sem paginação | Terminal cru, difícil de escanear |
| `doctor` mostra `fail derived` e ainda **sai com exit 0** | Scripts/olhos humanos não percebem a falha |

**Tese central:** introduzir um **modo humano automático por TTY** (humano vê rico; pipe/`--json`
continua byte-idêntico) + **faxina no help** (idioma e Dxx) + **apelidos curtos para ids**. Isso
resolve ~80% do atrito sem tocar no contrato de bytes.

---

## 1. P1 — Modo humano automático por TTY (maior impacto) ⭐

### Problema
Hoje não há distinção entre "humano lendo" e "máquina consumindo". O `Output` (`crates/knudge-cli/src/output.rs`)
carrega `text` (pipe) e `json`; o texto é sempre o formato de máquina. Só existe detecção de TTY
para **stdin** (`commands/input.rs`, `commands/ask/mod.rs`), nunca para **stdout**.

### Proposta
Quando `stdout` **é um terminal** e não há `--json`, renderizar uma saída **humana**; quando é
pipe/arquivo, manter **exatamente** o formato atual. A decisão de modo vive **na borda**
(`knudge-cli`), preservando o core puro e os goldens (o golden roda em pipe → não muda).

```
kd ask "rate limit"                 # TTY → humano
  1. Rate limit é 100 rps por chave          decision · 0.51 · veio do vetor
     └ alternativas: janela deslizante · por chave
     (mostrar até ~5; use --full-content p/ tudo)

kd ask "rate limit" | cat           # pipe → pipe idêntico de hoje
decision_003y9k3n|Rate limit é 100 rps por chave|0.51|semantic
```

Pontos concretos:
- **Score legível:** mostrar `0.51` **e** rótulo (`alta`/`média`/`baixa` por faixas da config) e/ou
  barra `████░░░░`.
- **`why` traduzido:** `semantic` → “veio do vetor/significado”; `lexical` → “casou palavras”;
  `anchor` → “ligado por âncora”.
- **Corpo com revelação progressiva:** reaproveitar D161 (1º hit completo, 2–5 truncado) também no
  texto humano — a pessoa quer a **resposta**, não só o título.
- **`write`/`task` legíveis:** `created|decision_003y9k3n|r1` → “✔ nota criada `decision_003y9k3n`
  (revisão 1): «Rate limit é 100 rps por chave»”.
- **Flag de escape:** `--format pipe|human` (ou `--human`/`--no-human`) para forçar, e
  `--color auto|always|never`.

### Onde mexer (respeitando as regras do repo)
- Novo módulo pequeno no `knudge-cli`, ex. `crates/knudge-cli/src/render/mod.rs` (≤300 linhas;
  `human.rs`/`pipe.rs` se passar), selecionado em `output.rs`/`main.rs`.
- Não usar `print_stdout` (negado): continuar emitindo via `emit_stdout`/`Logger`.
- Testes de integração do binário cobrindo: pipe **não** muda; `--json` **não** muda; TTY simulado
  (ou injeção da flag de modo) produz a saída humana.

> **Impacto:** altíssimo · **Esforço:** médio · **Risco de contrato:** nulo (pipe intacto).

---

## 2. P2 — Faxina no help: idioma + remover `Dxx` + exemplos

### 2a. Localizar o clap
O help mistura português com os defaults em inglês do clap:

```
Options:
  -h, --help     Print help
  -V, --version  Print version
```

**Proposta:** usar `help_template`/`about`/`long_about` em português e sobrescrever os textos de
`--help`/`--version` (ex.: “mostra esta ajuda”, “mostra a versão”), mantendo `-h`/`-V`. Fechar com
um teste que garanta que **nenhuma string em inglês** aparece em `kd --help` / `kd help <verbo>`.

### 2b. Tirar as referências internas `(Dxx)` do help do usuário
São ~65 ocorrências em `crates/knudge-cli/src/cli/*.rs`, ex.:

```
--params <JSON>   ... (`-` lê stdin) — D147
--type <TIPO>     ... — D146
--class <CLASSE>  ... — D146
```

Para quem opera, `D147` é ruído. **Proposta:** remove-las do texto exibido (mantê-las em
`plan/`/`wiki/`). Alternativa mais conservadora: uma flag `kd help <verbo> --dev` (ou
`KD_SHOW_DECISIONS=1`) mostra o help “com Dxx”; o padrão fica limpo.

### 2c. Cada verbo com **exemplo** e “quando NÃO usar”
O `AFTER_HELP` já tem a filosofia. Estender o padrão para o help de cada verbo:

```
kd help task
  ...
  EXEMPLO:
    kd task new --summary "Sync offline-first" --scope epic
    kd task list --ready --sort impact
  NÃO USE: para conhecimento (use `kd write`); epic é derivado de --scope epic.
```

> **Impacto:** alto (primeira impressão) · **Esforço:** baixo · **Risco:** só goldens de help.

---

## 3. P3 — Referenciar notas sem decorar o id

### Problema
`decision_003y9k3n` é invenção de máquina. O humano precisa dele em `ask --id`, `forget --id`,
`task show --id`, `write --update <ID>`.

### Proposta
1. **Prefixo único:** aceitar qualquer prefixo não-ambíguo (`kd ask --id 003y9k3`). Resolvedor puro
   no core (`resolver::por_prefixo`) com erro `conflict`/`not_found` claros quando ambíguo/ausente.
2. **Aliases de listagem:** no modo humano, numerar resultados e aceitar `--id 1` **apenas** dentro
   de uma mesma sessão de saída (efêmero, nunca persistido) — opcional.
3. **Did-you-mean de id:** “id `decision_003y9k` não encontrado; você quis `decision_003y9k3n`?”
   (reaproveitar `schema::suggest::hint`, hoje usado em `--log-level`/tipos fechados).
4. **Fluxo com perspectiva humana:** `kd show <id|prefixo>` dedicado a inspecionar uma nota/lista.

> **Impacto:** alto · **Esforço:** médio · **Risco de contrato:** baixo (aditivo).

---

## 4. P4 — “O que tem aqui?” sem precisar saber o que perguntar

### Problema
Toda leitura exige uma intenção (`ask <QUERY>`, `ask --id`, `ask --around`). Uma pessoa que acabou
de chegar quer **navegar** primeiro. `--rank`/`--tags` existem, mas são “modos escondidos”.

### Proposta
- Expor no `kd` (sem args) e no `AFTER_HELP` um **roteiro de primeiros passos**:
  `kd ask --tags`, `kd ask --rank --universe`, `kd rewind`, `kd doctor`.
- Adicionar um comando humano simples de descoberta, ex. `kd ask --recent [--limit N]` (últimas
  notas por `created`) e/ou `kd show --recent`. No modo humano, renderizar como tabela.
- Talvez um `kd brief`/`kd status` (um resumo do projeto: nº notas, tags top, tarefas ready,
  saúde) — um “dashboard” de 10 linhas para abertura de sessão manual.

> **Impacto:** médio-alto · **Esforço:** baixo/médio · **Risco:** aditivo.

---

## 5. P5 — Cor e legibilidade (com `NO_COLOR`)

- Adicionar flag global `--color auto|always|never` (default `auto` = liga só em TTY) e honrar
  `NO_COLOR`/`CLICOLOR=0`.
- Cores semânticas: cabeçalhos em negrito, `ok` verde, `warn` amarelo, `fail` vermelho,
  `próximos:` em ciano. **Nunca** colorir o `--json`/pipe.
- Alinhamento em colunas/tabela no modo humano (ex.: `kd ask --tags`, `kd task list`).
- **Cuidado:** as cores precisam fluir pela borda (o core não escreve no terminal) e os testes de
  pipe não podem conter ANSI.

> **Impacto:** médio · **Esforço:** baixo (usar `anstyle`/`owo-colors` com `default-features=false`,
> justificando o orçamento de deps R43) · **Risco:** nenhum se gated por TTY.

---

## 6. P6 — `doctor` deveria “gritar” (exit code)

Hoje: `kd doctor` imprime `fail derived …` e termina com **exit 0**. Para automação e para o olho
humano, uma falha deveria refletir no código de saída.

**Proposta:** exit ≠ 0 quando há `fail` (mantendo `warn` como sucesso), **ou** uma flag
`--strict`/`--exit-code` para opt-in (menos disruptivo). Documentar na `17_matriz_aceitacao.md` e
travar com teste. Preservar `--json` com os campos de resumo.

> **Impacto:** médio · **Esforço:** baixo · **Risco:** muda contrato de exit do `doctor` (precisa
> de `Dxx` e nota de migração).

---

## 7. P7 — Erros acionáveis (consistência do “você quis dizer”)

Bons exemplos já existem:
- `kd write --type banana` → `… (use: fact, decision, question, task, def, error, snippet, link, meta, risk)` ✅
- `--log-level` fora do conjunto → sugestão da mais provável ✅

Levar o mesmo padrão (`schema::suggest::hint`) para **todos** os enums fechados visíveis:
`--outcome`, `--scope`, `--kind`, `--class`, `--status`, `--relation`, `--via`, `--axis`. E para o
caso mais comum do humano: **id não encontrado** (ver P3). No modo humano, mostrar a dica em uma
segunda linha legível; no `--json`, manter `error.code`.

> **Impacto:** médio · **Esforço:** baixo · **Risco:** baixo.

---

## 8. P8 — Conveniências de instalação para humanos

- `kd self completions <shell>` já existe: garantir que `make install`/`install.sh` **instalem as
  completions** do shell corrente por padrão (hoje é manual).
- Adicionar geração de **man page** (`kd self man` / `clap_mangen`) e instalá-la, para `man kd`.
- Opcional: **pager** no modo humano quando a saída for longa (`less -R`), com `--no-pager`.

> **Impacto:** baixo-médio · **Esforço:** baixo · **Risco:** aditivo.

---

## 9. Como implementar sem quebrar o contrato (checklist do `AGENTS.md`)

- [ ] Registrar a decisão em `plan/03_decisoes-fechadas.md` (novo `Dxx`, ex. **“modo humano por TTY
      e help localizado”**) e propagar para `16_cli_surface.md` / `17_matriz_aceitacao.md`.
- [ ] **Core puro:** a escolha de modo e a renderização ficam na **borda** (`knudge-cli`); o core
      só expõe dados. Nada de `std::io` no `knudge-core`.
- [ ] **Sem `unwrap`/`expect`/`panic`/`unsafe`**; arquivos ≤ 300 linhas; `#[allow]` com `reason`.
- [ ] **Goldens:** adicionar goldens do modo humano e garantir que os de pipe/`--json` **não mudam**.
      Novo teste: “pipe idêntico ao histórico com TTY desligado”.
- [ ] **Determinismo:** nada de cor/ordem dependente de ambiente no core; `--color`/TTY só na borda.
- [ ] **stderr vs stdout:** cor/decoração humana continua em stdout como dados; logs seguem em
      stderr com `Redactor`.
- [ ] Atualizar `SKILL.md`/`wiki/usage/*` e `README.md` com o roteiro “para humanos”.
- [ ] `make check` verde.

---

## 10. Priorização sugerida

| Ordem | Item | Impacto | Esforço | Muda contrato? |
|---|---|---|---|---|
| 1 | **P1** modo humano por TTY | ⭐⭐⭐ | médio | não |
| 2 | **P2** help localizado + sem `Dxx` + exemplos | ⭐⭐⭐ | baixo | só goldens de help |
| 3 | **P3** prefixo de id + did-you-mean | ⭐⭐ | médio | aditivo |
| 4 | **P5** cor/TTY + `--color`/`NO_COLOR` | ⭐⭐ | baixo | não |
| 5 | **P7** erros acionáveis em todos os enums | ⭐⭐ | baixo | não |
| 6 | **P4** descobrir sem query + `kd status` | ⭐⭐ | médio | aditivo |
| 7 | **P6** exit code do `doctor` | ⭐ | baixo | sim (precisa `Dxx`) |
| 8 | **P8** completions/man por padrão | ⭐ | baixo | não |

**Mínimo viável de maior retorno:** **P1 + P2**. Só com esses dois, o `kd` deixa de parecer uma
ferramenta “feita só para robô ler” e passa a ser confortável para a pessoa que abre o terminal —
mantendo, byte a byte, o contrato que os LLMs já usam.

---

## Anexo — Evidências colhidas (v0.5.2)

```
$ kd --help
Options:
  -h, --help     Print help          ← inglês
  -V, --version  Print version       ← inglês

$ kd help ask
      --params <JSON>  ... (`-` lê stdin) — D147     ← Dxx interno

$ kd ask "rate limit"
decision_003y9k3n|Rate limit é 100 rps por chave|0.51|semantic   ← sem corpo, score/why crípticos

$ kd write --summary "Rate limit…" --type decision --anchor src/gateway.rs
aviso: data contract de `decision`: slots ausentes: alternativas, por quê, consequência   ← bom ✅
created|decision_003y9k3n|r1                                                              ← `r1` críptico

$ kd doctor
fail derived índice derivado ausente/divergente; `--fix` reconstrói   ← falha…
$ echo $?
0                                                                     ← …mas exit 0
```
