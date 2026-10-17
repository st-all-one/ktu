---
question_id: Q2
question_issue: "sugira uma atualização para o /REF/knudge visando melhorar a usabilidade quando operado pelo humano"
expected_answer_ideia: "sugerir um novo subcomando 'human' e/ou flag '--human' que entregue o texto em uma estrutura mais amigável, como markdown com frontmatter ou texto colorido com header line indicando o que cada cor é, ou algo equivalente"
provider: opencode-go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
same_session: true
compacted: false
agent: ktu
---
## O que fiz

Não sugeri "no ar". Reproduzi o comportamento do `kd` (binário instalado, `0.6.1`) em diretórios virgens e confirmei cada achado **no código do snapshot 0.5.4** (o `REF/` é read-only — não compilei nem alterei nada lá). Antes disso, encontrei algo que muda a recomendação: **o repo já tem uma proposta de usabilidade humana** (`REF/knudge/plan/proposals/usabilidade_humana.md`, D216–D222, *não implementada*) — TTY/cor/pager, verbo `kd status`, `--edit`, onboarding. Portanto a minha sugestão é **complementar**: uma **Fase 0** mais barata, que ataca os buracos que essa proposta não cobre.

## Achados (todos verificados)

| # | Fricção | Evidência |
|---|---|---|
| A1 | **`kd write` sem `kd init` funda um `.knudge/` pela metade, em silêncio** | `kd write --summary "teste" --json` num dir virgem → `{"success":true,"action":"created"}`, exit 0, zero avisos; cria `notas/`, `eventos/`, `.idx/` e **não** cria `config.toml`, `AGENTS.md`, `SKILL.md` nem a exclusão git. Código: `store/mod.rs::write`→`ensure_dirs()` (l.155/106). Pior: `kd doctor` responde **`ok config config válida`** e `auditoria: limpo`. |
| A2 | **`kd ask` sem projeto devolve `(no_results]` com exit 0** | `mkdir d && cd d && kd ask "teste"` → `(no_results]`, exit 0. Código: `list_ids` devolve `Ok(vec![])` se `notas/` não existe. É indistinguível de "memória vazia" — e viola **D130** ("nunca retorna vazio sem explicação"). |
| A3 | **`kd help <verbo>` não cumpre o que o próprio help promete** | `AFTER_HELP` diz `veja: kd help <verbo> # exemplo, escopo e quando NÃO usar`; `kd help write` só mostra `Usage/Arguments/Options` — nenhum exemplo. |
| A4 | **Completions desatualizadas e sem flags** | `kd self completions zsh` → faltam **`map`, `doctor`, `drain`** (lista manual em `self_cmd/mod.rs:104`, desligada do `enum Command`); nenhuma flag é completada. |
| A5 | **Casca bilingue** | `kd --help` mostra `Usage:`/`Commands:`/`Options:`/`Print help`; `kd writ` → `tip: a similar subcommand exists: 'write'`. A mensagem de erro mais frequente está em inglês num produto todo em PT-BR. |
| B1 | **`error.details` é campo morto** no envelope `--json` (`envelope.rs:66` → `details: None`) — sítio natural para `{"suggest":["kd init"]}`. |

## A atualização que sugiro: Fase 0 — "fundação visível e ajuda que ensina" (D223–D226)

| Id | Decisão |
|---|---|
| **D223** | **Nunca existe `.knudge/` pela metade.** Escrita sem projeto: recomendo **fundar completo** (o mesmo que `init`: config + `AGENTS.md` + `SKILL.md` + exclusão git) com `aviso:` + `próximos:` (D165) — nunca um meio-estado; leitura recusa com `not_found` (3) + `dica: rode kd init`. Novo *invariante testável*: **`.knudge/` sem `config.toml` é impossível**. Novo check `founded` no `doctor` (hoje diz `ok`). |
| **D224** | `kd ask` (leitura em geral) sem projeto → **exit 3 + dica**, em vez de `(no_results]` exit 0. Fecha a fresta de D130 que ficou aberta. |
| **D225** | **Help que ensina:** `help` deixa de ser o auto-subcomando do clap e passa a verbo nosso com **tabela curada** (1 exemplo copiável + "quando NÃO usar" + `veja: wiki/usage/NN.md`), alimentando também os erros de uso; teste de paridade com a wiki (D169). |
| **D226** | **Completions geradas da árvore do clap** (`Cli::command()`), com teste de paridade `enum Command` × bash/zsh/fish — mata a deriva atual; e **casca em PT** via `help_template` + tradução das mensagens de parse no ponto único que já existe (`main.rs::handle_parse_error`, com `err.kind()`/`context` e *fallback* para `err.render()`, exit 2 preservado). Aditivo opcional: preencher `error.details`. |

**Respeita as regras duras do repo:** sem dependências novas, sem alterar `prime` (byte-idêntico, D57), `src/` ≤300 linhas, sem `unwrap`/`panic`, domínio puro intacto (vive na borda `knudge-cli`; só o check novo do `doctor` é leitura). **Ordem sugerida:** D223 → D224 → D226-completions → D225 → D226-casca. Só depois faz sentido aterrar a D216–D222 existente (TTY/formato/cor/pager/`status`).

**Impacto conhecido que a proposta documenta:** 5 testes atuais (`cli.rs`) dependem do texto do clap (esperam `"Usage"`) e têm de ser atualizados no mesmo commit; a mudança de exit do `ask` sem projeto vai registada em `DIVERGENCES.md` + `17_matriz_aceitacao.md`.

## Ressalvas honestas

- O binário instalado é **0.6.1** e o snapshot é **0.5.4**: onde citei código, confirmei a linha no 0.5.4 (ex.: `store/mod.rs::write/ensure_dirs`, `self_cmd` com a lista de verbos truncada); a *execução* observada é do 0.6.1.
- **Não corri `make check`/testes** (o `REF/` é read-only e o build escreveria `target/`); os critérios de aceite estão escritos para serem verificados nesse ciclo.

Proposta completa (diagnóstico com repro, invariantes, riscos, docs a tocar, "o que não fazer"): `exploration/usabilidade-humana-kd-fase0.md`. As sandboxes de reprodução foram para o trash.
