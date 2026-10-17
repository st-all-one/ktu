# Revisão do `katu` — ADRs, planos e dossiê de pendências

> Revisão **profunda** do `REF/katu` (kernel agêntico Rust, o referencial mais próximo do ktu)
> com o filtro do ktu: **simplicidade · kernel · usabilidade objectiva**. Ao contrário da
> [comparação de escala](./revisao-katu-vs-ktu.md), aqui lêem-se as **27 ADRs**, os **planos**
> (`OPTIMIZATION_PLAN`, `KERNEL_SURFACE`, `LOOP_RESILIENCE`, `PI_GAINS`, `19-otimizacao-profunda`)
> e o **dossiê de pendências**. Data: 2026-10-17.

---

## 0. Método e escala

| | `katu` | `ktu` |
|---|---|---|
| Crates | 7 (`katu`, `-core`, `-policy`, `-providers`, `-tools`, `-tui`, `knudge`) | 7 |
| ADRs | **27** | — (usa `plan.md`/`STAGING.md`) |
| Governança | ADRs + dossiê de pendências + postmortems + `surface.toml` + `published.toml` | `plan.md` + `STAGING.md` |
| Escopo | kernel + política + memória + auditoria + providers + TUI | kernel + back/front |

O katu é **o mesmo animal** que o ktu (kernel Rust, log append-only, portas, firewall), mas levado
muito mais longe **no rigor documental**. O maior valor para o ktu **não** é copiar features: é
copiar o **método** — e evitar os becos que o katu já mediu e rejeitou.

---

## 1. O que o katu ensina ao nível da governança (o mais valioso)

### 1.1 ADRs com "Alternatives considered" obrigatório · **INCORPORAR**

**O que é:** cada decisão vive numa ADR imutável (`NNNN-titulo.md`) com secções obrigatórias
`Contexto`/`Decisão`/**`Alternatives considered`**/`Consequências`; uma ADR não se edita —
**substitui-se** por outra ligada por `Supersedes`/`Superseded by`. Um gate (`xtask check-docs`)
falha se faltar a secção das alternativas. *"Uma decisão arquitectural sem o que venceu convida a
re-litigá-la."*

**Porque interessa ao ktu:** o ktu decide em `plan.md`/`STAGING.md` (prosa corrida). Uma ADR por
decisão dá rastreabilidade e impede re-litigar. É o mecanismo que **já justificou** decisões como
`decision_00aw5etj` (GDK) e `decision_01e2183g` (sem micro-optimização) — só que no knudge, não
num doc versionado.

**Onde entra:** `wiki/adr/NNNN-*.md` + `xtask check-adr` (verifica as secções). **Esforço:** baixo.

### 1.2 Dossiê de pendências vivo · **INCORPORAR**

**O que é:** `DOSSIE_PENDENCIAS.md` — um levantamento **verificado no código** do que falta,
classificado por estado (`⛔ Deferido` · `🚫 Rejeitado` · `⏳ Opt-in` · `🔴 Lacuna` · `⚪ Formalismo`)
e por eixo (**Q** qualidade · **P** performance · **S** simplificação), com o **número** que
justificou cada rejeição.

**Porque interessa ao ktu:** o `STAGING.md` diz o que **está feito**; falta o **inverso** — o que
falta, porquê, e o que foi rejeitado com que número. É o antídoto do "fizemos tudo".

**Onde entra:** `wiki/pendencias.md` (ou secção do `STAGING.md`). **Esforço:** baixo.

### 1.3 Método: medir → A/B → adotar-ou-reverter · **INCORPORAR**

**O que é:** todo o item exige **fórmula + artefacto + teste que o trava**; **A/B** com recorte cru;
**adotar-ou-reverter** (≥ 20 % no alvo **ou** remoção de complexidade; senão reverter e **escrever
a rejeição com o número**). `bench/published.toml` publica cada número com uma **base de evidência**
(`measured`/`inferred`/`provider_reported`/`unpriced`) — *nunca um número sem base*.

**Porque interessa ao ktu:** o ktu já mede (`wiki/benchmarks/ktu-bench.json`) e já decidiu por
medição (`decision_01e2183g`). Falta formalizar: (a) a **base de evidência** de cada número
publicado; (b) a regra **adotar-ou-reverter**; (c) o registo das **rejeições com número**.

**Onde entra:** `wiki/benchmarks/` (coluna `basis`) + regra no `plan.md`. **Esforço:** baixo.

### 1.4 Postmortems · **AVALIAR**

**O que é:** `docs/postmortems/` — um template e o registo de incidentes (ex.:
`0001-fail-open-changed-files.md`).

**Porque interessa:** o ktu não tem registo de incidentes. Um postmortem por regressão real
(ex.: o HTTP 400 do `reasoning_content` foi um incidente) evita repeti-lo. **Esforço:** baixo.

### 1.5 Teto de superfície (`surface.toml`) · **INCORPORAR**

**O que é:** a superfície pública tem um **teto versionado**; `xtask check-surface` falha se crescer
sem subir o teto numa PR. Kill criterion K3: *"se o teto exceder em duas fases consecutivas sem
valor, podar antes de continuar"*.

**Porque interessa:** o ktu preza minimalismo; um teto **mecânico** impede o crescimento silencioso
da CLI/tools. **Onde entra:** `surface.toml` + alvo no `xtask`. **Esforço:** baixo.

---

## 2. Confirmações (o ktu já acertou — não mexer)

O katu valida, com medição, decisões que o ktu **já tomou**:

| Decisão do katu | O que o ktu já tem |
|---|---|
| **KERNEL_SURFACE**: kernel numa thread, front como cliente, protocolo `Command`/`Event` `serde` | back em 2 threads + front desacoplado; `ktu-port` |
| **Firewall LLM-free** (`katu-core` ↛ providers/tools) | `xtask check-layers` verde |
| **Determinismo**: sem RNG, ordem canónica, `total_cmp`, `clamp01`, `NaN→0` | só `BTreeMap`/`BTreeSet`; portas com fakes |
| **`Model-visible ⟺ logged`** (o histórico é projeção do log) | `derive_messages` a partir do log; teste de determinismo do wire |
| **Portas com fakes** (`Clock`/`Rng`/`Fs`/`Env`) | `ktu-core::ports` + `testing.rs` |
| **Diag**: ids estáveis, sem texto livre, custo zero por omissão, nunca no plano de dados | `events.rs` + `logging.rs` |
| **`StopReason`** com mensagem visível (nunca em silêncio) | `StopReason::{EndTurn, MaxSteps, Cancelled, Error, Stagnation}` |
| **Guard de loop/estagnação** | `Stagnation` (`agent/support.rs`) com `repeats`/`threshold` |
| **Instrução nunca evictada** (`pin_unit`) | `budget.rs::pin_last_user` |
| **Projeção prefix-stable** (cache de prefixo) | `session/tests.rs` prova prefixo byte-a-byte |
| **Cauda rasgada recuperada** | `store.rs`: o prefixo válido é recuperado |
| **Retry com backoff** | `ktu-provider/src/llama.rs` (`is_retryable`, `retry_backoff`) |
| **Cancelar injecta resultados sintéticos** | `cancel.rs` (`StopReason::Cancelled`, I2) |
| **Escrita durável** (`temp→fsync→rename`) | `checkpoint.rs`, `plan/store.rs`, `ktu-memory/store.rs` |
| **ADR 0026**: rejeitar o `batch` declarativo (ganho alegado, não medido) | YAGNI §0 do `plan.md` |
| **ADR 0027**: HTTP/1.1+SSE; HTTP/2/WS `Unsupported` tipado | provider bloqueante; dialetos tipados |
| **ADR 0025**: `structured_output` opt-in, fail-open | (ktu não tem — ver §3.6) |

> **Leitura:** o ktu e o katu convergiram por caminhos independentes. Isso é a melhor validação
> possível da arquitectura do ktu. **Não reabrir.**

---

## 3. Candidatos novos a incorporar (verificados no código do ktu)

Cada item foi confirmado **ausente** no ktu (grep em `crates/*/src`).

### 3.1 Política de durabilidade do log (`event` vs `turn`) · **INCORPORAR**

**O que é (ADR 0024):** `behavior.durability` = `event` (default, `fsync` por evento) ou `turn`
(*group commit*: escreve sem sincronizar, barreira no fim do turno). Janela de perda explícita por
modo. Complemento **obrigatório**: recuperação de **cauda rasgada** (descartar a última linha
incompleta + `log.recovered`). Medido: **−78,1 %** no caminho de anexação.

**Estado no ktu:** `fsync` presente, **sem** política opt-in (`durability` = 0 hits). A cauda rasgada
**já é recuperada**. **Onde entra:** `store.rs` + config. **Esforço:** baixo-médio.

### 3.2 Lock de turno por sessão (entre processos) · **INCORPORAR**

**O que é (L-Q6):** `turn.lock` (pid + idade; obsoleto após 300 s) impede **dois processos** no
mesmo log; recusa com `conflict`. O katu regista que `turn_open` (estado do **processo**) **não**
deteta outro processo — o lock é a única leitura **entre processos**.

**Estado no ktu:** `turn_lock` = 0 hits. **Onde entra:** `session/mod.rs` + porta `Fs`. **Esforço:** baixo.

### 3.3 Deteção de eco · **INCORPORAR**

**O que é (L-Q4):** comparar o texto final do modelo com os `delta` recentes (trim + colapso de
whitespace; igualdade/prefixo longo); eco → *nudge* + **1 retry**; esgotado o orçamento, o eco é
**substituído** por nota visível — **nunca aceite** (G5).

**Estado no ktu:** `echo` só aparece no `EchoProvider` de teste; **sem** deteção. Modelos fracos
devolvem o prompt como resposta. **Onde entra:** `agent/mod.rs` (fecho do turno). **Esforço:** baixo.

### 3.4 Tool calls declaradas como texto · **INCORPORAR**

**O que é (W8-1b):** modelos locais **não** emitem `tool_calls` nativas — escrevem a chamada no
`content` (bloco JSON ou `<tool_call>`). O katu decodifica a declaração **contra o catálogo
fechado** quando o passo não traz chamadas nativas; o texto declarado **não** entra no log como
mensagem do assistente.

**Estado no ktu:** `declared` = 1 hit (um teste). Sem decodificação de declarações. **Onde entra:**
`agent`/`wire`. **Esforço:** médio. **Risco:** aceitar chamadas que o modelo não "quis" — validar
contra o catálogo, fail-closed.

### 3.5 Recusa acionável (`remedy` por regra) · **INCORPORAR**

**O que é (Q-08):** cada regra negada devolve um **`remedy`** (dado, não código) — o que fazer a
seguir; `policy:audit` falha sem ele. O `edit` devolve as **âncoras únicas mais próximas** (Q-07).
*"Um erro que ensina vale um turno."*

**Estado no ktu:** `remedy` = 0 hits; os guardrails devolvem recusa sem "o que fazer". **Onde
entra:** `guardrails.rs`/`guard.rs`. **Esforço:** baixo. **Ganho:** menos turnos repetidos.

### 3.6 Saída estruturada (`response_format` json_schema) · **AVALIAR**

**O que é (ADR 0025):** `response_format: {json_schema}` derivado das tools do pedido, **opt-in** e
**fail-open** (400 ⇒ repetir sem o campo). Fecha a classe de erro `Decode` por construção. Custo
medido: **+962 B**/pedido, ~4× a codificação. **Revertido para `off`** no katu por partir a
terminação (o schema `oneOf` não tem variante de resposta final).

**Estado no ktu:** `structured` = 0 hits. **Veredicto:** só reconsiderar com **variante terminal**
de texto; senão repete-se o erro do katu. **Esforço:** médio.

### 3.7 Visibilidade das mensagens (`User` vs `Agent`) · **INCORPORAR**

**O que é (KS-G3):** `Visibility::{User, Agent}` no log/projeção: o *nudge* interno chega ao
**modelo** mas **não** à transcrição do utilizador.

**Estado no ktu:** `visibility` = 0 hits. **Onde entra:** `domain.rs` + projeção + TUI. **Esforço:** baixo.

### 3.8 Cancelamento dentro das tools (kill do grupo) · **AVALIAR**

**O que é (WL3-3):** `Ports`/`TurnRequest.cancel` → `ExecTool.cancel` → `Process::run(request,
cancel)`; o `StdProcess` mata o **grupo** de processos ao cancelar. Sem isto, cancelar a meio de um
`bash` de 30 s deixa o processo pendurado.

**Estado no ktu:** `cancel.rs` existe, mas a propagação até ao processo não foi confirmada. O ktu
confinou o `bash` (mitigação `risk_01ua5ei2`). **Onde entra:** `ktu-tools/src/bash.rs`. **Esforço:** médio.

### 3.9 Retry endurecido do provider · **INCORPORAR**

**O que é (ADR 0011/0012):** retry **só antes do primeiro delta** (depois duplicaria texto);
`Retry-After` em segundos/ms/**data `HTTP`** (descontando o `Date` — o provider não toca relógio);
limites de conta/quota são **permanentes** (não repetir); **sanitização de URLs** (credenciais/query
fora do log).

**Estado no ktu:** retry existe (`llama.rs`); falta o *só antes do 1º delta*, o `Retry-After` em
data e a sanitização de URL. **Onde entra:** `ktu-provider`. **Esforço:** baixo-médio.

### 3.10 Cache de prefixo declarativo + cabeçalhos de afinidade · **AVALIAR**

**O que é (ADR 0013):** `prompt_cache`/`prompt_cache_retention` são **campos declarativos por
modelo**; o `prompt_cache_key` deriva da sessão; `affinity_headers` (`x-client-request-id`,
`x-session-affinity`) são declarativos. Medido: `cached=896/1004` no 2.º turno.

**Estado no ktu:** a projeção é prefix-stable (a condição necessária) e há cabeçalhos de sessão;
falta a declaração por modelo. **Onde entra:** `ktu-provider` + catálogo. **Esforço:** baixo.

### 3.11 `edit` multi-bloco atómico · **INCORPORAR**

**O que é (Q-07):** N edições num só chamada, **atómico** (0 B gravados em falha) — 5 chamadas → 1
(**−80 %**), payload −19,9 %.

**Estado no ktu:** o `edit` do agente é uma edição por chamada. **Onde entra:**
`ktu-tools/src/fs_tools.rs`. **Esforço:** baixo. **Ganho:** menos turnos.

### 3.12 Continuação de leitura truncada · **INCORPORAR**

**O que é (dossiê §4):** o aviso de truncagem indica a **continuação** (`read <id>@<linha>`) em vez
de só dizer "truncado"; a leitura inexistente devolve `read.missing` legível (não `Unavailable`).
O modelo repetia a leitura até o guard cortar.

**Estado no ktu:** `continu` = 14 hits (provavelmente outros usos). **Onde entra:**
`ktu-tools/src/fs_tools.rs`. **Esforço:** baixo.

### 3.13 Configuração global/local + `--params` XOR flags · **AVALIAR**

**O que é (ADR 0020):** uma config global por SO (`~/.config/.../katu.toml`), override local
`<projeto>/.katu/katu.toml` (precedência projeto > global, chave a chave), **conjunto fechado de
chaves** (`deny_unknown_fields`), `--params` (toda a config) **exclusivo** com flags (ambos → exit 2).

**Estado no ktu:** config = 17 hits (provavelmente o `.agents/`). **Onde entra:** `ktu-cli`. **Esforço:** médio.

### 3.14 Layout central versionado · **AVALIAR**

**O que é (ADR 0021):** `.katu/` com layout explícito; **sempre fora do git** (`audit/`, `trash/`,
`log/`), o resto segue `git.versioned`; blocos geridos em `.gitignore`/`.git/info/exclude`; `--init
--force` preserva o conhecimento.

**Estado no ktu:** `.agents/` existe; a política de versionamento explícita não. **Esforço:** médio.

### 3.15 Snapshot no limite de fase (retomada rápida) · **AVALIAR**

**O que é (ADR 0008, Q-15):** snapshot do estado a cada transição de fase com `{seq, offset, …}`;
`resume` lê só a **cauda** (não relê o prefixo). Medido: 20 000 turnos em **321 µs** (−98,7 % vs
replay). O ktu tem `checkpoint.rs` — avaliar se cobre a retomada rápida.

**Esforço:** médio. **Risco:** divergência snapshot↔log (validar a invariante na retomada).

### 3.16 `prime` byte-idêntico por versão · **AVALIAR**

**O que é (ADR 0019):** `prime` **estático e byte-idêntico por versão**; um `prime` por grupo. O
ktu compõe o prompt a cada turno (sem versionar o prime). **Esforço:** baixo.

---

## 4. O que o katu tem e o ktu **não** deve trazer (YAGNI §0)

| Feature do katu | Porquê não (para o ktu) |
|---|---|
| **Motor de política** (`katu-policy`, `evaluate` puro, `Enforcement` versionado) | o ktu tem guardrails declarativos + `Guard`; §0 exclui policy engine |
| **Auditoria densa + índice invertido** (`KAI1`, Bloom) | §0 exclui audit store; o ktu tem o log |
| **Porta `Memory` + adaptador knudge in-process** | o ktu tem `ktu-memory` (knudge-core) — equivalente mais simples |
| **Jail de SO** (bwrap/Landlock/seccomp) | §0 exclui sandbox de SO |
| **Plugins WASM / ABI** | §0 exclui WASM; o ktu tem plugins por processo |
| **Adaptador MCP** | §0 exclui MCP |
| **`responses`/`messages`/`google` dialetos** | backlog 14 (provider fino por dialeto) |
| **Formalismos do Anexo A** (POMDP/MCTS, bandit, t-SNE/HLL, cross-encoder, conformal) | §0; o gargalo é o modelo |
| **`watch_service`** (systemd timer) | fora de escopo |
| **TUI completa** (ratatui) | backlog 46 |

---

## 5. Itens que o katu **rejeitou com o número** — não repetir o erro

O katu mediu e rejeitou (com artefacto). O ktu **não** deve repropor:

| Item | Número que o rejeitou |
|---|---|
| **A4** retrieval semântico de tools (top-k) | ganho máximo seguro **14,0 %** < 20 % |
| **C2** conformal prediction | cobertura **913–947 ‰** < 950 ‰; **0** ensaios (`n_cal ≥ 19`) |
| **PERT/CPM** sobre o DAG do plano | **0 arestas** → caminho crítico é a lista |
| **PPR** semeado pelo working set | **0 arestas**; o RRF já funde |
| **DPP** (determinantal) | o MMR já resolve a diversidade |
| **`batch` declarativo** (ADR 0026) | ganho de ~80 % é **alegado, não medido** |
| **Runtime programável real** (quickjs/wasmtime) | contradiz binário único / zero-dep |
| **Partilha estrutural do `State`** | `memcpy` de KiB dentro dos 321 µs — abaixo do ruído |
| **Cache de condensação do prompt** | `fs.write`+`sync_all` **25,8 ms** vs. µs da passagem linear |
| **`structured_output` por omissão** | parte a terminação (sem variante terminal de texto) |

> **Lição transversal:** *"um lint de estilo não substitui uma medição"* (B-01 deu −5 % e quase
> reverteu uma otimização por causa de um `needless_collect`). O ktu já tem esta cultura
> (`decision_01e2183g`) — o katu dá-lhe **artefacto**.

---

## 6. Pendências do katu que são **avisos** para o ktu

Do dossiê, o que o ktu pode **antecipar** (sem esperar pela dor):

1. **Contexto: `raw_min` fixo evicta resultados de tools** → o modelo relê os mesmos ficheiros em
   ciclo (`termination=loop` no passo 52). O ktu tem `budget.rs`; **verificar** que o tecto cru não
   evicta resultados recentes (o ktu já tem `pin_last_user`, mas não um `raw_min` reservado).
2. **`tool_voi` + `context_selection=utility` causou leituras repetidas** → o ktu **não** deve
   adotar VOI/seleção por utilidade sem o A/B (o katu reverteu).
3. **`structured_output` parte a terminação** → ver §3.6.
4. **Leitura inexistente devolve `Unavailable`** → parece falha transitória; o modelo repete. Ver §3.12.
5. **Chave `provider.structured_output` colidia com o escalar `provider`** (TOML) → escrita
   descartada em silêncio; o katu ganhou um guard `keys_have_no_scalar_table_collisions`. O ktu,
   se adotar config TOML, precisa do mesmo guard.
6. **`--no-default-features` não compilava** (import incondicional de um `mod` gated) → o ktu, se
   adotar features, deve ter um gate que compile todas as combinações.

---

## 7. Recomendação

**Incorporar já (barato, alto valor):**
1. **Governança**: ADRs com `Alternatives considered` (§1.1) + **dossiê de pendências** (§1.2) +
   **método** medir→A/B→adotar-ou-reverter com `basis` (§1.3) + **teto de superfície** (§1.5).
2. **Robustez do loop**: deteção de **eco** (§3.3) + **lock de turno** (§3.2).
3. **Prompt**: **recusa acionável** (`remedy`) (§3.5) + **visibilidade `User`/`Agent`** (§3.7) +
   **continuação de leitura** (§3.12).
4. **Tools**: **`edit` multi-bloco atómico** (§3.11).

**Avaliar:**
5. **Durabilidade `event`/`turn`** (§3.1), **tool calls declaradas** (§3.4), **retry endurecido**
   (§3.9), **cancelamento dentro das tools** (§3.8), **cache de prefixo** (§3.10),
   **saída estruturada** (§3.6, só com variante terminal), **config global/local** (§3.13),
   **layout versionado** (§3.14), **snapshot** (§3.15), **`prime` versionado** (§3.16).

**Manter fora:** motor de política, auditoria densa, MCP, WASM, jail de SO, dialetos extra,
formalismos do Anexo A, `watch_service`.

**Não repropor** (§5): A4, C2, PERT/CPM, PPR, DPP, `batch` declarativo, runtime programável,
partilha estrutural, cache de condensação, `structured_output` por omissão.

> **Nota final.** O katu confirma, por medição, **quase todas** as decisões estruturais do ktu
> (§2). O que falta ao ktu não é arquitectura: é **governança** (registar decisões, pendências e
> rejeições) e um punhado de **robustez de loop**. É aí que está o retorno.
