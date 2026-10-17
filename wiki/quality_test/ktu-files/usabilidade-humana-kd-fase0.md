# Proposta — usabilidade humana do `kd`, Fase 0: "fundação visível e ajuda que ensina"

> Alvo: `REF/knudge` (snapshot **0.5.4**, read-only — esta nota vive fora do repo).
> Relação com o que já existe: **complementa** `REF/knudge/plan/proposals/usabilidade_humana.md`
> (D216–D222, ainda não implementada). Aquela proposta ataca a *decoração* (TTY, cor, pager,
> `kd status`, editor); esta ataca a *fundação e o ensino* — os dois buracos que sobram e que
> custam quase zero a fechar. Numeração provisória: **D223–D226**.
>
> **Método:** achados reproduzidos a correr o binário instalado (`kd 0.6.1`, `~/.local/bin/kd`) em
> diretórios virgens, e confirmados depois **no código do snapshot 0.5.4** (linhas citadas). O
> binário instalado é mais novo que o snapshot; onde cito código, o comportamento verifica-se em
> 0.5.4 *por leitura*, não por execução (o snapshot é read-only e não foi compilado).

---

## 0. Diagnóstico (fricções verificadas, com repro)

### A1 — Fundação implícita **silenciosa** (a pior): `write` cria um `.knudge/` pela metade

```bash
mkdir dir && cd dir
kd write --summary "teste" --json
# {"success":true,"command":"write","data":{"action":"created","id":"fact_01jl4ex9","revision":1}}
# exit 0, sem aviso nenhum (nem em `warnings`)
ls .knudge          # notas/ eventos/ .idx/  … e MAIS NADA
ls .knudge/config.toml   # No such file or directory
```

Falta, em relação ao `kd init`: `config.toml`, `AGENTS.md`, `.agents/skill/kd/SKILL.md` e a
exclusão do git em `.git/info/exclude` (D34; `crates/knudge-core/src/git/exclude.rs`). O `init`,
esse, cria tudo (`kd init` num dir virgem lista `config.toml (novo)`, `AGENTS.md (novo)`, …).

**Código (0.5.4):** `crates/knudge-core/src/store/mod.rs` — `Store::write` chama `ensure_dirs()`
(l.155 → l.106), que só faz `notas/` + uma pasta por tipo; `crates/knudge-core/src/store/events/log.rs:65`
cria `eventos/` por conta própria. Nada, no caminho de escrita, conhece o `init`.

**Agravante:** o `doctor` não vê o problema. Num projeto meio-fundado:

```
ok config config válida
…
auditoria: limpo
```

Ou seja: **não existe check de "projeto fundado"**, e o auto-diagnóstico declara saúde total a um
projeto sem `config.toml` nem `AGENTS.md`.

**Porque dói a um humano:** (1) o agente dele nunca recebe `AGENTS.md`/`SKILL.md` — o setup que o
`init` existe para entregar; (2) `.idx/` (derivado) fica sem proteção, e um `git add .` comita
cache; (3) o humano nunca vê o `próximos:` do `init` (D165); (4) `kd config set` passa a criar o
`config.toml` a posteriori, de forma opaca.

**Base normativa violada (em espírito):** **D130** — "verbos falham alto, nunca em silêncio";
**D165** — `init`/`config`/`sync` **listam o que fizeram** e terminam em `próximos:`.

### A2 — `kd ask` num diretório sem projeto devolve `(no_results]` com exit 0

```bash
mkdir dir && cd dir && kd ask "teste"
# (no_results]
# exit 0   — nada é criado
```

**Código (0.5.4):** `Store::list_ids` devolve `Ok(Vec::new())` quando `notas/` não existe
(`store/mod.rs` l.178-181) → o recall recebe corpus vazio → "sem resultados".

**Porque dói:** é indistinguível de "a memória está vazia". O humano que ainda não fundou o
projeto conclui que o `kd` "não sabe nada", em vez de "aqui não há projeto". Isto é literalmente o
que **D130** proíbe: *"Nenhuma entrada vazia grava lixo nem retorna vazio sem explicação"*.

### A3 — `kd help <verbo>` não entrega o que o próprio help promete

O rodapé do help promete (`crates/knudge-cli/src/cli/help.rs`, `AFTER_HELP`):

```
veja: kd help <verbo>   # exemplo, escopo e quando NÃO usar
```

O que sai de `kd help write` (executado): `Usage: kd write [OPTIONS] [BODY]...`, `Arguments:`,
`Options:` com os doc-comments das flags — **nenhum exemplo, nenhum "quando NÃO usar"**. Só a raiz
e `drain service` têm `after_help`; `subcommand_help()` limita-se a `sub.render_long_help()`.

**Porque dói:** o humano que segue a instrução do próprio `kd` sai do terminal para o
`wiki/usage/06_write.md` à procura do exemplo.

### A4 — Completions desatualizadas (faltam 3 verbos) e sem flags

```bash
kd self completions zsh
# compadd -- init prime rewind ask write task maintenance config forget sync self
```

Faltam **`map`, `doctor`, `drain`** (e `help`). Fonte: lista manual em
`crates/knudge-cli/src/commands/self_cmd/mod.rs:104` — nada a liga ao `enum Command`
(`crates/knudge-cli/src/cli/mod.rs`). Nenhuma flag é completada em shell nenhum.

**Porque dói:** `kd do<TAB>` (o gesto humano mais natural depois de `init`) não completa.

### A5 — Casca bilingue: o casco é PT, o `clap` responde em inglês

```bash
kd --help
# knudge — memória otimizada para LLM
# Usage: kd [OPTIONS] [COMMAND]
# Commands:  …  Options:  …  -h, --help  Print help
kd writ
# error: unrecognized subcommand 'writ'
#   tip: a similar subcommand exists: 'write'
# Usage: kd [OPTIONS] [COMMAND]
# For more information, try '--help'.
```

Todo o produto é PT-BR; a moldura e — pior — **a mensagem de erro mais frequente (typo no verbo)**
estão em inglês. Nota: a sugestão de verbo **existe** e funciona (`writ` → `write`, exit 2), e o
domínio já tem `schema::suggest::hint` (D212, Levenshtein determinística) — falta só a língua.

### B1 (bónus) — `error.details` é um campo morto no envelope `--json`

`crates/knudge-cli/src/envelope.rs:36` declara `details: Option<Value>`; a l.66 preenche
**sempre** `details: None`. É o lugar natural para o "próximo passo" legível por máquina
(`{"suggest": ["kd init"]}`) — hoje a mensagem de erro é texto puro. A proposta existente (D221)
limita-se ao texto humano; vale reabrir, porque é aditivo e serve humano **e** agente.

---

## 1. Invariantes (o que esta proposta não toca)

1. `prime` permanece **byte-idêntico** (D57) — é o protocolo da IA, não a ajuda do humano.
2. Envelope `--json`, `ErrorKind`, exit codes e o pipe `id|statement|score|why` mantêm-se o
   contrato de máquina (R20/D71); o que muda é **aditivo** (`details`) ou texto humano.
3. **Sem dependências novas** (`clap_complete`, crates de cor/tabela estão fora — política do repo).
4. Domínio puro intacto: tudo isto vive na **borda** (`knudge-cli`), salvo o check novo do
   `doctor`, que é uma *projeção* de leitura (como os 13 checks atuais).
5. `src/` ≤ 300 linhas, sem `unwrap`/`expect`/`panic`/`unsafe`, iteração determinística.

---

## 2. Decisões propostas

| Id | Decisão |
|---|---|
| **D223** | **Fundação explícita: nunca existe `.knudge/` pela metade.** Verbo de escrita num diretório sem projeto (a) **recusa** com `not_found` (3) + `dica: rode \`kd init\``, ou (b) **funda o projeto completo** (o mesmo que `kd init`) e di-lo alto (`aviso:` + bloco `próximos:`, D165). Recomendação: **(b)** para escrita (mantém o `write` a funcionar, zero atrito para agentes) e **(a)** para leitura. Invariante testável: *nunca existe `.knudge/` sem `config.toml`*. Novo check `founded` no `doctor` (`fail` quando falta `config.toml`/`AGENTS.md`). |
| **D224** | **`kd ask` (e restantes verbos de leitura) num diretório sem projeto → `not_found` (3)**, com `dica:` e `details.suggest`, em vez de `(no_results]` (0). Fecha a fresta que D130 deixou aberta. |
| **D225** | **Ajuda por verbo que ensina.** `help` deixa de ser o auto-subcomando do `clap` (`disable_help_subcommand(true)`) e passa a ser **verbo nosso** (`kd help [verbo]`), cujo texto é uma **tabela curada em `cli/help.rs`**: 1 exemplo copiável, "quando NÃO usar", ponteiro `veja: wiki/usage/NN_<verbo>.md`. A tabela é também a fonte usada por `subcommand_help()` nos erros de uso. Teste de paridade com a wiki (D169): cada verbo tem verbete e a página citada existe. |
| **D226** | **Completions derivadas da árvore do `clap`** (`Cli::command()` → `get_subcommands()`/`get_arguments()`; valores dos conjuntos fechados por D212), substituindo a lista manual; teste que exige paridade `enum Command` × scripts (bash/zsh/fish). E **casca em PT**: `help_template` com cabeçalhos PT e tradução das mensagens de `clap` no ponto único que já existe — `handle_parse_error` (`main.rs`), mapeando `err.kind()`/`err.context()` para PT, com `schema::suggest::hint` na dica de typo; **fallback** para `err.render()` em qualquer caso não mapeado (nunca `panic`, exit 2 mantido). Opcional aditivo: preencher `error.details = {"suggest": [...]}` (B1). |

### Detalhe de implementação (o que olhei)

- **D223/D224** — o ponto de decisão é a abertura do `Session` (`crates/knudge-cli/src/session.rs`):
  um único `fn require_founded()` chamado no arranque do verbo (com `init`/`help`/`self`/`prime`
  isentos) evita repetir a verificação em 14 sítios. O `doctor` ganha o 14.º check no
  `commands/doctor/` (leitura apenas; `--fix` pode criar o `config.toml` por defeito **com aviso**).
- **D225** — o `help` explícito precisa de um `HelpArgs { verb: Option<String> }` e da tabela
  `VERB_HELP: &[(&str, &str, &str)]` (verbo, exemplo, "não use para"); `subcommand_help()` passa a
  ler dela. Impacto em testes: 5 asserções atuais dependem do texto do `clap`
  (`cli.rs`: `help_exits_zero_and_mentions_usage`, `no_args_shows_help`,
  `verb_without_args_shows_help`, `help_subcommand_lists_verbs`,
  `drain_without_flags_shows_help_and_does_nothing` — todas esperam `"Usage"`), a atualizar no
  mesmo commit.
- **D226** — `clap` fixado em **4.6.7** (`Cargo.lock`), features default (inclui `suggestions`,
  `color`, `help`, `usage`, `error-context`). Consequência a documentar: o *help* e os erros de
  parse já saem **coloridos** num TTY (via `clap`), ao passo que os resultados do `kd` nunca são —
  assimetria que a proposta D216/D218 existente resolverá; aqui só se alinha a **língua**.
  `handle_parse_error` já trata `DisplayHelpOnMissingArgumentOrSubcommand` → stdout/exit 0; os
  restantes caminhos são o ponto de tradução.

---

## 3. Critérios de aceite

| # | Critério (verificável) |
|---|---|
| 1 | `cd dir-virgem && kd write --summary "x"` **nunca** deixa `.knudge/` sem `config.toml` (teste de invariante + golden). |
| 2 | `cd dir-virgem && kd ask "x"` → exit **3**, `error.code=not_found`, dica `kd init` (golden `.txt` + `.json` novos); `--json` idêntico em forma aos erros atuais. |
| 3 | `kd doctor` sobre `.knudge/` sem `config.toml` → check `founded` em `fail` (hoje: `ok`). |
| 4 | `kd help write` contém **um exemplo** e uma linha "não use para" e cita `wiki/usage/06_write.md`; vale para os 14 verbos. |
| 5 | `kd self completions zsh\|bash\|fish` lista **todos** os verbos do `enum Command` (teste de paridade com `Command::name`). |
| 6 | `kd --help` e `kd writ` sem uma única palavra em inglês; exit 2 preservado; `kd --json ask` inalterado byte a byte. |
| 7 | `prime` byte-idêntico (golden `prime.txt`/`prime_long.txt` intactos). |

## 4. Riscos

| Risco | Mitigação |
|---|---|
| Quebrar agentes que hoje escrevem sem `init` | Preferir (b) em D223: a escrita passa a fundar **completo** e a avisar; nunca falha. |
| Fundar `AGENTS.md` sem consentimento no repo de alguém | O `aviso:` é explícito e `--no-init` (ou `KD_NO_BOOTSTRAP=1`) desliga a fundação implícita. |
| Golden/`llm` a mudar | Mudanças de texto só no modo humano; `prime` e `--json` congelados; mudanças de exit code (D224) vão para `DIVERGENCES.md`. |
| Tradução do `clap` a partir-se com atualização minor | Traduzir por `err.kind()`/`err.context()` com **fallback**; teste por kind, não por string. |
| `help` próprio divergir do `clap` | Tabela única + teste de paridade (verbo ↔ tabela ↔ wiki). |

## 5. Documentação a tocar (D169: docs vivas e estáticas em sincronia)

`README.md` (quickstart: `init` primeiro, agora **garantido**), `wiki/usage/03_init.md`,
`06_write.md`, `05_ask.md`, `10_doctor.md`, `13_config.md`, `19_troubleshooting.md` (novo sintoma
"dei `write` antes do `init`"), `wiki/specs/cli.md` + `saude.md` (check `founded`),
`plan/implementation/16_cli_surface.md`, `17_matriz_aceitacao.md` (linhas de `ask`/`write`/`doctor`),
`wiki/specs/DIVERGENCES.md` (exit novo de `ask` sem projeto) e `CHANGELOG.md`.

## 6. Ordem sugerida

1. **D223 + check `founded`** — tira o pior *footgun* e não muda nenhuma superfície.
2. **D224** — fecha D130 no caminho de leitura (depende de D223 para a mensagem ser a mesma).
3. **D226 completions** — *bug* de paridade, isolado e barato.
4. **D225 help** — mais trabalho de escrita; alimenta-se da wiki.
5. **D226 casca PT** — cosmético, mas é a moldura que o humano vê primeiro.

Depois disto, a proposta D216–D222 existente (formato/TTY, `kd status`, cor, pager, editor)
assenta em terreno firme: há sempre um projeto fundado e uma ajuda que responde no terminal.

## 7. O que **não** fazer

- Não transformar a fundação implícita em **modo silencioso** outra vez (era o estado anterior).
- Não pôr lógica de fundação no `knudge-core` (porta `Fs`, borda decide).
- Não trazer `clap_complete`/crate de cor/TUI (a já citada "infra que o caso não paga").
- Não tocar em `prime`, no envelope de sucesso nem nos exit codes existentes (só **acrescentar** o
  de `ask` sem projeto, com registo em `DIVERGENCES.md`).

---

### Apêndice — como reproduce (com o binário instalado)

```bash
mkdir -p /tmp/kd-repro && cd /tmp/kd-repro
kd write --summary "teste" --json && find .knudge -maxdepth 3 | sort && ls .knudge/config.toml
kd doctor
```

```bash
mkdir -p /tmp/kd-repro2 && cd /tmp/kd-repro2
kd ask "teste"; echo "exit=$?"        # (no_results] / exit 0
kd help write | head -40              # sem exemplo
kd self completions zsh               # sem map/doctor/drain
kd writ                               # tip em inglês
```

Sandboxes usadas nesta sessão: `exploration/h1`, `exploration/h3`, `exploration/h4`,
`exploration/sandbox-kd` (mantidas só durante a investigação).
