# 09 — Exemplos de implementação

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [08 — Boas práticas](./08-boas-praticas.md) · **Seguinte:** [10 — Para IA](./10-para-ia.md)

Receitas completas, do teste à aplicação. O exemplo **compilado no CI** é
[`examples/ktu-embed`](../../examples/ktu-embed/src/main.rs) — usa-o como ponto de partida.

---

## 1. Teste determinístico com fakes (sem modelo, sem disco)

```rust
use ktu_core::agent::{AgentRegistry, AgentSpec};
use ktu_core::budget::BudgetGate;
use ktu_core::cancel::CancelToken;
use ktu_core::domain::{Completion, StopReason, ToolCall, ToolCallId, ToolSpec};
use ktu_core::facade::Ktu;
use ktu_core::guard::AgentId;
use ktu_core::ports::{FixedClock, MemFs, NoopLogger, NoopMetrics, Timestamp};
use ktu_core::session::SessionLog;
use ktu_core::store;
use ktu_core::testing::{FakeProvider, FakeTools};
use ktu_core::tools::ToolOutcome;

#[tokio::test]
async fn dois_turnos_com_o_mesmo_script_dao_o_mesmo_log() -> Result<(), Box<dyn std::error::Error>> {
    let script = vec![
        Completion {
            text: String::new(),
            reasoning: String::new(),
            tool_calls: vec![ToolCall {
                id: ToolCallId::new("1"),
                name: "read_file".to_owned(),
                arguments: r#"{"path":"a.txt"}"#.to_owned(),
            }],
        },
        Completion { text: "li a.txt".to_owned(), reasoning: String::new(), tool_calls: vec![] },
    ];
    let mut logs = Vec::new();
    for _ in 0..2 {
        let fs = MemFs::new();
        let clock = FixedClock::new(Timestamp::from_millis(0));
        let provider = FakeProvider::new(script.clone());
        let tools = FakeTools::new(
            vec![ToolSpec { name: "read_file".to_owned(), description: String::new(), parameters: "{}".to_owned() }],
            vec![ToolOutcome::ok("conteudo")],
        );
        let registry = AgentRegistry::new(
            AgentSpec::new(AgentId::Primary, ["read_file".to_owned()], 1), None);
        let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
        let cancel = CancelToken::new();

        let ktu = Ktu::builder()
            .root("/projeto").fs(&fs).clock(&clock).logger(&NoopLogger).metrics(&NoopMetrics)
            .provider(&provider).tools(&tools).budget(&budget).cancel(&cancel).registry(&registry)
            .build()?;

        let mut log = SessionLog::new();
        let outcome = ktu.turn(&mut log, "le a.txt").await?;
        assert_eq!(outcome.reason, StopReason::EndTurn);
        assert_eq!(outcome.text, "li a.txt");
        assert_eq!(log.verify(), Vec::new());          // I1/I2
        logs.push(store::to_jsonl(&log)?);
    }
    assert_eq!(logs.first(), logs.last());             // I6: replay determinístico
    Ok(())
}
```

---

## 2. Aplicação real (adaptadores + um turno)

Base: [`examples/ktu-embed/src/main.rs`](../../examples/ktu-embed/src/main.rs) — mostra o caminho
completo com `StdFs`, `SystemClock`, `StderrLogger`, `ToolThread`, `LlamaProvider` e a fachada.

```bash
cargo run -p ktu-embed -- --backend scripted "explica o que fazes"   # sem modelo
cargo run -p ktu-embed -- --base-url http://127.0.0.1:8080 --model local "olá"
```

Esqueleto para copiar (o essencial, sem argumentos). **Liga cada dependência a uma variável**
antes da cadeia: `&T::new()` dentro da cadeia cria um temporário que morre no fim da instrução
(`E0716: temporary value dropped while borrowed`).

```rust
fn main() -> std::process::ExitCode {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    let Ok(runtime) = runtime else { return std::process::ExitCode::FAILURE };
    match runtime.block_on(turn()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => { /* escrever em stderr e devolver FAILURE */ }
    }
}

async fn turn() -> Result<(), String> {
    let root = std::path::PathBuf::from(".");
    ensure_layout(&root)?;                       // a porta Fs não cria directórios
    let fs = ktu_tools::StdFs;
    let clock = SystemClock::new();
    let tool_thread = ktu_tools::ToolThread::spawn(|ctx| {
        let fs = ktu_tools::StdFs;
        let clock = SystemClock::new();
        let toolbox = ktu_tools::Toolbox::new(".", &fs, &clock, None);
        ctx.serve(&toolbox);
    }).map_err(|error| error.to_string())?;
    let provider = ktu_provider::LlamaProvider::new(ktu_provider::LlamaConfig::new(
        ktu_provider::DEFAULT_BASE_URL, "local",
    )).map_err(|error| error.to_string())?;
    let logger = StderrLogger;
    let metrics = ktu_core::ports::NoopMetrics;
    let registry = ktu_tools::default_registry();
    let plan_spec = ktu_tools::plan_mode_spec();
    let budget = ktu_core::budget::BudgetGate::new(ktu_core::facade::Ktu::CONTEXT_BUDGET_TOKENS);
    let cancel = ktu_core::cancel::CancelToken::new();

    let ktu = ktu_core::facade::Ktu::builder()
        .root(&root).fs(&fs).clock(&clock).logger(&logger).metrics(&metrics)
        .provider(&provider).tools(&tool_thread)
        .budget(&budget).cancel(&cancel)
        .registry(&registry).plan_spec(&plan_spec)
        .build().map_err(|error| error.to_string())?;

    let mut log = ktu.session("default").map_err(|e| e.to_string())?.unwrap_or_default();
    let outcome = ktu.turn(&mut log, "olá").await.map_err(|e| e.to_string())?;
    ktu.save_session("default", &log).map_err(|e| e.to_string())?;
    let _ = outcome;
    Ok(())
}
```

---

## 3. Serviço com uma sessão por pedido (cancelamento de fora-de-banda)

```rust
struct Sessao {
    ktu: Ktu<'static>,          // ver nota sobre tempos de vida
    log: SessionLog,
    cancel: CancelToken,
    steering: Arc<BufferSteering>,
}

impl Sessao {
    async fn turno(&mut self, prompt: &str) -> Result<TurnOutcome, Error> {
        self.cancel = CancelToken::new();            // um token NOVO por turno
        self.ktu.turn(&mut self.log, prompt).await
    }
}

// de fora (outra thread / handler HTTP):
sessao.cancel.cancel();                          // fecha com StopReason::Cancelled
sessao.steering.push("afinal faz X".to_owned()); // entregue no início do próximo passo
```

Notas:

- Para um servidor com **N sessões**, mantém um actor/tarefa por sessão (como o `ktu-cli` faz com
  `SessionHub`): serializa os turnos e evita dois escritores no mesmo JSONL.
- `Ktu<'static>` exige que as portas vivam “para sempre” (ex.: um `OnceLock<Adaptadores>` ou um
  `Arc` guardado numa estrutura de topo). Se não quiseres essa ginástica, usa
  `std::thread::scope` por sessão (receita 4) — a fachada empresta as portas.

---

## 4. Várias sessões em paralelo (sem `'static`)

`Ktu<'a>` **empresta** as portas, logo `tokio::spawn` não é o encaixe natural. Usa
`std::thread::scope` (uma thread por sessão, cada uma com o seu `Ktu`) e mapeia os erros para um
tipo comum:

```rust
let prompts = vec!["tarefa A", "tarefa B", "tarefa C"];
let resultados: Vec<Result<StopReason, String>> = std::thread::scope(|scope| {
    let handles: Vec<_> = prompts
        .into_iter()
        .map(|prompt| {
            scope.spawn(move || -> Result<StopReason, String> {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| error.to_string())?;
                let fs = StdFs;
                let clock = SystemClock::new();
                let logger = StderrLogger;
                let metrics = NoopMetrics;
                let provider = LlamaProvider::new(LlamaConfig::new("http://127.0.0.1:8080", "local"))
                    .map_err(|error| error.to_string())?;
                let tools = ToolThread::spawn(|ctx| {
                    let fs = StdFs;
                    let clock = SystemClock::new();
                    ctx.serve(&Toolbox::new(".", &fs, &clock, None));
                })
                .map_err(|error| error.to_string())?;
                let registry = default_registry();
                let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
                let cancel = CancelToken::new();
                let ktu = Ktu::builder()
                    .root(".").fs(&fs).clock(&clock).logger(&logger).metrics(&metrics)
                    .provider(&provider).tools(&tools)
                    .budget(&budget).cancel(&cancel).registry(&registry)
                    .build()
                    .map_err(|error| error.to_string())?;
                let mut log = SessionLog::new();
                runtime
                    .block_on(ktu.turn(&mut log, prompt))
                    .map(|outcome| outcome.reason)
                    .map_err(|error| error.to_string())
            })
        })
        .collect();
    handles
        .into_iter()
        .map(|handle| handle.join().unwrap_or_else(|_| Err("thread morreu".to_owned())))
        .collect()
});
```

Regras: **um id de sessão por tarefa** (nunca o mesmo), um `Ktu` por tarefa, e resultados
recolhidos por índice (ordem determinística).

---

## 5. Memória durável + canal semântico opcional

```rust
use ktu_core::MemoryConfig;
use ktu_memory::JsonlMemory;
use ktu_provider::HttpEmbedder;

// 1. config opcional (schema fechado; ausente = lexical)
let config = match fs.read(&MemoryConfig::path(&root)) {
    Ok(bytes) => MemoryConfig::parse(&String::from_utf8_lossy(&bytes))?,
    Err(_) => MemoryConfig::default(),
};

// 2. o embedder tem de viver mais que a memória
let embedder = if config.embeddings.enabled() {
    Some(HttpEmbedder::from_config(&config.embeddings).map_err(|e| e.to_string())?)
} else {
    None
};

// 3. T1: recall no arranque da sessão e nota de fim de turno
let mut memory = JsonlMemory::new(&fs, &clock, &root, root.join(".agents/memory"));
if let Some(embedder) = &embedder {
    memory = memory.with_embeddings(embedder, config.embeddings.clone());
}

// 4. T2: as tools de memória (memory_ask/memory_write)
let tool_thread = ToolThread::spawn({
    let root = root.clone();
    let config = config.clone();
    move |ctx| {
        let fs = StdFs;
        let clock = SystemClock::new();
        let embedder = HttpEmbedder::from_config(&config.embeddings).ok();
        let mut memory = JsonlMemory::new(&fs, &clock, &root, root.join(".agents/memory"));
        if let Some(embedder) = &embedder {
            memory = memory.with_embeddings(embedder, config.embeddings.clone());
        }
        ctx.serve(&Toolbox::new(&root, &fs, &clock, Some(&memory)));
    }
})?;

// 5. fachada
let ktu = Ktu::builder()/* … */.memory(&memory).build()?;
```

Fora do processo, o ciclo de vida da memória é da CLI/`ktu_memory`:
`ktu memory drain --status|--digest|--force`, `ktu memory drain service --install` (worker),
`ktu memory project` (projeta o log em notas derivadas) e `kd ai memory|session-log` (consulta).

---

## 6. Modo plano com aprovação (I15)

```rust
use ktu_core::plan::{Approval, FsPlanHost, Mode};

// a) entrar em modo plano (restringir é sempre permitido)
log.set_mode(Mode::Plan);

// b) o toolbox tem de servir a tool `plan`
let tool_thread = ToolThread::spawn({
    let root = root.clone();
    move |ctx| {
        let fs = StdFs;
        let clock = SystemClock::new();
        let toolbox = Toolbox::new(&root, &fs, &clock, None)
            .with_plan(FsPlanHost::new(&fs, root.join(".ktu/plan"), "s1"));
        ctx.serve(&toolbox);
    }
})?;

// c) a fachada exige uma spec read-only (ligada a variável!)
let plan_spec = plan_mode_spec();
let ktu = Ktu::builder()/* … */.plan_spec(&plan_spec).build()?;

// d) sair do modo plano exige aprovação explícita do utilizador
match Mode::transition(Mode::Plan, Mode::Build, Approval::Approved) {
    Ok(modo) => log.set_mode(modo),
    Err(recusa) => eprintln!("transicao recusada: {recusa}"),
}
```

O artefacto fica em `.ktu/plan/<sessão>.json` (`plan::{load, save, render, parse_action}`).

---

## 7. Plugins e *hooks*

```rust
use ktu_core::plugin::{HookPayload, HookVerdict, Hooks, NoHooks};
use ktu_plugin::PluginHost;

let plugins = PluginHost::load(root.join(".ktu/plugins"));
for error in plugins.errors() {
    // um plugin mau é DESACTIVADO, não derruba a sessão
    logger.log(&ktu_core::ports::LogRecord {
        level: ktu_core::ports::Level::Warn,
        event: ktu_core::events::PLUGIN_LIFECYCLE,
        message: "plugin desativado",
        fields: &[("erro", error.as_str())],
    });
}
let ktu = Ktu::builder()/* … */.hooks(&plugins).build()?;
```

Um *hook* próprio (Rust, sem processo externo):

```rust
struct Auditar;

#[async_trait::async_trait]
impl Hooks for Auditar {
    async fn dispatch(&self, payload: &mut HookPayload) -> HookVerdict {
        if let HookPayload::UserInput { text } = payload {
            *text = text.trim().to_owned();       // transformar é permitido
        }
        HookVerdict::pass(1)                      // `HookVerdict { allowed, reason, generation, transforms }`
    }
    fn generation(&self) -> u32 { 1 }
}
```

Sem plugins, passa `&NoHooks` (ou omite `.hooks(...)`).

---

## 8. Subagentes (opcional)

O `SubagentHost` corre uma sessão-filha. Truque útil: como o `KtuBuilder` recebe um `AgentRegistry`,
constrói o filho com uma registry cujo **primário é a spec read-only** (o `explorer`):

```rust
#[async_trait::async_trait]
impl SubagentHost for MeusSubagentes {
    async fn run(&self, request: SubagentRequest) -> Result<SubagentReport, String> {
        let spec = self.registry.get_by_name(&request.agent)
            .ok_or_else(|| format!("subagente desconhecido: {}", request.agent))?;
        // filho: primário = spec read-only; sem memória/hooks/subagentes (profundidade 1)
        let registry = AgentRegistry::new(spec.clone(), None);
        let ktu = Ktu::builder()
            .root(&self.root).fs(&self.fs).clock(&self.clock)
            .logger(&self.logger).metrics(&self.metrics)
            .provider(self.provider).tools(self.tools)
            .budget(self.budget)                 // o MESMO BudgetGate (I14)
            .cancel(self.cancel)
            .registry(&registry)
            .build().map_err(|error| error.to_string())?;
        let mut log = SessionLog::new();
        let outcome = ktu.turn(&mut log, &request.prompt).await.map_err(|e| e.to_string())?;
        let session_id = request.session_id.unwrap_or_else(|| "filha".to_owned());
        ktu.save_session(&session_id, &log).map_err(|e| e.to_string())?;
        Ok(SubagentReport { session_id, text: outcome.text })
    }
}
```

Se **não** implementares, não passes `.subagents(...)`: a tool `subagent` não é oferecida.

---

## 9. Testar a configuração do projecto (guardrails/permissões)

```rust
#[test]
fn guardrail_quebrado_impede_o_arranque() {
    let fs = MemFs::new()
        .with_dir("/projeto/.agents/guardrails")
        .with_file("/projeto/.agents/guardrails/x.md", "sem frontmatter\n");
    let error = Ktu::builder()/* portas fake */.root("/projeto").fs(&fs)/* … */.build();
    let message = error.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(message.contains("x.md"));            // I16: nomeia o ficheiro
}

#[test]
fn permissao_invalida_impede_o_arranque() {
    let fs = MemFs::new().with_file("/projeto/.agents/permissions.toml",
        "[permissions]\nbash = \"talvez\"\n");
    assert!(Ktu::builder()/* … */.fs(&fs)/* … */.build().is_err());
}
```

---

## 10. Correr o exemplo do repositório

```bash
cargo run -p ktu-embed -- --backend scripted "explica o que fazes"      # sem modelo
cargo run -p ktu-embed -- --root /tmp/proj --session s1 "primeiro"      # grava a sessão
cargo run -p ktu-embed -- --root /tmp/proj --session s1 "segundo"       # retoma
cargo run -p ktu-embed -- --mode plan "planeia a migração"              # read-only
```

Próximo: [10 — Para IA](./10-para-ia.md).
