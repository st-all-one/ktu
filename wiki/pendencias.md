# Dossiê de pendências

> **Levantamento verificado no código** (não na memória): o que **falta**, porquê, e o que o
> reabre. É o espelho do `STAGING.md` — que diz o que está feito; este diz o que não está e
> **o que foi rejeitado com que número**. Antídoto do "fizemos tudo".
>
> **Estados:** ⛔ Deferido · 🚫 Rejeitado · ⏳ Opt-in · 🔴 Lacuna · ⚪ Formalismo
> **Eixos:** **Q** qualidade · **P** performance · **S** simplificação
> **Regra de poda** (M1, [revisão de propósito](revisao-proposito.md)): um item só fica aqui com a **âncora** no
> código e o **teste/sintoma que o reabre**. Sem isso, sai.

## 0. Como ler

- **🔴 Lacuna** — algo que o ktu *promete* e não cumpre (verificado no código, hoje).
- **⛔ Deferido** — decidido não fazer agora, com critério de reabertura (YAGNI explícito).
- **🚫 Rejeitado** — avaliado e recusado; fica o **número** (ou `unpriced`, com a razão).
- **⏳ Opt-in** — existe, mas desligado por omissão (custo/risco fora do caminho comum).
- **⚪ Formalismo** — falta de travão num artefacto (documentação, gate), não de funcionalidade.

## 1. Lacunas (🔴) — verificadas no código

| # | Lacuna | Eixo | Âncora (verificado) | Reabre com |
|---|---|---|---|---|
| ~~L1~~ | ~~O workspace do **knudge** está excluído da raiz e `make check` não o cobre~~ — **fechado na F7** (L1: `make check-knudge` + job `knudge` na CI, com a suite corrida no G22) | Q | `Makefile`, `.github/workflows/ci.yml` | — |
| L2 | `ktu serve` **não tem autenticação**: o default é `127.0.0.1:7717`, mas `--bind 0.0.0.0:…` expõe a porta sem credenciais | Q | `crates/ktu-cli/src/cli.rs` (`Serve`), `crates/ktu-cli/src/serve.rs` | alguém expor a porta fora do `localhost` (ou um front remoto real) |
| ~~L3~~ | ~~Nenhum gate verifica links internos da wiki nem intra-doc do Rust~~ — **fechado na F7** (L3: `xtask check-links` + `cargo doc -D warnings` dentro do `make check`; gate G22 verde) | Q | `Makefile`, `xtask/src/links.rs` | — |
| L4 | O limite de 400 linhas **não** cobre `crates/*/tests/` (isentos por decisão) — um teste pode crescer sem travão | S | `scripts/check_file_length.sh` | um ficheiro de teste que deixe de ser navegável (ou um teste que esconda lógica de produção) |
| L5 | O **modelo de referência** (Ornith-1.5-9B Q4_K_M) não corre nos gates neste host: ~8 min por *completion* (CPU-only, medido na H12). Os gates G10/G11 provam a mecânica com guião determinístico e o G11 observa um modelo pequeno | Q | `wiki/gates/README.md` §4, `scripts/run-server.sh` | um host com GPU (ou um modelo local mais rápido) onde a cadeia de 15 turnos caiba no orçamento |
| ~~L6~~ | ~~O runtime **usa** os guardrails globais mas `ktu guardrails list` só mostra os do projecto — o piso é invisível~~ — **fechado na F3** (M8: `origem` global/local em `guardrails`/`commands`/`plugins` e o novo `ktu skills ls`; gate G18 verde) | Q | `crates/ktu-cli/src/guardrails_cmd.rs` | — |

> **Verificação desta secção (2026-10-18):** L1 conferido em `Cargo.toml` (`exclude = ["REF",
> "crates/knudge"]`) e na CI (job `knudge` → `make check-knudge`); L2 em `cli.rs`
> (`default_value = "127.0.0.1:7717"`, sem opção de credenciais); L3 no `Makefile` (`check` passou a
> incluir `links` e `doc`); L4 em `scripts/check_file_length.sh` (percorre `src`, não `tests`);
> L5 medido na H12 (uma completion do Ornith 9B > 8 min neste host); L6 conferido em
> `guardrails_cmd.rs` (lê só `.agents/guardrails`) contra `facade/builder.rs` (carrega os dois).

## 2. Deferidos (⛔) — com critério de reabertura

**Plano de execução:** [`implementation_phase3/`](implementation_phase3/README.md)
(`epic_00j1cgo3`). Correspondência: **D1→F3**, **D6→F4**, **D7→F1**, **D5→F2** (excepto a
*Tab*) e **D2→F5** estão **fechados** (gates G16–G20 e G22 verdes) e **D3→F6** foi **fechado sem
plano** (o `ktu init` já cobre o caso comum; a pendência fica nesta tabela com o gatilho afinado).
Ficam
**fora** da Fase 3: **D4** (é um caminho, não trabalho — [ADIADOS §2](implementation_phase2/ADIADOS.md))
e **D8–D12** (sem consumidor). O épico antigo (`epic_00ey9lmk`) foi absorvido. **D2** (migrações
de config) foi feita **antes do gatilho** — o mecanismo de versionamento é pré-condição de
qualquer migração futura; desvio registado na
[ADR 0020](adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md).

| # | Item | Eixo | Âncora | Reabre com |
|---|---|---|---|---|
| ~~D1~~ | ~~Recursos globais (**skills/plugins/receitas** em `~/.config/ktu/`)~~ — **fechado na F3** (gate G18 verde): dois níveis, precedência declarada e `origem` nas listagens (M8) | S | `crates/ktu-core/src/skill.rs` | — |
| ~~D2~~ | ~~Migrações de configuração (versão + migração de `.agents/config.toml`)~~ — **fechado na F5** (gate G20 verde): `schema = 1`, `read_migrate` não destrutiva, `migrate` idempotente, fail-closed em versão futura e `ktu config doctor` — **sem o gatilho** (ver [ADR 0020](adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md)) | Q | `crates/ktu-core/src/config/migrate.rs` | — |
| D3 | `ktu self setup <cliente>` (instalar a integração num cliente externo) — **A3**; o protocolo vive em `ktu prime`/base do prompt (o `ktu init` **não** escreve no `AGENTS.md`, [ADR 0021](adr/0021-o-init-nao-escreve-no-agents-md.md)) e o ktu não tem MCP | S | `crates/ktu-cli/src/init.rs`, [`F6`](implementation_phase3/F6-self-setup.md) | um cliente **concreto** que precise de configuração própria (ou um pedido explícito de *setup*); se for MCP, decidir primeiro o MCP |
| D4 | `.ktu/logs/` (log de diagnóstico estruturado por sessão) — **A4**, **fora da Fase 3** | Q | `crates/ktu-cli/src/logging.rs` | um incidente que precise de log de diagnóstico (o `doctor --bundle` é o embrião) |
| D5 | REPL: ~~`Ctrl-C`~~ (feito) e ~~`/compact`, `/fork`, `/memory`, `/resume`~~ (feito) — falta a **`Tab`** (*raw mode* + gestão do terminal em volta dos logs do turno) — **A5**/**F2**, gate G17 verde | Q | `crates/ktu-cli/src/repl.rs` | utilização interactiva real a pedir o atalho |
| ~~D6~~ | ~~H5: fila de mutação, remédio por regra, âncoras próximas~~ — **fechado na F4** (gate G19 verde): `FileMutationQueue` por caminho canónico, remédio da regra + estrutural, sugestão das âncoras próximas | Q | `crates/ktu-tools/src/mutation.rs` | — |
| D7 | ~~*Steering* durável na admissão + idempotência por ID~~ — **A7/F1, fechado** (G16 verde) | Q | `crates/ktu-core/src/agent/steering.rs` | *(disparou; ver [G16](gates/G16-steering.md))* |
| D8 | Sandbox de SO (Landlock/seccomp) | Q | `crates/ktu-tools/src/bash.rs` | um sintoma que o confinamento de *path* não resolva |
| D9 | `--record` (gravar as completions de uma sessão real para *replay* determinístico) | S | `crates/ktu-cli/src/provider_script.rs` | uma sessão real a precisar de ser reproduzida (o guião escrito à mão chega aos gates) |
| D10 | `sub_recipes` (receitas que invocam receitas) | S | `crates/ktu-core/src/command.rs` | uma receita real que precise de compor outra |
| D11 | `parameters` declarados (nome/omissão/tipo) nas receitas | S | `crates/ktu-core/src/command.rs` | um comando a precisar de validação por tipo (ou ajuda gerada) |
| D12 | `model`/`hidden`/`color` nos agentes declarativos | S | `crates/ktu-core/src/agent_file.rs` | provider por agente (hoje é por sessão) ou um pedido de UI |
| D13 | **Lock de ficheiro entre processos** (a fila da F4 é do **processo**): `.ktu/locks/<caminho canónico>.lock`, com a política de obsoleto do `TurnLock` | S | `crates/ktu-tools/src/mutation.rs` | dois processos no mesmo projecto a editar o mesmo ficheiro (o `TurnLock` só trava o mesmo id de sessão) |
| D14 | **Escrita atómica** (`temp` + `Fs::rename` — a porta já tem o `rename` «para escritas duráveis»): evita ficheiro truncado se o processo morrer a meio | S | `crates/ktu-tools/src/fs.rs` | um ficheiro truncado por queda a meio da escrita (o log de sessão é reescrito inteiro a cada turno) |

## 3. Rejeitados (🚫) — com o número

| # | Item | Eixo | Base | Número / razão | Reabre com |
|---|---|---|---|---|---|
| R1 | Micro-optimização do núcleo | P | `measured` | projecção de 10 000 eventos ≈ 0,5 ms; turno de 1 passo ≈ 7 µs — o gargalo é o **modelo**; limiar declarado: projeção > 5 ms | uma medição cruzar o limiar (`wiki/benchmarks/`) |
| R2 | GDK (`goose-sdk`/`goose-providers`) | S | `measured` (crates) + `unpriced` (resto) | ~167 crates extra (com `rmcp`/MCP) sobre 267; *alpha*; rede não-determinística | necessidade real de um dialeto que o caminho OpenAI-compatible não cubra |
| R3 | MCP como fonte de tools | S | `unpriced` | na lista YAGNI do `plan.md` §0; sem consumidor | um cliente que só fale MCP |
| R4 | Motor de política genérico (regras + prioridades + *containment*) | S | `unpriced` | foi o que impediu o katu de operar; as três camadas de guardrails cobrem o caso | um sintoma que os guardrails declarativos não exprimam |
| R5 | Plugins WASM / *dispatch* paralelo / marketplace | S | `unpriced` | sem consumidor; o host de plugins é de processo. A **fila de mutação** da F4 já prepara o paralelismo de tool calls (a concorrência continua rejeitada) | um plugin real que exija isolamento ou paralelismo |
| R6 | `HashMap` no núcleo (por velocidade) | S | `unpriced` | a ordem de iteração imprevisível quebra I6; o custo não foi medido porque o determinismo decide antes | uma medição que mostre o `BTreeMap` no caminho quente (não é o caso hoje) |
| R7 | TUI completa (temas, keybindings, *splash*) | S | `unpriced` | a CLI é o front primário ([ADR 0010](adr/0010-cli-e-o-front-primario.md)); a TUI existe como cliente | uso interactivo real a pedir o que a TUI mínima não dá |

## 4. Opt-in (⏳) — existe, desligado por omissão

| # | Item | Eixo | Âncora | Nota |
|---|---|---|---|---|
| O1 | Canal semântico (embeddings locais) | P | `crates/ktu-memory/src/drain.rs` | `ktu memory drain --status/--digest/--force`; sem *worker* o índice fica vazio e o *recall* é lexical |
| O2 | `ktu memory drain service --install` (worker residente) | Q | `crates/ktu-cli/src/memory_drain.rs` | instalado por pedido; **não** arranca sozinho |
| O3 | Provider remoto (`--provider opencode-go`) | Q | `crates/ktu-provider/src/wire.rs` | só quando o host não aguenta o modelo local |
| O4 | `shell: true` nas receitas | Q | `crates/ktu-core/src/command.rs` | opt-in **por comando**; sem ele `!cmd` é texto |
| O5 | `deferred_tools` (`tool_search`) | Q | `crates/ktu-tools/src/toolbox.rs` | opt-in por `.agents/config.toml`; sem diferidas a tool não é oferecida |
| O6 | `ktu serve` / `ktu tui` (porta HTTP e cliente) | S | `crates/ktu-cli/src/serve.rs` | a CLI não precisa de servidor; ver **L2** (sem autenticação) |

## 5. Formalismo (⚪) — travões em falta

| # | Item | Eixo | Âncora | Reabre com |
|---|---|---|---|---|
| ~~F1~~ | ~~`cargo doc` fora do `make check`~~ — **fechado na F7**: alvo `doc` (`RUSTDOCFLAGS="-D warnings"`, `--no-deps`) dentro do `make check`; medido ~4 s quente / ~34 s com alvo vazio (limiar 60 s) | Q | `Makefile` | — |
| F2 | Nenhum teste de contrato do envelope `--json` contra um *golden* único | Q | `crates/ktu-cli/src/output.rs` | um consumidor a quebrar com uma mudança de envelope |
| F3 | Via de integração do `crates/knudge` (git pinada / submódulo / vendored) por decidir | S | `Cargo.toml` | decisão humana; a CI já corre o portão dele (**L1** fechado na F7), mas a via de integração continua por decidir |

## 6. Verificação e manutenção

- Este dossiê é verificado **contra o código** em cada revisão de fase: cada linha da §1 tem
  âncora e foi conferida (ver a nota de verificação no fim da secção).
- Uma pendência que deixa de ter sintoma **sai** (regra de poda); uma que se implementa passa para
  o `STAGING.md` e, se for decisão, para uma [ADR](adr/README.md).
- Rejeições com número vivem aqui (§3) e são citadas pelas ADRs e pelo
  [método de medição](metodo-medicao.md).
