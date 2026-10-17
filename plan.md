# ktu — plano da 2.ª tentativa

> **Tese:** *fazer funcionar, depois melhorar.* A 1.ª tentativa (`REF/katu`, ~70k linhas, 7 crates,
> MVK assinado, plano de otimização Q-21 fechado) **nunca operou uma sessão agêntica estável** —
> o loop perdia contexto a cada turno (ver transcrição real em `.katu/transcript.md`: tools com
> argumento inválido, repetição de chamadas falhadas, sessão sem recuperação). Este plano inverte a
> ordem: um **slice vertical mínimo que funciona de ponta a ponta**, e só depois otimização — sempre
> com número na mão.
>
> **Métrica de aceite:** a bateria `wiki/quality_test/p1`–`p4` existe para colocar os agentes de
> referência (`REF/goose`, `REF/pi`, `REF/opencode`) **no mesmo contexto**, a responder às mesmas
> perguntas, **sem que um interfira no outro** — cada sujeito corre isolado, com sessão e memória
> próprias, e o resultado fica arquivado por questão (`p1/p1-goose.md`, `p1/p1-pi.md`,
> `p1/p1-opencode.md`…), com o `frontmatter` a registar as condições (`provider`, `model`,
> `thinking-level`, `cache`, `same_session`, `compacted`). O ktu entra como **mais um sujeito
> isolado**, nas mesmas condições, e tem de atingir o **mesmo grau de qualidade** dos referenciais
> — mais a persistência entre sessões (Q3 escreve, Q4 lê de novo após compactação; e uma sessão
> nova, de processo fresco, recupera o que ficou gravado).

---

## 0. O que se aprendeu com a 1.ª tentativa (e o que este plano proíbe)

| Falha observada em `REF/katu` | Causa-raiz | Regra do ktu |
|---|---|---|
| Contexto perdido a cada turno | estado vivo só em memória; log não era fonte da verdade | **log append-only em disco é a fonte da verdade**; o pedido ao modelo é *derivado* do log |
| Tool chamada com argumento inválido (`/REF/knudge` como caminho) repetida 3× | schema fraca, erro de tool não devolvido ao modelo de forma accionável | tool devolve **erro tipado + dica de correcção**; o loop nunca morre por tool: vira mensagem |
| Sessão não retomável | compactação ambígua | `resume` por id; compactação **sem perda** (resumo + cru em disco + âncora verificável) |
| 70k linhas, 0 sessões | policy, audit+bloom, rate-distortion, containment, confiança, diag-catalog **antes** do loop | **F0–F2 sem nenhum desses.** Cada subsistema entra só quando um teste do loop o exigir |
| Turno pendura sem resposta | cancelamento/degradação não modelados | todo turno termina com `StopReason` explícito; `cancelled` é estado de primeira classe |

**Não-fazer (YAGNI explícito, até o gate de qualidade passar):** motor de política, audit store,
selecção de contexto por rate-distortion, contenção soft, sistema de confiança, catálogo de diag,
sandbox fail-closed com Landlock/seccomp, MCP, TUI extravagante, provider GDK. Nos plugins: WASM,
dispatch paralelo, API rica de plugin, marketplace. Nada disso entra sem uma issue que aponte o
teste que o exija; os pontos de intercepção são exceção: entram no F1 (§2.9).

---

## 1. Invariantes do sistema (travadas por teste desde o F1)

| # | Invariante | Como é imposta |
|---|---|---|
| **I1** | Todo `TurnStart` tem `TurnEnd`; em repouso nenhum turno fica aberto (erro, cancel, tecto) | fecho do turno em todos os caminhos do loop |
| **I2** | Nenhum `ToolCall` no log sem o `ToolResult` correspondente | verificação do log (`session.verify()`); cancel injecta resultado sintético |
| **I3** | *Model-visible ⟺ logged*: tudo o que o modelo viu está no log e é reconstruível | `derive_messages(log)` é a **única** fonte do pedido |
| **I4** | Cancelamento devolve texto parcial + `cancelled` + turno fechado | `CancellationToken` verificado em todos os `.await` do loop |
| **I5** | O turno termina por fim natural, tecto de passos, cancelamento ou erro — **nunca em silêncio** | `StopReason` obrigatório |
| **I6** | Replay determinístico: mesmo log + mesmo provider-gravado ⇒ mesma conversa | provider fake com fixtures (teste de ouro) |
| **I7** | Nenhum caminho de código usa `unwrap/expect/panic`/`todo` em `src/` | lints do workspace (ver `clippy.toml`, já no repo) |
| **I8** | O kernel e a thread de tools nunca partilham estado com o front-end — só mensagens | revisão de arquitectura + tipos (`Send` com fronteira clara) |
| **I9** | O read-only do subagente é **estrutural**: conjunto de tools do `AgentSpec` + rejeição por capacidade na thread de tools | teste que injeta uma tool de escrita numa sessão `explorer` e prova a recusa |
| **I10** | Nenhuma tool produz efeito **antes** de `Guard::evaluate` devolver `Allow`; `Deny` e `Escalate` nunca são ignorados por nenhum caminho (incl. subagentes) | fixture adversarial por regra do catálogo §2.8 |
| **I11** | **Ordem de enforcement:** o guardrail do kernel é avaliado **primeiro**; plugins só podem **adicionar** gates e transformar dados — nenhum plugin transforma `Deny` em `Allow`. Gate de plugin em timeout/erro = **fail-closed** (`Deny`); observe/transform em falha = passa adiante | teste com plugin que tenta libertar uma escrita negada + teste de timeout de plugin |
| **I12** | **Hot reload não derruba sessão:** eventos em voo completam na geração anterior do plugin; recarregar nunca fecha turno nem sessão | teste de reload a meio de turno, com sessão viva antes e depois |
| **I13** | Output de tool e conteúdo de ficheiro são **dado não confiável**: entram no contexto dentro do envelope *taint* (tags de abertura/fecho, escape sem alterar comprimento, verificação estrutural) | testes do `taint` (§2.11): payload integral no envelope, zero tags cruas no interior |
| **I14** | **Um só dono do teto de contexto** (`BudgetGate`); atingir o teto é recusa/compactação, nunca cortar evidência | teste de orçamento + revisão: nenhum outro módulo mantém teto próprio |
| **I15** | **Modo plano é read-only estrutural** e a transição de modo exige aprovação; o modo é logado e nunca escalado pelo modelo | teste: tool de escrita negada em `plan`; `plan→build` sem aprovação = recusa |
| **I16** | **Guardrails declarativos são fail-closed:** regra inválida em `.agents/guardrails/` impede o arranque (a nomear o ficheiro); ficheiros nunca desligam invariantes estruturais (I9/I10) | carga de ficheiro quebrado = recusa; ficheiro que tenta permitir escrita no explorer = ignorado |
| **I17** | **Contrato de memória do knudge é lei:** o registo é a verdade, o índice é derivado e descartável; escrita atómica antes do evento; write idempotente (`<0.75` cria / `0.75–0.92` merge / `≥0.92` rejeita); busca determinística `(score desc, id asc)`; *hints* são ponteiros (`id+statement+score`), nunca o corpo | testes de conformação contra os *goldens* do knudge; apagar `index/` e reconstruir não perde nada; recall idêntico após rebuild |

---

## 2. Arquitectura

### 2.1 Crates (mínimos, com firewall)

```
  ┌──────────┐   ┌──────────┐   ┌───────────┐        front-ends (nunca partilham estado)
  │ ktu-cli  │   │ ktu-tui  │   │ ktu-web   │──── todos falam a MESMA porta ▶
  └────┬─────┘   └────┬─────┘   └─────┬─────┘
       │        (HTTP/1.1 + SSE/NDJSON, 127.0.0.1 ou in-process)
       ▼              ▼                ▼
  ┌──────────────────────────────────────────────┐
  │ ktu-port  (protocolo + transporte)           │  ◀ única superfície agnóstica
  └───────────────────┬──────────────────────────┘
                      │ mensagens tipadas
  ┌───────────────────▼──────────────────────────────────────────────┐
  │ ktu-core                                             THREAD 1    │
  │  · loop agêntico (máquina de estados do turno)        kernel     │
  │  · sessão: log JSONL append-only, meta, checkpoint               │
  │  · derivação de mensagens + compactação                          │
  │  ── portas: Provider, Tools, Memory, Clock, Fs, Rng              │
  └───────┬───────────────────────────────┬──────────────────────────┘
          │ ToolCall (mpsc)               │ Memory (chamada directa, sync puro)
          ▼                               ▼
  ┌───────────────────────────┐   ┌───────────────────────────────┐
  │ ktu-tools        THREAD 2 │   │ ktu-memory                    │
  │  fs/read/write/edit/      │   │  adaptador do knudge-core     │
  │  glob/grep/bash + memory  │   │  (notas = verdade, índice     │
  │  tools (containção:       │   │   derivado, ask/write/task)   │
  │  timeout, output cap)     │   │                               │
  └───────────────────────────┘   └───────────────────────────────┘
          ▲
          │ reqwest (streaming, OpenAI-compatible)
  ┌───────┴───────────────────────────┐
  │ ktu-provider (só llama.cpp)       │  llama-server local (HTTP /v1/chat/completions)
  └───────────────────────────────────┘
```

Firewall de dependências (verificado por um `xtask check-layers` mínimo):

- `ktu-core` **não** depende de `ktu-tools`, `ktu-provider`, `ktu-port`, `ktu-memory` (só traits).
- `ktu-tools` não depende de `ktu-provider`/`ktu-port`.
- `ktu-memory` não depende de nada do runtime (puro + portas).
- `ktu-port` não contém lógica de agente.

> **Nota:** o conjunto de tools oferecido ao modelo é função do `AgentSpec` em execução (§2.6) — o
> agente primário vê tudo, o `explorer` vê apenas leitura. A thread de tools valida a capacidade
> antes de executar (I9); nenhuma tool corre sem `Guard::evaluate` em `Allow` (I10, §2.8).

### 2.2 Threads (decisão do dono, honrada literalmente)

| Thread | Dono | Responsabilidade | Proibido |
|---|---|---|---|
| **T1 — kernel** | `ktu-core` | sessão, loop, provider, log, compactação, decisões de turno | tocar em filesystem de tools, terminal, socket do front-end |
| **T2 — tools** | `ktu-tools` | execução de tools (fs, exec, memória), timeouts, caps de saída | conhecer sessão, histórico, provider |
| front-end | `ktu-cli`/`tui`/`web` | renderizar eventos, enviar prompts | estado do kernel; só fala pela porta |

Implementação: dois `tokio` runtimes single-threaded (ou dois `Runtime` dedicados) — `current_thread`
basta e simplifica o determinismo. Comunicação **exclusivamente** por `tokio::sync::mpsc` + oneshot
para o resultado. Front-end nunca corre dentro de T1/T2; em modo *embedded* (CLI) o front-end liga-se
por canais, em modo *socket* por TCP — **mesmo código de protocolo nos dois casos**.

### 2.3 A porta única (`ktu-port`)

Um protocolo versionado, **NDJSON/SSE sobre stream**. In-process: canais. Externo: **HTTP sobre
`127.0.0.1:<porta>` com HTTP/1.1 *e* HTTP/2** (multiplexing: CLI + TUI + web na mesma porta, sem
head-of-line blocking), servido por **`hyper`/`axum`** — uma só pilha HTTP para toda a
superfície. **HTTP/2 é o teto**; HTTP/3/QUIC fica de fora (exigiria UDP+TLS e não traz nada ao
caso local).

O **provider** não depende desta pilha: o llama-server fala HTTP/1.1 com keep-alive (cliente
mínimo, sem HTTP/2, que nada acrescenta no loop local) e **os providers remotos vão pela GDK**, que
carrega o seu próprio transporte (goose). Ou seja: o HTTP/2 é decisão da **porta**, não do provider.

O núcleo é o mesmo: um `Port` (trait) com duas impls (`Channel`, `Http`).

```
POST /v1/session                  → cria/retoma sessão            {session_id, resumed}
POST /v1/session/{id}/turn        → envia prompt (202)            {turn_id}
GET  /v1/session/{id}/events      → SSE: deltas, tool_calls, resultados, fim de turno
GET  /v1/session/{id}             → estado + memória de contexto (tokens, passo)
POST /v1/session/{id}/cancel      → cancela o turno em curso
GET  /v1/health                   → {provider, model, tools, memoria}
```

- Eventos são **logados antes de emitidos** (I3): o front-end é um espectador do log, nunca a fonte.
- `ktu run "prompt"` (CLI, caso principal) = modo embedded, sem socket; `ktu serve` = socket.
- TUI e web são clientes HTTP do mesmo socket — zero lógica duplicada.

### 2.4 O loop (implementado do zero, em `ktu-core`)

```
turno(prompt):
  log.turn_start(prompt)
  loop (passo <= MAX_STEPS):
    msgs = derive_messages(log)          # I3 — nunca estado em memória
    stream = provider.complete(msgs, tools)
    for await delta in stream:            # cancelável em qualquer ponto (I4)
        if cancel requested: log.turn_end(cancelled); return
        log.assistant_delta(delta)        # antes de emitir (I3)
    match assistant.end:
      EndTurn(text)  => log.turn_end(end_turn); return
      ToolCalls(cs)  => log.tool_calls(cs)
                      for c in cs:                       # sequencial: estabilidade primeiro
                          r = tools.execute(c)            # thread T2, via mpsc
                          log.tool_result(r)              # erros viram resultado, não pânico
  log.turn_end(max_steps)
```

Propriedades deliberadas: **sequencial** (sem paralelismo de tools no início — elimina uma classe
inteira de bugs de interleaving; volta como opt-in depois), **derivado do log** (replay,
`resume`, compactação e testes ganham de graça), **cancelável** (token verificado em cada await).

**Projecção de wire (lição directa do katu).** `derive_messages` produz o modelo do
log; a **projecção para o provider** é uma camada separada e testada à parte: **um passo
= uma mensagem `assistant` com N `tool_calls`, seguida de N mensagens `tool`** (nunca um `assistant`
por call — era `HTTP 400` real com ≥2 calls). Cada dialeto agrupa à sua maneira
(OpenAI/llama.cpp: `tool_calls` no assistant + N `tool`; Anthropic: alternância de papéis e
**um** `user` com todos os `tool_result`; Google: `functionResponse` agrupado) — mesmo no F2, só
com llama.cpp, a separação já evita o bug quando a GDK entrar.

### 2.5 Provider (só llama.cpp built-in)

Porta em `ktu-core`:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, req: &ProviderRequest) -> Result<ProviderStream, ProviderError>;
}
```

`ktu-provider` implementa para **llama.cpp via `llama-server`**: HTTP `POST /v1/chat/completions`
(`stream: true`, `tools: [...]`), SSE parseado em deltas; tool-calls nativas quando o template do
modelo suporta; **fallback** para parsing de tool-call em texto (formato `<tool_call>…` e similar).
Auto-descoberta: `ktu providers doctor` testa `base_url`, lista modelos (`/v1/models`) e valida que
o modelo declara tool-calling.

**Modelo de referência (F2/F5):** [`ornith-ai/Ornith-1.5-9B-GGUF`](https://huggingface.co/ornith-ai/Ornith-1.5-9B-GGUF),
quant **Q4_K_M** (`Ornith-1.5-9B-Q4_K_M.gguf`, ~5,78 GB) — dense ~9B, arquitectura `qwen35`, corre
em **`llama.cpp` *stock*** (sem fork, sem gambiarras). Escolhido **para este host CPU-only**
(Ryzen 5 5500U, 17 GiB, sem GPU dedicada): é o menor quant que mantém boa qualidade de *coding/agentic*
(SWE-bench Verified 70,6; MCP-Atlas 54,2) e deixa RAM para a KV cache; `Q5_K_M` (6,64 GB) e `Q6_K`
(7,56 GB) cabem mais justos. Contexto até 262 144 tokens.
**Reasoning/tools:** é um modelo de raciocínio (o turno abre com ` thinking…<｜end▁of▁thinking｜>`) e emite
`tool_calls` nativos (`--jinja`, template do modelo). O **reasoning** é campo próprio, nunca
misturado com a resposta final, e o tecto de geração tem de o acomodar. Para o
`llama-server` não é reenviado; gateways `OpenAI` em *thinking mode* exigem-no de
volta como `reasoning_content` (ver `crates/ktu-provider/src/wire.rs`).
**Sampling (tarefas gerais):** `temperature=1.0`, `top_p=0.95`, `top_k=20`, `min_p=0.0`,
`presence_penalty=1.5`, `repetition_penalty=1.0` (código preciso: `temperature=0.6`,
`presence_penalty=0.0`). `ktu providers doctor` valida o ficheiro GGUF e faz uma sonda de tool-calling
antes de servir.

- **Outros providers (Anthropic, OpenAI, opencode-zen…) só via GDK do goose** (`goose-sdk`),
  atrás da feature `gdk`, num crate separado `ktu-provider-gdk` — **nada no core muda** para isso.
- Testes de provider **sem modelo**: um `llama-server` de brincadeiro (test double HTTP com
  respostas gravadas em fixture) exercita o caminho real de streaming/parsing.

### 2.6 Subagentes read-only (`explorer`)

**Decisão:** o ktu é **um agente único** — o loop da §2.4 não se multiplica em “agentes de
personalidade”. O que existe é **delegação para subagentes read-only**, inspirada no opencode
(`packages/core/src/plugin/agent.ts`: agente com `mode: "subagent"`, permissões `deny *` + allow só
de leitura; `packages/core/src/tool/plugin/subagent.ts`: tool `subagent` que cria **sessão-filha com
contexto fresco** e devolve só o resultado final).

- **Um só subagente no início: `explorer`.** Vasculha ficheiros (`read_file`, `list_dir`, `glob`,
  `grep`, e `memory_ask` só-leitura) e devolve um **relatório descritivo** com caminhos absolutos.
  Não pode criar, modificar, apagar, mover ou copiar nada — nem ficheiros temporários, nem relatórios
  escritos em disco.
- **O read-only é enforcement estrutural, não prompt.** Cada `AgentSpec` declara o seu **conjunto de
  tools**; o loop só oferece ao modelo as tools desse conjunto, e a thread de tools **rejeita por
  capacidade** qualquer tool fora do conjunto do agente em execução (defesa em dupla camada, mesmo
  que o modelo alucine um nome de tool). O explorer não tem, no nível de código: `write_file`,
  `edit_file`, `bash`, `memory_write`, `subagent`.
- **Sessão-filha, contexto fresco.** A tool `subagent` recebe `{agent, description, prompt,
  session_id?}` (foreground; **sem** `background` e **sem** override de modelo por agora — YAGNI).
  A filha é uma sessão de primeira classe (`.ktu/sessions/<filha>/` com `meta.parent =
  <pai>`): o pai recebe apenas o **texto final, capado** (tecto de ~8k chars) dentro do
  `<tool_result>`; o log completo da filha fica em disco, inspeccionável e retomável
  (`session_id` permite continuar a mesma conversa da filha).
- **Vários subagentes, em fan-out.** O pai pode evocar **N** subagentes — no mesmo passo ou em
  passos seguidos. As chamadas de `subagent` emitidas no mesmo passo correm **em paralelo** (tarefas
  assíncronas no runtime do kernel), e o pai faz *join* de todas antes de continuar; as restantes
  tools continuam **sequenciais** (estabilidade do lado da escrita). Caps estruturais: número
  máximo de subagentes simultâneos e por turno.
- **Profundidade máxima 1.** A filha não tem a tool `subagent` — não há subagentes aninhados
  (espelha o `experimental.subagent_depth` do opencode).
- **Threads intactas.** As filhas correm **na thread do kernel (T1)** como tarefas assíncronas: o
  turno do pai espera pelo *join*, não por uma thread. Nenhuma thread nova; a thread de tools (T2)
  serve pai e filhas, por mensagens.
- **Porquê importa para a métrica:** delegar exploração é a principal defesa contra a perda de
  contexto da 1.ª tentativa — o pai mantém o relatório (curto), não 40 leituras de ficheiros. Na
  bateria p1, o ktu deve usar o `explorer` para mapear o `REF/knudge`.

### 2.7 Memória (knudge, local e persistente)

> **Referência directa:** `REF/knudge` — projeto **bem-sucedido** na estruturação de
> memória (883 testes, contrato de bytes com *goldens*, portas + adaptadores, ≤300
> linhas/ficheiro, sem `unwrap`/`unsafe`). O ktu não reinventa: **adopta o contrato do
> `knudge-core`** e muda só onde a decisão do ktu o exige.

**Casa e container.** `.agents/memory/` é a casa; o registo é **JSONL append-only**:

```
.agents/memory/
  notes.jsonl        # registos de nota (fonte da verdade) — uma linha JSON por nota
  events/*.jsonl     # log de eventos (auditoria), com dedup-on-read
  index/             # DERIVADO (BM25 + âncoras + grafo + embeddings) — apagável
```

A diferença face ao knudge é só o **container**: lá uma nota é um ficheiro Markdown
editável à mão; aqui é uma **linha JSONL** (machine-first, *append* rápido, *replay*
trivial, escrita atomicamente segura). O contrato de **conteúdo** é idêntico. Compensa-se a
perda de edição humana com uma vista de leitura derivada (`ktu memory show`/export Markdown),
que **não** é fonte.

**Contrato herdado do `knudge-core` (não negociável):**

| Propriedade | Regra adoptada |
|---|---|
| **ID endereçado por conteúdo** | `<tipo>_<base36(8)>` = hash(`type` + `U+001F` + `normalize(statement)`); reclassificar o tipo **não** reescreve o `id` |
| **Contrato de bytes canónico** | ordem de campos fixa, enums fechados, opcionais **omitidos** (nunca `null`), normalização explícita — com *goldens* |
| **A nota é a verdade** | o índice é **derivado e descartável**; reconstruí-lo nunca perde conhecimento |
| **Escrita atómica** | `tmp → fsync → rename` **antes** do evento correspondente; lock *advisory* (CLI + `ktu serve` no mesmo projeto) |
| **JSONL tolerante** | *dedup-on-read*; linha malformada é saltada **com warning**, nunca aborta |
| **Write idempotente por conteúdo** | duas fases: *recall* → decisão `<0.75` cria, `0.75–0.92` **merge**, `≥0.92` **rejeita**; estrito na forma, tolerante na operação; **nada é apagado** |
| **Busca híbrida determinística** | filtros → BM25 → âncoras → **RRF** (+ vetorial e autoridade quando há índice); desempate `(score desc, id asc)`; canal ausente degrada para lexical com `warnings` |
| **Âncoras (`anchor PATH`)** | ligam a nota ao código; repetíveis, aceitam vírgula e glob; `doctor` lista âncoras quebradas |
| **Tarefas com hierarquia fechada** | `epic ⊃ {issue ⊃ task \| task}`; fechar exige **evidência** (`--outcome/--note`) |
| **Ciclo de vida / confiança** | posterior Beta, factor de idade, penalidade por contradição, confiança derivada de tarefas |
| **Portas** | `Clock`, `Rng`, `Env`, `Fs`, `Git`, **`HookRunner`**, `Logger` — o domínio não toca SO; adaptadores isolados |
| **Disciplina de código** | ≤300 linhas/ficheiro, sem `unwrap`/`expect`/`panic`/`unsafe`, iteração determinística (`BTreeMap`/`IndexMap`) |
| **Gatilhos (vindos do MCP do knudge)** | `pre_write`, `pre_edit`, `session_end`, `status` — e os *hints* são **ponteiros** (`id + statement + score`), **nunca o corpo** |

**Adaptação ao ktu (onde não copiamos):**

- **Porta `Memory` com tipos do ktu** (`ktu-memory`): o backend é substituível; o `knudge-core`
  é o **motor de recuperação** (BM25 + âncoras + RRF), consumido através da porta — nunca
  acoplado ao kernel. Tools:
  `memory_ask`, `memory_write`, `memory_task` (new/list/done).
- **Memory gate no loop** (do katu, casando com os gatilhos do knudge): o `pre_write` (dedup forte,
  âncora, single-claim) e o recall de início de sessão correm **no loop**, com capacidade — a
  persistência **não** depende de o modelo decidir chamar a tool. **Hints são ponteiros**:
  `id + statement + score` entram no contexto; o corpo só a pedido explícito (economia de tokens,
  a mesma disciplina do resto do plano).
- **Persistência entre sessões:** o agente **pergunta antes de escrever**; Q3 grava em
  `.agents/memory/notes.jsonl` e Q4 (sessão nova, contexto compactado) recupera-o. `ktu doctor`
  (`memory`) verifica âncoras quebradas e a saúde do índice.
- **Sincronização git:** o commit é **decisão do utilizador** (porta `Git`), como no
  knudge — nunca automático.
- **Risco registado:** validar no início do F4 se o `knudge-core` aceita um corpus JSONL
  directamente; se não, o adaptador projeta os registos → vista de notas do knudge (JSONL
  continua a ser a fonte) e reconstrói o índice.
- **Sessão (runtime, não conhecimento):** `.ktu/sessions/<id>/session.v1.jsonl` + `meta.json` +
  `checkpoint.json`; `ktu sessions list|show|resume`.

### 2.8 Guardrails: a regra nunca é violável (tese herdada do katu, em modo mínimo)

> **Tese (do katu original, mantida):** *enforcement* são **regras determinísticas que decidem sobre
> factos tipados, com evidência — não prompts.* O modelo é **não confiável por construção**: o texto,
> os nomes de tool e os argumentos que ele emite são **entrada externa**, tratada como tal.

**Fronteira de confiança.** Tudo o que vem do provider (deltas, tool-calls, caminhos, comandos,
nomes de ficheiros) entra por um funil único: `Guard::evaluate(facto: &Fact) -> Decision`, onde
`Fact` é tipado (`ToolCall { agent, tool, args, path_resolvido, orçamento_restante, profundidade }`) e
`Decision = Allow | Deny(motivo tipado) | Escalate`. A decisão é tomada **antes de qualquer efeito**
— incluindo subagentes (a filha corre com o seu próprio `Fact`, nunca com o do pai).

**Princípios (não negociáveis):**

1. **Enforcement no código do kernel/tools** — nunca no prompt, nunca no provider, nunca no
   front-end. Prompt *explica*; o código *impede*.
2. **Fail-closed:** tool desconhecida, path não resolvível, orçamento esgotado ou decisão ausente ⇒
   `Deny`. Desconhecido nunca é permissão.
3. **O front-end pode restringir, nunca libertar.** A UI (aprovação humana, modo `--yes`) só pode
   *atrasar* ou *negar*; nenhum front-end transforma `Deny` em `Allow`.
4. **Toda decisão é evidência:** `Allow`/`Deny`/`Escalate` entram no log da sessão com o motivo
   tipado; o modelo vê a negação como `tool_result` (para se adaptar), mas a regra fica de pé.
5. **Caps são estruturais:** bytes de saída, tempo de comando, passos por turno, profundidade de
   subagente e orçamento de contexto são limites do sistema, não pedidos ao modelo.

**Três camadas de guardrail (o controlo nunca depende do modelo):**

1. **Invariantes estruturais — em código, não desactiváveis.** Confinamento de path, capacidade
   do `AgentSpec` (I9), fail-closed, caps de recursos. Um ficheiro de guardrails **nunca** as
   desliga; só pode apertar.
2. **Guardrails declarativos — dados em `.agents/guardrails/*.md`** (2.2). Cada ficheiro é
   **.md estruturado**: frontmatter com a regra máquina (`id`, `version`, `applies_to`, `fact`,
   `decision`, `reason`) e corpo com a **razão** humana e **exemplos adversariais** em blocos
   cercados. Carregados para `Guard` no início da sessão; **parse inválido é fail-closed**
   (recusa arrancar com a regra quebrada, nomeando o ficheiro); `ktu guardrails reload` recarrega por
   **geração** (I12), logado. O documento **é** a regra **e** é o conjunto de casos de teste
   — um só artefacto, sem *drift* entre o que se lê e o que se impõe.
3. **Plugins — runtime** (§2.9), só **adicionam** gates e transformam; nunca libertam.

**Catálogo mínimo (é o *seed* de `.agents/guardrails/`):**

| Regra | Facto tipado | Decisão | Onde | Teste adversarial |
|---|---|---|---|---|
| Capacidade (I9) | `tool` fora do conjunto do `AgentSpec` | `Deny` | tools (T2) + oferta no loop | fixture de tool de escrita numa sessão `explorer` |
| Confinamento de path | path resolvido sai do root da sessão (`..`, absoluto externo, symlink) | `Deny` | tools (T2), antes de abrir | fixture com `../../etc/passwd`, `/etc/passwd`, symlink para fora |
| Escrita dentro do projeto | `write_file`/`edit_file` só dentro do root | `Deny` | tools (T2) | fixture a escrever em `$HOME` |
| Comandos destrutivos | `bash` casa padrões (`rm -rf /`, `mkfs`, `git push --force`, `> /dev/…`) | `Deny` + motivo | tools (T2) | fixture com cada padrão |
| Recursos | saída > cap, timeout, passos > teto, profundidade > 1 | `Deny`/truncar | kernel + tools | fixture de saída gigante, loop infinito |
| Segredos | leitura de `.env*` pelo explorer; padrões de chave em notas de memória | `Deny`/redigir | tools + memória | fixture `read .env` no explorer |
| Invariantes do loop (I1–I9) | qualquer um dos I1–I9 violado | nega terminar o turno | kernel | propriedades sobre o log |

**Fora do catálogo agora ( deliberadamente):** DSL/linguagem de política, catálogo estilo
`katu-policy` com dezenas de regras, modo `ask` interactivo, sandbox de SO (Landlock/seccomp/bwrap).
O tipo `Decision` já reserva o espaço; nada disso entra sem ficheiro em `.agents/guardrails/` e um teste
que o exija.

### 2.9 Plugins: pontos de intercepção em runtime, com hot reload

> **Novidade face ao katu original** (que era fechado): o ktu é **extensível em runtime**, e a
> extensibilidade vive nos **pontos de intercepção do próprio loop** — o utilizador
> intercepta o que o agente faz, no momento exacto em que faz.

**Análise da lista inicial.** Os 8 pontos cobrem o *caminho feliz* (sessão, input, saída do
modelo, acções), mas faltam quatro coisas que o backend faz sempre: o **lado do pedido** ao
modelo (`model.request` — onde vivem o contexto, o system e as tools oferecidas), os **erros**
(`model.error`, `kernel.error`), a **observabilidade das decisões de guardrail** (`guard.decision`,
sem a qual a tese do §2.8 não é auditável) e o **resultado** das acções
(`action.after`, distinto de `action.before`). Acresce o ciclo de vida dos próprios plugins e a
detecção de fim inesperado. Catálogo completo:

| Ponto | Quando dispara | Tipo | Relevância | Fase |
|---|---|---|---|---|
| `session.start` | sessão nova | observe + **transform** (injectar instruções/contexto) | essencial | F1 |
| `session.resume` | recuperação de checkpoint após fim inesperado | observe + transform | alta | F1 |
| `session.end` | fim adequado (flush, resumo, commit de memória) | observe | essencial | F1 |
| `session.crash` | fim inesperado (kill, OOM, erro fatal) | observe | alta | F1 (detecção em F4) |
| `session.compact` | antes e depois da compactação (política de resumo) | observe + transform | alta | F4 |
| `user.input` | input do utilizador, **incluindo steering** | transform + **gate** | essencial | F1 |
| `user.interrupt` | interrupção/cancel do turno | observe + gate (limpeza antes de abortar) | alta | F1 |
| `user.approval` | decisão `Escalate` (aprovação humana) | **gate** (auto-aprovar por regras do utilizador; aprovador externo) | alta | F2 |
| `model.request` | **antes** de enviar: mensagens, system, tools oferecidas | transform | essencial | F1 |
| `model.output` | deltas: texto, thinking, tool-call | observe + transform (redigir, pós-processar) | essencial | F1 |
| `model.finish` | fim do turno: `StopReason`, uso de tokens | observe | alta | F1 |
| `model.error` | erro/retry do provider | observe + gate (política de retry/fallback) | alta | F2 |
| `action.before` | antes do efeito — `read`, `write`, `edit`, `trash`, `exec`, `move/rename`, `subagent`, escrita de memória | **gate** (+transform de argumentos) | essencial | F1 (`read`) → F2 (restantes) |
| `action.after` | depois do efeito — resultado tipado | transform (truncar, redigir, resumir **antes** de entrar no contexto) | alta | F2 |
| `subagent.start` / `subagent.end` | ciclo da sessão-filha | observe | média | F2 |
| `memory.ask` | antes da busca | transform + gate (reescrever query) | alta | F4 |
| `memory.write` | nota criada/actualizada | observe + gate + transform (dedup, enriquecimento) | alta | F4 |
| `memory.task` | ciclo de vida de tarefas | observe | média | F4 |
| `guard.decision` | cada `Allow`/`Deny`/`Escalate`, com motivo tipado | observe (auditoria/alertas — é a **evidência** da tese) | essencial | F1 |
| `loop.warning` | sem progresso/repetição, orçamento a esgotar | observe + gate (parar) | alta | F2 |
| `kernel.error` | erro interno do kernel | observe | alta | F1 |
| `plugin.lifecycle` | plugin carregado, recarregado, errado | observe | essencial (hot reload) | F4.5 |
| `frontend.connect` / `frontend.disconnect` | clientes da porta (§2.3) | observe | média | F3 |

Nota: a sessão-filha é uma sessão normal — emite os seus próprios `session.*`,
`model.*` e `action.*` com o campo `parent` preenchido; os pontos `subagent.*` são a vista do pai.

**Desenho do runtime.**

- **Catálogo tipado em `ktu-core`:** um único `enum HookEvent` (payload tipado, `serde`) é a
  verdade — usado pelo dispatch interno e pelo protocolo externo. Não há dois catálogos.
- **Dois modos de plugin, mesmo catálogo:** (a) **in-process**, trait `Plugin` assíncrona, para
  quem embute o ktu como crate; (b) **script externo** — o padrão recomendado.
- **Padrão de script (recomendação): processo externo, JSON-lines sobre stdio.**
  `.ktu/plugins/<nome>/plugin.toml` declara `entrypoint`, `events` subscritos, `mode`
  (`observe|transform|gate`), `timeout_ms` e se é daemon. O kernel escreve `{id, event, payload}`;
  o plugin responde `{id, verdict}`. **Porquê processo externo antes de WASM:** crashes ficam
  isolados (um plugin com panic não mata o kernel — requisito de robustez), qualquer linguagem
  serve, hot reload é trivial e o binário não carrega VM. WASM fica como opt-in futuro
  (perf/segurança mais fina), nunca como dependência do caminho mínimo.
- **Hot reload:** watcher em `.ktu/plugins/`; recarregar incrementa a **geração**; eventos em
  voo completam na geração anterior (I12); plugin que não arranca é desactivado com
  `plugin.error` e a sessão continua. Nenhuma sessão é derrubada por recarga.
- **Ordem e determinismo:** plugins correm sequencialmente, por ordem alfabética do nome, na thread
  do kernel (T1). Gates encadeiam: cada um pode negar.
- **Integração com os guardrails (§2.8): a ordem é fixa — `guardrail do kernel →
  gates de plugin → execução`.** Plugins **nunca** vêm antes do guardrail estrutural e
  nunca podem transformar `Deny` em `Allow` (I11); podem apenas *adicionar* restrição e
  *transformar* dados. É a extensibilidade sem reabrir o buraco da tese.
- **Falhas:** `gate` em timeout/erro ⇒ `Deny{plugin_timeout}` (**fail-closed**); `observe`/`transform`
  em falha ⇒ passa adiante sem alteração.
- **Registo:** cada dispatch (evento, plugin, veredicto, duração) entra no log da sessão como
  evidência — intercepção sem registo é buraco de auditoria.

**Fora do escopo, deliberadamente:** WASM, dispatch paralelo, API rica de plugin, marketplace,
aprovador interactivo completo (o `Escalate` já existe; a UI de aprovação é front-end).

### 2.10 Contexto e orçamento (a disciplina que o katu aprendeu à força)

> É **a** lição do `STAGING.md` do katu: contexto pequeno → thrashing → o
> modelo relê os mesmos ficheiros → `termination=loop`. E a correcção não foi
> "optimizar a selecção": foi **um teto grande e com um só dono**, mais **fixar a
> instrução corrente**.

- **Orçamento com dono único.** Um `BudgetGate` é o **único** dono do teto de
  contexto (a cicatriz dos "6 donos" é o anti-padrão); a política pode opinar, mas não
  mantém um segundo teto. Atingir o teto é **recusa/compactação**, nunca "cortar
  evidência para caber".
- **Mínimo cru + teto resumido** (`raw_min`/`raw_max`), configurável
  (`behavior.context_budget`), com default **alto** — o erro do katu foi `4096`; mediu-se que a
  tarefa só fecha com ~39–48k tokens de input.
- **Fixar a instrução.** A última mensagem do utilizador é a única unidade que a
  montagem **nunca** deixa cair (reserva o custo antes do resto).
- **Uma só máquina de orçamento** para o cru e para o digest (o katu tinha duas decisões,
  `Q-02b`/`Q-03`, a partilhar a mesma máquina) — no ktu começa como **sufixo + pin**, e
  só muda com A/B.
- **Determinismo de prefixo:** qualquer transformação de contexto guarda um mapeamento
  determinístico original→substituto, ou invalida o cache de prefixo e custa mais do que poupa.

### 2.11 Conhecimento ancorado em `.agents/`, prompt, skills e taint

**Layout de conhecimento (versionável, junto ao `AGENTS.md`) vs. runtime:**

```
.agents/                      # CONHECIMENTO — versionável, editável pelo humano
  skills/*/SKILL.md           # 2.1 — skills, organizadas e consultadas sob demanda
  guardrails/*.md             # 2.2 — regras determinísticas (.md estruturado)
  memory/*.jsonl              # 2.3 — memória (JSONL append-only = fonte da verdade)
.ktu/                         # RUNTIME — estado efémero/operacional
  sessions/<id>/              #   log da sessão, meta, checkpoint
  plugins/                    #   plugins (§2.9)
  trash/                      #   ficheiros apagados (recuperáveis)
  plan/<sessão>.json          #   artefacto do modo plano (§2.13)
```

A separação não é cosmética: `.agents/` é o que o humano **revê e versiona** (skills,
guardrails, memória); `.ktu/` é o que o agente **produz e consome em execução** (sessões,
plugins, lixo, planos).

- **System prompt composto, não improvisado:** base do agente + `AGENTS.md` do projecto +
  catálogo de skills + guia de tools + memória relevante. O `AGENTS.md` é um *router*:

- **System prompt composto, não improvisado:** base do agente + `AGENTS.md` do projecto +
  catálogo de skills + guia de tools + memória relevante. O `AGENTS.md` é um *router*:
  condensar a sintaxe redundante do markdown corta ~29 % dos bytes sem perder ponteiros, de forma
  pura e sem cache (o katu rejeitou cache com o número: 89 µs de cálculo vs 25,8 ms de
  escrita).
- **Skills (`.agents/skills`):** descoberta **fail-open** de `.agents/skills/*/SKILL.md`
  (frontmatter `name`/`description`); catálogo no prompt (nome + primeira frase). A consulta é
  **sob demanda**: o modelo carrega o corpo via uma tool `skill` (ou `read` do próprio ficheiro),
  nunca tudo de uma vez — o catálogo custa pouco, o corpo só entra quando faz falta. Um
  `SKILL.md` malformado é ignorado — **nunca** quebra o arranque.
- **Taint / spotlighting (I13):** o output de tool e todo o conteúdo de ficheiro é **dado
  não confiável**, nunca instrução. O payload viaja num envelope com tags de
  abertura/fecho; o escape **não altera o comprimento** (para o teto de bytes continuar exacto) e
  uma verificação estrutural garante que nenhuma tag crua escapa. É contenção
  **estrutural** — separa dado de instrução; não promete obediência do modelo.

### 2.12 Lacunas herdadas do katu: o que faltava no plano

O katu original falhou por ordem de construção, não por falta de ideias. Vários
subsistemas que ele tinha **são o antídoto directo do problema** que o ktu tem de resolver
(perda de contexto, loop, injecção). Cada um recebe aqui um veredicto explícito —
**adoptar agora**, **deixar o seam** ou **deferir com razão** — para não se repetir o
erro nos dois sentidos (reconstruir tudo antes de funcionar, ou esquecer o que era essencial).

**A. Adoptar agora (crítico para a métrica)**

| Ponto no katu | O que é | Porque é crítico | Onde entra |
|---|---|---|---|
| `context/select` + `pin_unit` + `BudgetGate` | montagem de contexto com orçamento, **um só dono do teto**, e a última mensagem do utilizador nunca sai | o `STAGING.md` do katu mostra que o contexto pequeno causou **thrashing e `termination=loop`**; é literalmente a perda de contexto | nova §2.10 |
| `kernel/project::wire_messages` | **um passo = uma mensagem** `assistant` com N `tool_calls`, seguida de N `tool` results | era um **HTTP 400** real com ≥2 tool calls; sem isto o provider local e o GDK quebram | §2.4 (nota de projecção) |
| `prompt.rs` + `skill.rs` | montagem do system prompt: `AGENTS.md` como router condensado (~29 % menos bytes) + descoberta **fail-open** de skills | sem instruções de projecto o agente não atinge qualidade nenhuma; hoje o plano não diz de onde vem o prompt | nova §2.11 |
| `taint.rs` (spotlighting) | output de tool/ficheiro entra em envelope `untrusted`, com escape que **não muda o comprimento** e verificação estrutural | defesa contra *prompt injection*; par obrigatório da tese de guardrails | I13 + §2.11 |
| `agent.stall` + taxonomia de terminação | detecção de sem-progresso/repetição e `StopReason` fechado | foi o modo de falha observado (reler os mesmos ficheiros em ciclo) | §2.4 + I5 |
| `kernel/memory_gate` | recall/write automáticos no loop, com capacidade e `pre_write` (dedup, âncora) | a persistência não pode depender de o modelo *escolher* chamar a tool | §2.7 |
| `kernel/budget` + `cost/caps` + `prices.toml`/`tiers.toml` | consumo (turnos, tool calls, tokens, tempo) e teto único; atingir o teto é **recusa**, nunca cortar evidência | custo/tokens sem dono é o que faz um agente correr sem fim; e a GDK vai custar dinheiro | §2.10 + guardrail |
| `tools/trash` + `api/trash.rs` | apagar é **mover para lixo recuperável** (com índice) | guardrail barato e forte: o agente nunca destrói irreversivelmente | §2.6 tools + F2 |
| `plan` (artefacto) | tool de planeamento que mantém a lista de tarefas e o contrato de escopo | sem plano persistente o agente perde o fio em tarefas longas (Q3/Q4 da bateria) | F2 tools |

**B. Deixar o seam agora, implementar depois**

| Ponto no katu | O que é | Seam no ktu |
|---|---|---|
| `katu-policy` (facts, rule, glob, paths, argv, approval, decision) | política como **dado** sobre factos tipados; `argv` **analisado** (não regex); decisões de aprovação memorizadas | `Guard::evaluate` usa o **mesmo** tipo de factos, para que um motor de regras entre sem tocar no loop; `Escalate` → aprovação com memória |
| Porta `Memory` + `PreWriteOutcome` | contrato com tipos do ktu; backend substituível (MCP depois); `pre_write` (duplicata forte, âncora, single-claim) | `ktu-memory` implementa uma **porta**, não chama knudge-core directamente |
| `model.rs` + `StepModel`/`PhaseModel` + `Control::SetModel/Thinking` | catálogo de modelos e modelo por passo/fase, controlado pelo **utilizador** em runtime, logado e sobrevivente a `resume` | catálogo no provider; controlo pela porta |
| `retry.rs`, `sse.rs` | política de retry/backoff e parser SSE robusto | F2 provider |
| gramática restringida (GBNF) + structured output | output do llama.cpp forçado por gramática → tool-call JSON fiável | alavanca grande de qualidade local; F2 (opt-in) |
| TOON/colunar + `read/views` + `ToolReport` | economia de tokens: resultado de tool como **ponteiro + delta truncado**, não corpo; views de leitura | `action.after` transforma; decisão plain vs TOON em F4 |
| `kernel/checkpoint` (temp→fsync→rename, schema versionado) + `resume`/`reconcile`/`lock` | durabilidade real e recuperação de crash | F4 (sessão + crash) |
| `api/command|live|handle` + `kernel/control` | comandos **out-of-band** da porta (Continue, SetModel, Trash, compact) além do chat | §2.3 (porta) |
| `[git] versioned` | versionar sessões/notas no git | F4 (opt-in) |
| `evidence` (métrica com base + artefacto) | "nenhum número sem artefacto" | F5/F6 (benchmarks e gate de qualidade) |

**C. Deferir com a razão (a não repetir)**

`audit/` (bin + bloom + índice), `confidence/` + FDR/conformal, `feedback.rs`,
`diag` com catálogo de 122 eventos, selecção de contexto por rate-distortion/submodular,
`prices`/`tiers` completos, plugin ABI/WASM, MCP, providers além de llama.cpp+GDK,
`stats`/`report`, contenção soft, sandbox de SO. **Razão transversal:** cada um só
entra com um consumidor actual e um teste que o exija — exactamente o que o katu fez ao
contrário. Nota de convergência: o próprio `12-plugins-e-abi.md` do katu diz que o *seam*
de controlo é a **política (dado)** e que plugins são **efeito colateral**, nunca regra
— é o que a §2.9 adopta (plugins adicionam gates e transformam, jamais libertam).

### 2.13 Modo plano (plan mode)

- **O que é:** um modo de sessão em que o agente **explora e planeia** e produz um **plano
  estruturado** para tarefas complexas; a execução só começa depois de aprovação.
- **Read-only estrutural.** O `AgentSpec` do modo plano tem só leitura + `plan` (+ `subagent`
  read-only): sem `write_file`/`edit_file`/`bash`/`memory_write`. É a mesma maquinaria de I9 —
  **não** é prompt a pedir contenção.
- **Artefacto do plano:** ficheiro estruturado (JSON) em `.ktu/plan/<sessão>.json` —
  estágios → tarefas (`id`, objetivo, estado, dependências, critério de pronto), escrito
  atomicamente pelo kernel a partir das chamadas da tool `plan`. Espelhado em tasks de memória
  (knudge) para busca/persistência entre sessões.
- **Transição `plan → build` exige aprovação** (`user.approval`/`Escalate`); a troca de modo
  é um comando da porta (`SetMode`), **sempre logado** e sobrevivente a `resume`. O modo nunca
  muda por vontade do modelo.
- **Fan-out no plano:** no modo plano o agente usa **vários `explorer` em paralelo** para mapear o
  terreno antes de escrever o plano.
- **Porquê:** tarefa complexa sem plano perde o fio (Q3/Q4 da bateria); o plano persistente dá ao
  humano um ponto único de revisão e aprovação.

---

## 3. Fases (cada uma termina com um teste que passa)

Regra de ouro: **uma fase não abre antes de a anterior fechar verde**, e nenhuma fase pode “ficar
90% feita” — ou o slice funciona, ou não passou.

### F0 — Fundação (½ dia)
Workspace `ktu` (crates `ktu-core`, `ktu-tools`, `ktu-provider`, `ktu-port`, `ktu-cli`, `ktu-memory`),
edition 2024 / rust 1.99+, lints iguais aos do repo (`clippy.toml` já existe),
`rustfmt.toml`, `Makefile` (`check` = fmt + clippy `-D warnings` + test), README enxuto.
Layout `.agents/` (skills, guardrails, memory) e `.ktu/` (runtime) criados por `ktu init`.
**Saída:** `cargo make check` verde com um crate vazio e 1 teste.

### F0.5 — Destrinchamento dos guardrails (½ dia, antes de escrever o loop)
Define o **schema de `.agents/guardrails/*.md`** (frontmatter máquina + corpo com razão e exemplos
adversariais) e **semeia os defaults** do catálogo mínimo (§2.8) como ficheiros reais. Nenhuma regra
entra no código sem ficheiro correspondente; o conjunto cresce por adição (nunca por remoção
silenciosa). Os exemplos adversariais dos ficheiros **são** os casos de teste.
**Saída:** schema + `seed` de `.agents/guardrails/`, com as invariantes I1–I16 mapeadas uma a uma.

### F1 — Slice vertical “hello agent” (2–3 dias) ⬅ *o único objectivo é isto funcionar*
`ktu-core`: tipos (`Message`, `ToolCall`, `ToolResult`, `Turn`, `StopReason`), log JSONL,
`derive_messages`, loop completo; **fake provider** com fixtures gravadas; **1 tool**
(`read_file`) já atrás do funil `Guard` (capacidade + confinamento de path + caps mínimos);
`ktu-cli`: `ktu run "..."` embedded, stderr em eventos, stdout com a resposta final.
**Pontos de intercepção (§2.9) já existem no F1:** `HookEvent` tipado + dispatcher +
pontos `session.start/end`, `user.input`, `model.request`, `model.output`, `action.before` (em
`read`), `guard.decision`, `kernel.error` — testados com um plugin in-process falso (observe e
transform). Nenhum plugin ainda; os pontos, sim.
Acresce, desde o F1: **montagem do system prompt** (base + `AGENTS.md` condensado), **envelope taint** no output de tool (I13), **BudgetGate** com dono único (I14), projecção de
wire do passo, e detecção de estagnação (`agent.stall`).
testes: I1–I14 com replay; teste de ouro com tool-call e cancel.
**Saída:** `echo 'ktu run "lista os ficheiros de REF/knudge"'` contra o provider fake, com
tool-call real, turno fechado, log verificável. **Ainda não há modelo real — e já funciona.**

### F2 — Provider llama.cpp real + tools de filesystem + subagente explorer (3–4 dias)
`ktu-provider` (llama-server, streaming, tool-calling nativo + fallback texto), `ktu-tools`
(`read_file`, `write_file`, `edit_file`, `list_dir`, `glob`, `grep`, `bash` com timeout/cap),
erros de tool sempre accionáveis, paths resolvidos contra o root da sessão.
Subagentes: `AgentSpec` + registo (primário `ktu` + `explorer`), tool `subagent` (sessão-filha com
contexto fresco, relatório capado, profundidade máx. 1, **fan-out de vários em paralelo** com
*join*), I9 imposta na thread de tools.
**Modo plano:** `AgentSpec` read-only + tool `plan` + artefacto estruturado em `.ktu/plan/`;
transição `plan→build` via aprovação (I15). Guardrails: fecham-se as regras de `bash` destrutivo, escrita só dentro do root e segredos —
o `seed` de `.agents/guardrails/` do F0.5 é **estendido** (facto tipado, decisão, ponto de enforcement, teste
adversarial) antes do gate de qualidade, nunca depois.
Tool `plan` (lista de tarefas persistente). **Trash em vez de apagar** (recuperável, com
índice). Retry/backoff do provider. **Gramática restringida (GBNF)** do llama.cpp como opt-in
para tool-call fiável. Intercepção: `action.before/after` em todas as tools, `user.approval`
(fluxo `Escalate`), `model.error` (gate de retry), `loop.warning`, `subagent.start/end`.
**Saída:** sessão real contra `llama-server` local resolve um bug simples de 2 passos num repo de
teste; o mesmo agente delega um mapeamento de repo ao `explorer` e o relatório volta capado no
`tool_result`, com a sessão-filha em disco; uma tool de escrita pedida pelo explorer é **recusada**.

### F3 — A porta + front-ends (2–3 dias)
`ktu-port` protocolo + `Channel` e `Http`; `ktu serve` (**HTTP/1.1 + HTTP/2** sobre
`hyper`/`axum` em `127.0.0.1`); TUI mínima (ratatui, eventos da porta);
página web de demonstração (fetch+SSE). Toda a UI partilha o mesmo protocolo — nenhum acesso
directo ao kernel.
Intercepção: `frontend.connect`/`frontend.disconnect`; os eventos de plugin e do log
ficam observáveis pela porta (sem acesso directo ao kernel).
**Saída:** `ktu serve` + TUI + web mostram a mesma sessão ao mesmo tempo, sem estado separado.

### F4 — Memória knudge + persistência (3–4 dias)
`ktu-memory` (porta + knudge-core como motor de recuperação) sobre
`.agents/memory/` — `notes.jsonl` (fonte da verdade) + `events/*.jsonl` (auditoria) + `index/`
(derivado). Adoptar **o contrato do knudge** (§2.7): ID endereçado por conteúdo, escrita atómica
antes do evento, JSONL tolerante, write idempotente com limiares `0.75`/`0.92`, busca híbrida
determinística, âncoras, tarefas com hierarquia fechada, portas `Clock/Rng/Env/Fs/Git/HookRunner/Logger`.
3 memory tools (`memory_ask`/`memory_write`/`memory_task`), sessões `resume`/`sessions list|show`,
**compactação** orçamental (resumo dos turnos antigos + cru no disco + âncora),
recall de início de sessão. `ktu doctor --memory` (porta `health` do knudge): âncoras quebradas
e saúde do índice; `ktu memory export --md` para vista humana (derivada).
Intercepção: `memory.ask`/`memory.write`/`memory.task`, `session.compact` (transform do
resumo) e detecção de `session.crash` (heartbeat/checkpoint).
Memory gate (recall/pré-escrita automáticos no loop). Checkpoint tipado e durável
(`temp→fsync→rename`, esquema versionado) + recuperação de crash; `[git] versioned`
opt-in para sessões/notas.
**Saída:** teste de persistência: escreve nota numa sessão, morre o processo, sessão nova recupera
a nota; teste `compacted: true` equivalente ao Q4 da bateria.

### F4.5 — Runtime de plugins + hot reload (3 dias)
Protocolo externo JSON-lines sobre stdio, `plugin.toml` (entrypoint, eventos, modo, timeout, daemon),
watcher de `.ktu/plugins/` com **gerações** (I12), ordem alfabética determinística na
thread do kernel, fail-closed em gates (I11), plugin de exemplo e `ktu plugins list|reload|doctor`.
**Saída:** um plugin script que transforma `user.input` e um gate que nega `bash destrutivo` —
ambos carregados, recarregados a quente a meio de uma sessão viva, sem derrubar turno nem sessão.

### F5 — Gate de qualidade: a bateria repetida com o ktu como sujeito isolado (2–3 dias)
Um sujeito novo na bateria já existente — **não** uma bateria nova. O `xtask quality`
(`scripts/quality.sh`) reproduz o método dos referenciais:

1. **Mesmas perguntas, mesmo contexto:** Q1–Q4 de `p1`–`p4`, tiradas do `frontmatter` de cada
   `pN/pN.md`, correlas contra o mesmo repo-alvo (`REF/knudge`) com que goose/pi/opencode correram.
2. **Isolamento entre sujeitos:** cada subject corre com sessão e memória **privadas**, limpas de
   ruído — id de sessão próprio, `.agents/memory` próprio (cópia limpa), working dir próprio; nenhum
   ficheiro de um subject é lido ou escrito por outro (o `.ktu/sessions/` do ktu nunca é lido pelos
   referenciais, e as memórias dos referenciais nunca entram no contexto do ktu).
3. **Condições registadas:** cada corrida grava o artefacto com o **mesmo esquema de frontmatter**
   (`question_id`, `question_issue`, `expected_answer_ideia`, `provider`, `model`,
   `thinking-level`, `cache`, `same_session`, `compacted`, `agent: ktu`) — as condições do ktu
   ficam transparentes e comparáveis (llama.cpp local vs. `opencode_go/deepseek-v4.1-flash`).
4. **Encadeamento dentro do sujeito:** `same_session: true` corre nas mesmas sessões do ktu; Q4
   (`compacted: true`) corre **depois** de compactação real da sessão.
5. **Persistência entre sessões:** sonda extra, no fim da bateria — mata o processo, abre sessão
   nova e pede para recuperar o que Q3 gravou; passa só se o registo sobreviveu no `.agents/memory/notes.jsonl`.
6. **Veredicto:** julgamento estruturado por questão contra a `expected_answer_ideia`, com o artefacto
   do ktu arquivado **ao lado** dos referenciais (`p1/ktu-p1.md`, e comparações `p1/ktu-goose.md`
   etc. quando úteis). O bar é **equivalência de grau** com goose/pi/opencode — não vitória,
   não derrota: mesma qualidade, mesmas condições declaradas.
**Saída:** `wiki/quality_test/p*/ktu-*.md` + tabela de veredicto por questão, com a sonda de
persistência marcada como passa/falha.

### F6 — Robustez e endurecimento (contínuo, pós-gate)
Adaptador GDK (feature `gdk`), paralelismo opt-in de tools, plugins em WASM (opt-in), dispatch
paralelo de plugins, `ktu providers doctor`, releases, benchmarks (latência por passo, custo de
token, memória), só então discutir optimização profunda.

---

## 4. Testes (o que trava cada invariante)

| Camada | Tipo | Exemplos |
|---|---|---|
| core | unit + replay | `derive_messages` a partir de um log gravado ⇒ bytes iguais; I1/I2 por `session.verify()` |
| core | integração com fake provider | tool-call → resultado → resposta final; cancel a meio de stream; tecto de passos |
| provider | integração com test double HTTP | SSE fragmentado, tool-call nativa, tool-call em texto, erro 429/timeout |
| tools | integração em `tmpfs` | cada tool em caminho válido/inválido, saída excessiva truncada, timeout do `bash` |
| porta | integração | dois clientes (TUI/web) vêm os mesmos eventos; cancel pelo socket reflectido no log |
| guardrails | adversarial (fixtures) | por regra do §2.8: path escapa (`..`, absoluto, symlink), escrita fora do root, `bash` destrutivo, tool de escrita no explorer, saída acima do cap, profundidade > 1 — tudo tem de dar `Deny`, nunca `Allow` |
| subagentes | integração | explorer recusa tool de escrita (I9); profundidade máx. 1; filha com contexto fresco; relatório capado; log da filha em disco e retomável por `session_id` |
| plano | integração | modo plano nega escrita (I15); `plan→build` sem aprovação = recusa; artefacto `.ktu/plan/*.json` é válido, atómico e sobrevive a `resume`; tarefas espelhadas na memória |
| memória (knudge) | conformidade + propriedade | IDs endereçados por conteúdo (≡ hash); _goldens_ de bytes; write idempotente nos três regimes; rebuild do `index/` é byte-idêntico e não perde nada; JSONL tolerante (linha malformada = skip+warning); desempate `(score desc, id asc)` |
| guardrails declarativos | carga + adversarial | `.agents/guardrails/*.md` válidos carregam; **um ficheiro quebrado impede o arranque a nomeá-lo** (I16); ficheiro que tenta desligar confinamento de path é ignorado; os exemplos adversariais do ficheiro são executados como casos |
| subagentes (fan-out) | integração | N `explorer` em paralelo com *join*; caps de simultaneidade; ordem de resultados determinística no log |
| plugins | integração | fake plugin observa e transforma (F1); gate de plugin nega acção com evidência no log; plugin que tenta libertar um `Deny` do kernel é ignorado (I11); timeout de gate = `Deny`; plugin que panifica não mata o kernel; hot reload a meio de turno mantém a sessão (I12) |
| ponta-a-ponta | `xtask quality` | bateria p1–p4 replicada com o ktu isolado (llama-server real) + sonda de persistência entre sessões |

Determinismo: `Clock`/`Rng`/`Fs` são **portas** com fakes nos testes (herdado da disciplina do
knudge/katu); zero `unwrap/expect/panic` em `src/`.

---

## 5. Riscos e respostas

| Risco | Prob. | Resposta |
|---|---|---|
| Modelo local fraco em tool-calling (granite-97m é pequeno demais) | alta | usar o modelo de referência (Ornith-1.5-9B Q4_K_M, tool-calling nativo; doctor faz sonda); testes do loop usam fake provider, independentes do modelo |
| Reasoning do modelo consome o tecto de geração / polui a resposta | média | reasoning em campo próprio (não reenviado ao `llama-server`; reenviado como `reasoning_content` em gateways `OpenAI`); tecto de geração acomoda o ` thinking`; fallback de parsing inline |
| Parsing de tool-call divergente entre modelos | média | camada de parsing única + fallback texto; fixture por família de template |
| knudge-core pesado p/ compilar | média | feature `memory` ligada por defeito mas isolada; desligável para builds de teste do loop |
| Compactação comer informação | média | compactação **aditiva** (cru preservado, resumo marcado como derivado); teste de ouro Q3→Q4 |
| Tentação de reintroduzir a policy/audit “já” | alta | este documento: nenhum subsistema sem teste que o exija; revisão de PR contra a lista do §0 |
| knudge-core não indexar corpus JSONL directamente | média | JSONL continua a ser a fonte; o adaptador projeta registos → vista de notas do knudge e reconstrói o índice (validar no início do F4) |

---

## 6. Definição de pronto do projecto (2.ª tentativa)

1. `ktu run` / `ktu serve` / crate / TUI / web operam a **mesma** sessão pela **mesma** porta.
2. Kernel e tools em threads separadas, assíncronas, sem estado partilhado com front-ends.
3. Só llama.cpp built-in; o resto só via GDK (feature).
4. Memória local e persistente em `.agents/memory/*.jsonl` (fonte da verdade), com `ask → write → task` e índice derivado (knudge-core).
5. Um só agente; subagentes **read-only**, com o `explorer` como único habitante inicial — read-only
   provado por teste (I9), não por prompt.
6. Conhecimento ancorado em `.agents/` (skills, guardrails em `.md` estruturado, memória em
   JSONL) com runtime em `.ktu/`; guardrails declarativos fail-closed que nunca desligam os
   estruturais.
7. Subagentes read-only com **fan-out** (vários em paralelo) e **modo plano** read-only, com
   transição aprovada e plano estruturado persistente.
8. Guardrails fortes ao nível do loop: `Guard::evaluate` sobre factos tipados como **único** caminho
   para qualquer efeito; fail-closed; front-end só restringe; toda decisão é evidência no log
   (I10 + catálogo do §2.8, destrinchado em `.agents/guardrails/*.md`).
9. Plugins: catálogo único de pontos de intercepção (§2.9), script externo JSON-lines,
   hot reload por gerações, ordem fixa (kernel → gates de plugin → execução) —
   extensibilidade sem reabrir os guardrails (I11/I12).
10. Bateria `wiki/quality_test/p1`–`p4` repetida com o ktu como sujeito **isolado**, nas mesmas
   condições declaradas, com veredicto de **equivalência de grau** a goose/pi/opencode + persistência
   entre sessões **verde**.
11. Só depois disso: um único documento de optimização, com números.
