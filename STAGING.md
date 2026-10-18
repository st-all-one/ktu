# STAGING — Estado do épico ktu (revisão 2026-10-18)

> Documento de staging: fotografia do estado após o fecho do **Gate G4** e, na revisão de
> 2026-10-18, da **Fase 2** (H0–H12, gates G10–G15 verdes — ver [`wiki/gates/`](wiki/gates/README.md)).
> Fontes: `epic.md`, `plan.md`, `wiki/quality_test/ktu-veredicto.md` e a memória `kd`.
> **O inverso deste documento** (o que falta, porquê, e o que reabre cada item) é o
> [dossiê de pendências](wiki/pendencias.md). As decisões com alternativas vivem em
> [`wiki/adr/`](wiki/adr/README.md).
> Portão único: `make check` (`fmt` + `clippy -D warnings` + `test` + `check-layers`
> + `check-adr` + `check-surface` + `file-length`). Verificado **verde** nesta revisão, com
> `make gates` (G10–G15) e `make supply-chain` também verdes.

## 0. Fase 2 — fechada ✅ (2026-10-18)

| Issue | Entrega | Estado |
|---|---|---|
| H0–H6 | superfície CLI, `init`, config, efémero/lock, REPL, fiabilidade, controlo | fechadas |
| H7–H8 | eixo de memória (knudge) e embeddings opcionais | fechadas |
| H9 | fachada de incorporação (`ktu_core::facade`) + `examples/ktu-embed` | fechada |
| H10 | convenções (`SYSTEM.md`, `AGENTS.md` aninhados, comandos, agentes, título) | fechada |
| H11 | governança (ADRs, postmortems, dossiê, medição, teto de superfície) | fechada |
| H12 | gates G10–G15 (artefactos em `wiki/gates/`) | fechada |

**Fase 3** (adiados por YAGNI, com critério de reabertura) em
[`wiki/implementation_phase2/ADIADOS.md`](wiki/implementation_phase2/ADIADOS.md) e no
[dossiê de pendências](wiki/pendencias.md).

## 1. Gate G4 — confirmado fechado ✅

**Issue** `task_010xrtzt` (GATE inegociável) → `closed`, com 2 outcomes `success` e
todos os filhos fechados.

| Task | Estado | Evidência |
|---|---|---|
| `task_013dfilu` harness (`scripts/quality.sh` + `xtask quality`) | closed | `--dry-run` reproduz o plano p1–p4 |
| `task_00vq487c` executar p1–p4 | closed | `wiki/quality_test/p*/ktu-*.md` + `ktu-summary.md` |
| `task_00kz804y` julgar/arquivar | closed | `wiki/quality_test/ktu-veredicto.md` |
| `task_016ifxg0` repetir até equivalência | closed | veredicto final |

Critérios de `plan.md` §5, conferidos nos artefactos:

- **Bateria isolada por sujeito** — `xtask/src/quality/{workspace,exec}.rs`, root
  privado, `REF/` read-only.
- **Q1–Q3 `same_session: true`; Q4 `compacted: true`** — frontmatter de
  `p1..p4/ktu-p*.md`.
- **Sonda de persistência** — `true` no resumo (`p4`), via processo novo
  (`memory list`), sem regra extra no prompt.
- **Artefactos ao lado dos referenciais** — `p1/p1-goose.md`… e `p1/ktu-p1.md`, etc.
- **Veredicto de equivalência de grau por questão** — Q1–Q4 equivalentes.

A correção exigida pelo gate (escrita automática de memória no fim do turno,
`kernel/memory_gate`, em vez de depender do modelo) está em `fact_01eob2w8` e
coberta por teste de regressão.

## 2. Itens anteriores (G0–G3) — revistos, todos OK ✅

As **28 tasks de G0–G3 estão `closed`, cada uma com `outcome` + âncoras a ficheiros
reais**. Amostra verificada por leitura:

- **G0** (7/7): workspace + firewall de camadas (`xtask check-layers`), lints,
  portas/DI com fakes, logs estruturados, métricas por feature, harness
  (unit+property+goldens+CI), `ktu init`.
- **G1** (7/7): tipos/`SessionLog` JSONL, `derive_messages` + projeção de wire, loop
  cancelável/Stagnation, `guard.rs`+`taint.rs`, `BudgetGate`, provider falso,
  `read_file`+`ktu run`.
- **G2** (9/9): `LlamaProvider` (SSE, tool-calling+fallback, retry), test double
  HTTP, tools fs, `bash` fail-closed, resume/cache de prefixo, trash+doctor,
  sampling/thinking, GGUF doctor, modelo real (Ornith).
- **G3** (5/5): spike do knudge-core, porta `Memory`+adaptador JSONL, memory gate
  (recall+write automáticos, dedup 0.75/0.92), compactação+checkpoint durável,
  `memory export`/`doctor --memory`.

## 3. Caveats / pontos a vigiar (não invalidam o gate)

1. **Provider da corrida G4** — usou `opencode-go`/`longcat-2.5-preview-free`
   (remoto), **não** o `llama.cpp` local, por falta de fundos; condição declarada no
   frontmatter e no veredicto (`decision_01sexrmm`).
2. **Q2** — entregou um REPL/`--human` equivalente, forma diferente da ideia
   esperada; aceite como equivalência.
3. **Q3/p3** — resposta no artefacto curta (~117 chars) e remete para o ficheiro; o
   entregável real existe (`ktu-files/knudge/ktu/knudge.md`, 8 KB).
4. **`.agents/memory/notes.jsonl` do repo está vazio** — esperado: a bateria corre em
   workspace isolado; este é o runtime de dev.
5. **`task_01kf735v` está `forgotten`** — duplicado de `task_00o74kgy` (HookEvent,
   G7); inofensivo, convém limpar.
6. **G5–G8 continuam `active`/não iniciadas** — coerente com “G4 bloqueia as fases
   seguintes”; o épico está `active` a 32/51.

## 4. Conclusão

Até ao G4 inclusive, tudo está consistente e verde. G0–G3 fecham com evidência e
âncoras; G4 está formalmente fechado com veredicto de equivalência, artefactos com
frontmatter e sonda de persistência a passar; `make check` verde. O único desvio
relevante (provider remoto em vez do `llama.cpp` local) está declarado e é uma
condição transparente, não uma falha do gate.

Com o G4 verde, a próxima fase desbloqueada é **G5 — Subagentes read-only + modo
plano** (`task_00j43pma`).

---

## 5. Avanço G5 (subagentes + modo plano)

| Task | Estado | Evidência |
|---|---|---|
| `task_00zjgygk` AgentSpec + registo | ✅ fechada | `ktu-core/src/agent/spec.rs` + `ktu-tools/src/agents.rs` (`default_registry`) |
| `task_01rbaqmz` tool `subagent` | ✅ fechada | `ktu-core/src/subagent.rs` (porta) + `agent/delegation.rs` + `ktu-cli/src/subagents.rs` |
| `task_01sai4yj` fan-out | ✅ fechada | `agent/delegation.rs::run_calls` (`join_all`, ordem determinística, cap por turno) |
| `task_018f3xhq` modo plano | ✅ fechada | `ktu-core/src/plan.rs` (+ `actions`/`store`/`host`), `ktu-tools` `plan_mode_spec`, CLI `--plan` e `ktu mode` |

O issue `task_00j43pma` (G5) está **fechado**. Garantias estruturais verificadas
por teste: o `explorer` não vê/executa
`write_file`/`edit_file`/`bash`/`trash_file`/`memory_write` (`ScopedTools` + guard),
não cria netos (`subagents: None` + `max_depth: 0`), e o relatório é capado
(`LoopConfig.subagent_report_chars = 8192`). No modo plano, o `AgentSpec` é
read-only + `plan` + `subagent`, o artefacto vive em `.ktu/plan/<sessão>.json`
(atómico `temp→fsync→rename`), o modo é logado (`Event::ModeSet`) e sobrevive a
`resume`, e a transição `plan→build` exige aprovação (I15). `make check` verde.

**Próxima fase desbloqueada: G6 — Porta única + front-ends** (`task_00af69ek`).

Registos knudge: `fact_00uz1ew4` (revisão G4), `fact_01mbzhdz` (AgentSpec),
`fact_01ygtjoy` (subagent), `fact_0045bcrx` (fan-out), `fact_00ghyy87` (modo plano).

---

## 6. Modo plano (detalhe)

- `ktu-core/src/plan.rs`: `Mode` (`build`/`plan`), `Approval` (`Approved`/`Pending`),
  `Plan`/`PlanStage`/`PlanTask`/`PlanState`; `Mode::transition` aplica I15
  (`build→plan` livre; `plan→build` exige `Approved`).
- `plan/actions.rs`: tool `plan` (`stage_add`, `task_add`, `task_state`, `list`).
- `plan/store.rs`: artefacto `.ktu/plan/<sessão>.json` com escrita durável.
- `plan/host.rs`: `FsPlanHost` (carrega→aplica→grava).
- CLI: `ktu run --plan` (restringe a sessão a plan) e
  `ktu mode <plan|build> [--approve]` (`SetMode`, logado, sobrevive a `resume`).

---

## 7. G5.5 — Endurecimento estrutural (`task_015epqsj`) ✅

Fase de consolidação entre G5 e G6: isolou o kernel e alinhou a disciplina de
código com `.agents/skills/rust/` (1.99) e o `REF/sniff-css`.

| Task | Estado | Evidência |
|---|---|---|
| `task_016trqis` T2 (thread de tools) | ✅ | `ktu-tools/src/tool_thread.rs` (`ToolThread`/`WorkerCtx`, mpsc+oneshot) |
| `task_00xdwxvl` quebrar a god-struct `Deps` | ✅ | `Ports`/`Telemetry`/`Capabilities` em `agent/mod.rs` |
| `task_00js2z5y` toolchain Rust 1.99 | ✅ | `rust-toolchain.toml`, MSRV 1.99, `rustfmt` `style_edition` |
| `task_0039qlfv` supply chain + CI | ✅ | `deny.toml`, CI `check`/`supply-chain`/`miri`, `tests/miri_smoke.rs` |
| `task_00ddzsgt` pendências de organização | ✅ | `cargo machete` limpo; `tokio-util`/`indexmap`/deps mortas removidas |

Garantias novas provadas por teste: as tools correm **noutra thread**
(`tools_run_on_a_dedicated_thread`), erros de tool ficam contidos
(`tool_errors_are_contained_and_returned`), e o núcleo puro passa sob a Miri
(`miri_smoke`). `make check` e `cargo machete` verdes.

Pendente para G6: re-adicionar as dependências reais do `ktu-port` (hoje um
esqueleto sem deps).

### Revisão de rigor (2.ª passagem)

- **Logger completo**: `tool.result` passa a ser emitido num **ponto único**
  (`Driver::push_result`), cobrindo também negações/escalonamentos, cap de
  subagentes e resoluções sintéticas (estagnação). `guard.decision` e
  `tool.call`/`tool.result` vão todos pela porta `Logger` — o `risk_01cpastn`
  fica resolvido e é agora **provado por teste** (`guard_denies_a_tool_outside_the_spec`,
  `repeated_calls_are_detected_as_stagnation`).
- **`support.rs` dividido**: a emissão de eventos vive em `agent/logging.rs`
  (respeita o limite de 300 linhas por ficheiro).
- **`typos.toml`**: o projeto é português; aceitamos explicitamente o
  vocabulário (87 palavras) e excluímos `REF/`, `wiki/`, `.knudge/`, `.agents/`.
  Um erro novo continua a ser apanhado.
- **Supply chain validada com `cargo-deny`**: `advisories`/`bans`/`licenses`/
  `sources` verdes. Corrigido o advisory do `rustls` (`0.23.45 → 0.23.46`) e
  adicionada a licença `CDLA-Permissive-2.0` (`webpki-roots`). Alvo novo
  `make supply-chain` = `deny + machete + typos`.
- **Referências actualizadas**: `clippy.toml` (Rust 1.99) e `plan.md` (F0, 1.99).

---

## 8. G6 — Porta única + front-ends (`task_00af69ek`) ✅

Todos os front-ends falam a **mesma porta**; nenhum toca no kernel (I8).

| Task | Estado | Evidência |
|---|---|---|
| `task_007t2rm8` `ktu-port` protocolo + `Channel`/`Http` | ✅ | `crates/ktu-port/{protocol,port,host,channel,http}.rs` |
| `task_01s7s1ka` `ktu serve` (axum) + SSE/health/estado | ✅ | `crates/ktu-cli/src/serve.rs`, `hub.rs`, `hub/actor.rs` |
| `task_01czs2ai` fora-de-banda (cancel, `SetMode`) | ✅ | `Host::cancel` (token partilhado), `SetMode` logado/persistido |
| `task_0144p1iv` TUI (ratatui) cliente da porta | ✅ | `crates/ktu-cli/src/tui.rs` (`ktu tui`) |
| `task_01wwavuq` página web (fetch + SSE) | ✅ | `INDEX_HTML` em `serve.rs` (`GET /`) |

**Protocolo** (`ktu-port`): `ClientMsg`/`ServerMsg` em NDJSON/SSE, versão `1`.
Um `Port` (cliente) com duas impls: `Channel` (in-process, sem socket) e
`HttpPort` (HTTP/1.1+HTTP/2, SSE). O servidor implementa `Host`.

**Serviço** (`ktu-cli`): `SessionHub` com **um actor por sessão** (dono do
`SessionLog`); o turno corre em `turn_runtime::execute_turn` (partilhado com o
`ktu run`, sem DI duplicada). O `cancel` é de fora-de-banda (não espera o actor).

**Endpoints**: `GET /v1/health`, `POST /v1/session`, `GET /v1/session/{id}`,
`POST /v1/session/{id}/turn` (202), `GET /v1/session/{id}/events` (SSE),
`POST .../cancel`, `POST .../mode`, `GET /` (web).

**Provado por teste**: `hub/tests.rs` (turno + eventos + modo persistido + cancel),
`tests/serve_http.rs` (sobe `ktu serve` e conduz a sessão via `HttpPort`),
`protocol`/`http` (round-trip NDJSON + parser SSE), `tui` (formatação).

Novo id de evento estável: `server.start`. `make check` e `make supply-chain`
verdes (`axum 0.8`, `ratatui 0.30` — sem `paste`, sem advisories).

---

## 9. G7 — Plugins em runtime + guardrails declarativos (`task_002xypc1`) ✅

| Task | Estado | Evidência |
|---|---|---|
| `task_00o74kgy` HookEvent + dispatcher | ✅ | `crates/ktu-core/src/plugin.rs` + `plugin/dispatch.rs` |
| `task_010xowkm` Guardrails declarativos fail-closed | ✅ | `crates/ktu-core/src/guardrails.rs`, `.agents/guardrails/*.md` |
| `task_01gsm1nb` Runtime de plugins externos | ✅ | `crates/ktu-plugin/{manifest,process,host}.rs` |
| `task_01oo7e84` Hot reload por gerações | ✅ | `PluginHost::reload` (I12); ordem kernel→gates→execução (I11) |
| `task_00uuq3x5` Plugin de exemplo | ✅ | `examples/plugins/{input-note,guard-bash}` |

**Hooks (§2.9):** catálogo tipado único (`HookEvent` + `HookPayload`), porta
`Hooks` e `Dispatcher` determinístico (ordem alfabética do nome). Pontos vivos no
loop: `session.start/end`, `user.input`, `model.request/output`,
`action.before/after`, `guard.decision`, `kernel.error`. Cada *dispatch* entra no
log da sessão (`Event::PluginDispatch`) e na porta de log (`plugin.dispatch`).

**I11 (ordem + fail-closed):** o guardrail estrutural é avaliado **primeiro**; só
se for `Allow` os *gates* de plugin correm. Um *gate* em erro/timeout ⇒
`Deny{plugin_timeout}`; `observe`/`transform` em falha ⇒ passa adiante. Provado
por `plugin_gate_denies_and_the_denial_is_evidence` e
`a_plugin_cannot_free_a_structural_deny` (`tests/loop.rs`).

**I16 (guardrails declarativos):** schema `*.md` (frontmatter
`id/version/applies_to/fact/match/decision/reason`); loader **fail-closed**
(ficheiro inválido impede o arranque, nomeando-o); regras que tentam permitir são
**ignoradas**; `ChainGuard` garante que a camada declarativa só aperta. Seed em
`.agents/guardrails/` (`ktu init`). Provado por `guardrails/tests.rs`.

**Plugins (§2.9):** processo externo JSON-lines sobre stdio, `plugin.toml`
(`entrypoint/events/mode/timeout_ms/daemon`), comandos `ktu plugins list|reload|doctor`
e `ktu guardrails list`. Recarregar incrementa a **geração** (I12); um manifesto
inválido desactiva só o plugin, não a sessão. Provado por
`crates/ktu-plugin/tests/process.rs`.

**Novos ids de evento:** `plugin.dispatch`, `plugin.lifecycle`. Novo crate
`ktu-plugin` (firewall: não depende de tools/provider/port/memory/cli; o core não
depende dele). `make check` e `make supply-chain` verdes (`toml` sem advisories).

---

## 10. G7.5 — Endurecimento: confinamento do `bash` (`risk_01ua5ei2`)

A re-validação do gate revelou que a tool `bash` (cwd = root, mas sem
confinamento de caminhos) deixava o modelo escrever **fora** do workspace
isolado, com caminhos absolutos ou `~`. Correção em `ktu-tools/src/bash.rs`:

- `analyze_paths(script, root)` recusa `~`/`$HOME`, `..`/`../` e caminhos
  absolutos fora do *root*, permitindo apenas prefixos de sistema read-only
  (`/bin`, `/sbin`, `/usr`, `/lib`, `/lib64`, `/dev/null`, `/dev/stdout`,
  `/dev/stderr`, `/dev/fd`, `/proc`, `/sys`). Aplicado em `BashTool::run`
  depois da análise de comandos destrutivos (fail-closed).
- **Testes:** unitários (`denies_paths_outside_the_root`,
  `allows_commands_inside_the_root_and_system_tools`) + integração
  (`crates/ktu-tools/tests/bash.rs`).
- **Nota:** é uma mitigação determinística de *argv*, não um sandbox de SO
  (Landlock/seccomp fica fora do escopo, §2.8). `make check` verde.

## 11. G8 — Packaging, CI/MSRV e releases (`task_01r8grbe`)

Primeira parte do G8, mantendo simplicidade e determinismo.

**GDK adiado** (`decision_00bj1nxa`). O `goose-sdk` real (`0.1.0-alpha.12`) com
features por omissão só reexporta tipos ACP (`agent-client-protocol` +
`goose-sdk-types`) para falar com `goose acp` por stdio; a biblioteca de
providers é `goose-providers` (alpha), que depende de `rmcp` (MCP, na lista
YAGNI) e exige rede. Integrá-la como `Provider` do ktu criaria um segundo agente
a competir com o loop do kernel. `task_010qrmko` fica aberta mas adiada; o
caminho OpenAI-compatible (`opencode-go`) cobre os providers remotos atuais.

**Entrega:**

- `scripts/dist.sh` — empacota `target/release/ktu` em
  `target/dist/ktu-<versão>-<triple>.tar.gz` (+ `.sha256`), determinístico pelo
  alvo (`KTU_TARGET`, default host) e pela versão do `Cargo.toml`.
- `install.sh` — instala a partir de uma release (descobre a `latest` pela API,
  descarrega, **verifica o SHA-256**, extrai para `~/.local/bin`) ou compila do
  código-fonte (`KTU_FROM_SOURCE=1`). `KTU_RELEASE_BASE` permite
  teste/auto-hospedagem.
- `.github/workflows/release.yml` — tag `v*` → matriz
  `x86_64-unknown-linux-gnu` + `aarch64-apple-darwin` → `dist.sh` → anexa à
  release.
- `ci.yml` — jobs novos: `msrv` (`cargo check --workspace --all-targets
  --locked` em 1.99) e `dist` (smoke do empacotamento).
- `make dist`; secção de instalação no `README.md`.

**Evidência (local):** `make dist` produziu
`ktu-0.0.1-x86_64-unknown-linux-gnu.tar.gz` (2,8 MB) + checksum; `install.sh`
validado em 4 caminhos — fonte (ok), release via `file://` (checksum
`SUCESSO`), checksum errado (falha), repo inexistente (mensagem limpa). Sem tag
(regra: sem commits), pelo que a release real só dispara quando a pessoa criar
a tag.

## 12. G8 — Benchmarks com evidência (`task_005zfzx3`)

Medir antes de optimizar (`plan.md` §1.4: um número sem artefacto não fundamenta
decisão).

- **Harness:** `crates/ktu-core/benches/core.rs` (`harness = false`), sem modelo
  (provider/tools falsos), corre com `make bench` (ou
  `cargo bench -p ktu-core --bench core`). Escreve
  `wiki/benchmarks/ktu-bench.json` (host, CPU, `rustc`, RSS, tamanho do binário)
  e imprime uma tabela. Fora do `make check` (medições ruidosas).
- **Medições (Ryzen 5 5500U, `rustc` 1.99, 2026-10-17):**
  `derive_messages` 6 µs/100 · 45 µs/1 k · 534 µs/10 k eventos; `log_build_1k`
  45 µs; turno com 1 passo 5 µs, com 8 passos 12 µs; pico de RSS 8 MiB; binário
  de release 6,0 MB.
- **Conclusão (`decision_01e2183g`):** o núcleo custa **microssegundos**; o
  gargalo é o **modelo**. Pela prioridade do `plan.md` §2 (performance terciária),
  **não há micro-optimização a fazer** — e há número para o dizer. Revisitar só
  se uma medição cruzar um limiar (ex.: projeção > 5 ms). Interpretação em
  `wiki/benchmarks/README.md`.

## 13. G9 — GDK: decisão de não implementar (`task_010qrmko`)

Avaliação com evidência (não por impressão) da integração do GDK do goose.

- **`goose-sdk` (`0.1.0-alpha.12`)** com features por omissão só reexporta tipos
  **ACP** para falar com o binário `goose acp` por stdio; a biblioteca de
  providers é **`goose-providers`** (também alpha).
- **Custo:** `goose-providers` puxa **~167 pacotes** extra (Cargo.lock de um
  crate novo) — incluindo **`rmcp`/MCP**, na lista YAGNI do `plan.md` §0 — num ktu
  com 267 crates únicos.
- **Determinismo:** exige rede (sem test double HTTP próprio); não é testável
  como o resto.
- **Arquitectura:** o goose é um **agente** (ACP) ou traz tipos de mensagem
  próprios; integrá-lo como `Provider` obriga a adaptador e/ou a um segundo
  agente a competir com o loop do kernel.
- **Cobertura atual:** o caminho OpenAI-compatible (`opencode-go`) já serve os
  providers remotos usados.

**Decisão (`decision_00aw5etj`):** não implementar o GDK. Se surgir necessidade
concreta (ex.: Anthropic), o caminho coerente é um **provider fino por dialeto**
(ex.: *Messages* da Anthropic), determinístico e testável com o test double HTTP
existente — como o `opencode-go`. `task_010qrmko` e a G9 fechadas; épico
`epic_00ynu6q8` concluído (G0–G9).


