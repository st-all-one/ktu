# 05 — Portas e adaptadores: o que injectar e como escrever o que falta

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [04 — Sessões](./04-sessoes.md) · **Seguinte:** [06 — Projecto](./06-projecto.md)

## 1. Mapa completo

| Porta (`ktu_core::…`) | Obrigatória | Adaptador pronto | *Fake* para testes |
|---|---|---|---|
| `ports::Fs` | ✅ | `ktu_tools::StdFs` | `ports::MemFs` |
| `ports::Clock` | ✅ | **escreve o teu** (§3.1) | `ports::FixedClock`, `ports::SeqClock` |
| `ports::Logger` | ✅ | **escreve o teu** (§3.2) | `ports::NoopLogger`, `ports::RecordingLogger` |
| `ports::Metrics` | ✅ | `ports::NoopMetrics` | `ports::NoopMetrics`, `ports::RecordingMetrics` |
| `provider::Provider` | ✅ | `ktu_provider::LlamaProvider` | `testing::FakeProvider` |
| `tools::ToolExecutor` | ✅ | `ktu_tools::{ToolThread, Toolbox}` | `testing::FakeTools` |
| `memory::Memory` | opcional | `ktu_memory::JsonlMemory` | — (define o teu, §3.5) |
| `plugin::Hooks` | opcional | `ktu_plugin::PluginHost` / `plugin::NoHooks` | `plugin::Dispatcher` |
| `subagent::SubagentHost` | opcional | **escreve o teu** (§3.4) | — |
| `agent::Steering` | opcional | **escreve o teu** (§3.3) | — |
| `ports::Embedder` | opcional (H8) | `ktu_provider::HttpEmbedder` | `ports::FakeEmbedder`, `ports::UnavailableEmbedder` |
| `ports::Instructions` | opcional | **escreve o teu** (§3.7) | `ports::MapInstructions` |
| `ports::{Env, Rng}` | **não usadas pela fachada** | `ports::MapEnv`, `ports::SeqRng` | — |

> **Porque faltam alguns adaptadores reais:** `Clock`, `Logger`, `SubagentHost`, `Steering` e
> `Instructions` reais vivem na `ktu-cli` (um binário, não publicável). A fachada que os constrói
> está adiada; até lá, são ~30 linhas copiáveis (abaixo). Os *fakes* do núcleo estão todos em
> `ktu_core::ports::{…}` e `ktu_core::testing::{…}`.

## 2. Adaptadores prontos (detalhe)

### 2.1 `StdFs` — sistema de ficheiros real

```rust
use ktu_tools::StdFs;
let fs = StdFs;                     // unit struct; implementa ktu_core::ports::Fs
```

Cobre `read`/`write`/`exists`/`read_dir`/`canonicalize`/`is_dir`/`rename`/`sync`. **Não cria
directórios** (`create_dir_all` não faz parte da porta): quem embute usa `std::fs::create_dir_all`
na borda.

### 2.2 `LlamaProvider` — modelo via `llama-server`

```rust
use ktu_provider::{LlamaConfig, LlamaProvider, ReasoningEffort, Flavor};

let mut config = LlamaConfig::new("http://127.0.0.1:8080", "local");
config.effort = ReasoningEffort::High;          // Low|Medium|High|Xhigh (default)
config.timeout = Duration::from_secs(600);
let provider = LlamaProvider::new(config)?;     // Err(ProviderError::Unavailable) se o cliente falhar
```

Campos úteis de `LlamaConfig`: `effort`, `sampling` (`Sampling`), `max_attempts` (4) +
`retry_backoff` (1 s) para erros transitórios, `timeout` (600 s), `api_key` (gateway remoto),
`session`/`session_header`/`affinity_headers` (afinidade/cache), `flavor`
(`Flavor::Llama` = extensões llama.cpp; `Flavor::OpenAi` = só campos OpenAI).
`ktu_provider::DEFAULT_BASE_URL` = `http://127.0.0.1:8080`.

### 2.3 `ToolThread` + `Toolbox` — as tools na T2

```rust
use ktu_tools::{ToolThread, Toolbox};
use ktu_core::plan::FsPlanHost;

let tool_thread = ToolThread::spawn({
    let root = root.clone();
    move |ctx| {
        let fs = StdFs;
        let clock = SystemClock::new();                    // adaptadores da T2
        let mut toolbox = Toolbox::new(&root, &fs, &clock, Some(&memory));
        if plan_mode {
            toolbox = toolbox.with_plan(FsPlanHost::new(&fs, root.join(".ktu/plan"), session));
        }
        ctx.serve(&toolbox);
    }
})?;
```

- `ToolThread::spawn` devolve `io::Result<ToolThread>`; o *handshake* das specs acontece antes de
  retornar (se a thread morrer, tens erro em vez de silêncio).
- `Toolbox::new(root, &dyn Fs, &dyn Clock, Option<&dyn Memory>)`: sem memória, as tools
  `memory_ask`/`memory_write` não são oferecidas.
- Constrói os adaptadores **dentro** do fecho (é o que a CLI faz): a T2 tem os seus.
- `ktu_tools::{default_registry(), plan_mode_spec(), read_only_tool_names()}` dão os `AgentSpec`
  prontos (`primary` com todas as tools + `subagent`; `explorer` só-leitura; plano read-only+`plan`).

### 2.4 `JsonlMemory` — memória durável (knudge)

```rust
use ktu_memory::JsonlMemory;
let memory = JsonlMemory::new(&fs, &clock, &root, root.join(".agents/memory"));
// com embeddings (H8): memory = memory.with_embeddings(&embedder, embeddings.clone());
```

Implementa `ktu_core::memory::Memory` (`recall`, `write`). Ver [06](./06-projecto.md) §6 para o
formato dos ficheiros e [09](./09-exemplos.md) §5 para o ciclo completo.

### 2.5 `PluginHost` — plugins externos

```rust
use ktu_plugin::PluginHost;
let plugins = PluginHost::load(root.join(".ktu/plugins"));   // nunca falha: plugins maus são desactivados
for error in plugins.errors() { logger.log(&LogRecord { level: Level::Warn, event: events::PLUGIN_LIFECYCLE, message: "plugin desativado", fields: &[("erro", error.as_str())] }); }
```

Sem plugins, passa `&ktu_core::plugin::NoHooks` (ou omite `.hooks(...)`).

### 2.6 `HttpEmbedder` — embeddings locais (opcional)

```rust
use ktu_core::MemoryConfig;
use ktu_provider::HttpEmbedder;

let config = MemoryConfig::parse(&fs.read(&MemoryConfig::path(&root))? …)?;
let embedder = HttpEmbedder::from_config(&config.embeddings)?;   // Err(EmbedError::Invalid) se o endpoint não for http://
```

## 3. Escrever os adaptadores que faltam

### 3.1 `Clock` (obrigatório)

```rust
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use ktu_core::ports::{Clock, Timestamp};

#[derive(Debug)]
struct SystemClock { start: Instant }

impl SystemClock {
    #[allow(clippy::disallowed_methods, reason = "adaptador de borda: base monótona real")]
    fn new() -> Self { Self { start: Instant::now() } }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods, reason = "adaptador de borda: relógio de parede real")]
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |e| e.as_millis());
        Timestamp::from_millis(i64::try_from(millis).unwrap_or(i64::MAX))
    }

    #[allow(clippy::disallowed_methods, reason = "adaptador de borda: relógio monótono real")]
    fn monotonic_micros(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_micros()).unwrap_or(u64::MAX)
    }
}
```

> É o **único** sítio onde leres o relógio real: o domínio recebe só `Timestamp`. Num teste, usa
> `FixedClock::new(Timestamp::from_millis(0))` e o resultado é determinístico.

### 3.2 `Logger` (obrigatório)

```rust
use std::io::{self, Write};
use ktu_core::ports::{Level, LogRecord, Logger};

struct StderrLogger;

impl Logger for StderrLogger {
    fn log(&self, record: &LogRecord<'_>) {
        let level = match record.level {
            Level::Error => "error", Level::Warn => "warn", Level::Info => "info",
            Level::Debug => "debug", Level::Trace => "trace",
        };
        let mut line = format!("[{level}] {}: {}", record.event, record.message);
        for (key, value) in record.fields {
            line.push(' '); line.push_str(key); line.push('='); line.push_str(value);
        }
        let _ignored = writeln!(io::stderr().lock(), "{line}");   // stdout é DADO
    }
}
```

Regras da porta: **nunca** escrever em `stdout`, **nunca** panicar, **nunca** incluir segredos nem
corpo de ficheiro. O `event` é um id estável (`ktu_core::events::*`) — usa-o para métricas e alertas.

### 3.3 `Steering` (opcional)

```rust
use std::sync::{Mutex, PoisonError};
use ktu_core::agent::Steering;

struct BufferSteering { pendentes: Mutex<Vec<String>> }

impl BufferSteering {
    fn push(&self, text: String) {
        self.pendentes.lock().unwrap_or_else(PoisonError::into_inner).push(text);
    }
}

impl Steering for BufferSteering {
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.pendentes.lock().unwrap_or_else(PoisonError::into_inner))
    }
}
```

O kernel chama `take()` no início de cada passo; a mensagem é logada antes de entrar no contexto
(`Event::Steer` + `turn.steer`).

### 3.4 `SubagentHost` (opcional)

```rust
#[async_trait::async_trait]
pub trait SubagentHost: Send + Sync {
    async fn run(&self, request: SubagentRequest) -> Result<SubagentReport, String>;
}
```

`SubagentRequest { agent, description, prompt, session_id }` →
`SubagentReport { session_id, text }`. Se implementares, a tool `subagent` passa a ser oferecida ao
modelo; o relatório é capado por `LoopConfig::subagent_report_chars` (8192). A CLI faz isto com
uma sessão-filha + `AgentSpec` do `explorer` (profundidade máxima 1). Se **não** implementares,
não passes `.subagents(...)` — a tool simplesmente não existe.

### 3.5 `Memory` (opcional)

```rust
fn recall(&self, query: &str, limit: usize) -> MemoryResult<Vec<Hint>>;
fn write(&self, draft: &MemoryDraft) -> MemoryResult<MemoryWrite>;
```

`Hint { id, statement, score }` são **ponteiros** (o corpo nunca entra no prompt).
`MemoryDraft { kind, statement (≤120 escalares), body, tags, anchors }`;
`MemoryWrite::{Created{id}, Merged{id,score}, Unchanged{id}, Rejected{id,score}}` (dedup
`<0.75` cria · `0.75–0.92` funde · `≥0.92` rejeita). `MemoryError::{Io, Schema, Unavailable}`.

### 3.6 `Provider` e `ToolExecutor` (obrigatórios)

```rust
#[async_trait::async_trait]
impl Provider for OMeu {
    async fn complete(&self, request: &ProviderRequest) -> Result<Completion, ProviderError> {
        // request: system, messages, tools, cancel
        Ok(Completion { text: "…".into(), reasoning: String::new(), tool_calls: vec![] })
    }
}

#[async_trait::async_trait]
impl ToolExecutor for AsMinhas {
    fn specs(&self) -> Vec<ToolSpec> { vec![ToolSpec { name: "…".into(), description: "…".into(), parameters: "{}".into() }] }
    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        ToolOutcome::ok("resultado")      // erros de tool são ToolOutcome::error(..), não panics
    }
}
```

Um `Completion` traz `text` **e/ou** `tool_calls`; devolver os dois é aceite (o texto entra no log
como mensagem do assistente). `ProviderError::Cancelled` faz o turno fechar com
`StopReason::Cancelled`.

### 3.7 `Instructions` — instruções locais na leitura

Anexa o `AGENTS.md` mais próximo ao resultado de `read_file`/`edit_file` (uma vez por turno e por
fonte). O adaptador da CLI usa `agents_md::nearest` sobre `StdFs`:

```rust
use ktu_core::agents_md;
use ktu_core::ports::{Instructions, Local};

struct LocalInstructions { root: PathBuf }

impl Instructions for LocalInstructions {
    fn nearest(&self, path: &str) -> Option<Local> {
        let fs = StdFs;                                   // o teu `Fs` real
        let doc = agents_md::nearest(&fs, &self.root, &self.root.join(path))?;
        Some(Local { source: agents_md::label(&self.root, &doc.path), text: doc.text })
    }
}
// … .instructions(&local)
```

> Sem a porta, nada é anexado (comportamento anterior). Se quiseres compor as instruções no
> **arranque** em vez de a pedido, usa `agents_md::discover`/`render` (é o que
> `facade::system_prompt` faz).

## 4. *Fakes* para testes (determinismo, I6)

| Fake | Onde | Nota |
|---|---|---|
| `FixedClock` / `SeqClock` | `ktu_core::ports` | instante fixo / sequência monótona pré-definida |
| `MemFs` | `ktu_core::ports` | `with_file(path, bytes)` / `with_dir(path)` para semear |
| `NoopLogger` / `RecordingLogger` | `ktu_core::ports` | `RecordingLogger` guarda os registos para asserções |
| `NoopMetrics` / `RecordingMetrics` | `ktu_core::ports` | idem para métricas |
| `FakeProvider` | `ktu_core::testing` | *replay* de um script de `Completion`; `tool_counts()` diz quantas tools foram oferecidas |
| `FakeTools` | `ktu_core::testing` | respostas pré-definidas; `recorded_calls()` |
| `FakeEmbedder` / `UnavailableEmbedder` | `ktu_core::ports` | canal semântico determinístico / indisponível |
| `MapEnv` / `SeqRng` | `ktu_core::ports` | se precisares das portas `Env`/`Rng` |
| `MapInstructions` | `ktu_core::ports` | `with_dir(prefixo, fonte, texto)`: instruções locais por prefixo de caminho |

## 5. Checklist

- [ ] Cada porta injectada tem um dono claro (quem a cria e até quando vive).
- [ ] Se injectares `Instructions`, o `AGENTS.md` mais próximo **não** é também composto no arranque
      (evita repetir as mesmas regras no prompt).
- [ ] Adaptadores da T2 construídos **dentro** do fecho do `ToolThread`.
- [ ] `Logger` escreve em `stderr` com ids estáveis; nada de segredos.
- [ ] `Clock` real só na borda; testes com `FixedClock`.
- [ ] Provider com `timeout`/retries razoáveis (o default é 600 s e 4 tentativas).
- [ ] Sem `unsafe`/`unwrap`/`expect`/`panic` nos adaptadores (propaga `Result`).

Próximo: [06 — Projecto](./06-projecto.md).
