# 02 — O construtor: `Ktu::builder()` / `KtuBuilder`

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [01 — Panorama](./01-panorama.md) · **Seguinte:** [03 — Turnos](./03-turnos.md)

## 1. Forma geral

```rust
let ktu = Ktu::builder()          // KtuBuilder<'_> — só campos, zero efeitos
    .root("/projeto")             // obrigatório
    .fs(&fs)                      // obrigatório
    …                             // restantes dependências
    .build()?;                    // Result<Ktu<'_>> — aqui há efeitos de leitura (projecto)
```

- `builder()` não faz nada: é um construtor de dados. **Todos** os efeitos acontecem no `build()`
  (ler guardrails/permissões/`AGENTS.md`) e no `turn()`.
- O builder é `#[must_use]` e consumido por valor: cada setter devolve `Self`, portanto a cadeia
  tem de terminar em `build()`.
- O `Ktu` **empresta** as dependências (`Ktu<'a>`): os adaptadores têm de viver mais que ele.
  Declara-os **antes** do `Ktu` (ver §5).

## 2. Referência dos setters

### 2.1 Obrigatórios

| Setter | Tipo | Porquê |
|---|---|---|
| `.root(dir)` | `impl Into<PathBuf>` | raiz do projecto: `.agents/`, `.ktu/`, `AGENTS.md` |
| `.fs(&dyn Fs)` | porta `Fs` | ler guardrails/permissões/`AGENTS.md` e ler/gravar sessões |
| `.clock(&dyn Clock)` | porta `Clock` | carimbos de sessão e métricas de duração (`instrument!`) |
| `.logger(&dyn Logger)` | porta `Logger` | eventos do kernel — **nunca** silenciados por omissão |
| `.metrics(&dyn Metrics)` | porta `Metrics` | amostras/contadores (`NoopMetrics` é aceitável) |
| `.provider(&dyn Provider)` | porta `Provider` | o modelo (uma chamada devolve texto e/ou tool calls) |
| `.tools(&dyn ToolExecutor)` | porta `ToolExecutor` | execução das tools (T2); com `ToolThread` é o handle |
| `.budget(&BudgetGate)` | dado | **dono único** do tecto de contexto (I14) |
| `.cancel(&CancelToken)` | dado | sinal de cancelamento (I4) que a tua UI vai accionar |
| `.registry(&AgentRegistry)` | dado | agente primário: define as tools e o *enforcement* estrutural |

### 2.2 Opcionais (default: desligado)

| Setter | Efeito quando ausente | Notas |
|---|---|---|
| `.plan_spec(&AgentSpec)` | **modo plano recusado** (I15) | especificação read-only; sem ela `turn` devolve `Error::Invalid` |
| `.user_dir(&Path)` | só a directriz do projecto | directório do utilizador (`~/.config/ktu`) para `SYSTEM.md`/`APPEND_SYSTEM.md` |
| `.memory(&dyn Memory)` | sem recall no arranque, sem `memory_ask`/`memory_write` | injecta o mesmo backend que as tools usam |
| `.hooks(&dyn Hooks)` | sem *dispatch* de plugins | `PluginHost` ou `NoHooks` |
| `.subagents(&dyn SubagentHost)` | a tool `subagent` não é oferecida | se presente, a tool entra no conjunto |
| `.steering(&dyn Steering)` | sem *steering* | mensagens do utilizador entregues em fronteira segura |
| `.instructions(&dyn Instructions)` | sem anexação | `AGENTS.md` mais próximo anexado ao resultado de `read_file`/`edit_file` |
| `.max_steps(n)` | `LoopConfig::default().max_steps` (24) | tecto de passos por turno |

> **Dica:** `plan_spec` é obrigatório na prática. A CLI passa sempre
> `ktu_tools::plan_mode_spec()`; num embedder, passa uma `AgentSpec` só-leitura do teu conjunto
> de tools (é ela que garante que o modo plano não escreve).

## 3. `build()` — o que resolve e o que falha

`build()` faz, por esta ordem:

1. valida as portas obrigatórias (`Error::Invalid("fachada sem …")`);
2. carrega **guardrails** de `<root>/.agents/guardrails/*.md` → erro `guardrails invalidos: …` (I16);
3. carrega **permissões** de `<root>/.agents/permissions.toml` → erro `erro nas permissoes: …`;
4. compõe o **system prompt** (`BASE` + `AGENTS.md` condensado, via `facade::system_prompt`);
5. guarda a `LoopConfig` (`max_steps` + defaults de estagnação/subagentes).

| Situação | Resultado |
|---|---|
| ficheiro de guardrail com frontmatter inválido | `Err(Error::Invalid { message })` — **nomeia o ficheiro** |
| permissão com valor desconhecido (`talvez`) | `Err(Error::Invalid { … })` — nomeia o ficheiro |
| `.agents/guardrails/` inexistente | `Ok` — sem regras |
| `.agents/permissions.toml` inexistente | `Ok` — sem políticas explícitas |
| falta uma porta | `Err(Error::Invalid { message: "fachada sem porta Fs" })` |

## 4. `Ktu` — a superfície pública

| Item | Assinatura | Notas |
|---|---|---|
| `Ktu::CONTEXT_BUDGET_TOKENS` | `usize` = 120 000 | valor de referência para `BudgetGate::new` (≈4 bytes/token) |
| `builder()` | `fn() -> KtuBuilder<'a>` | — |
| `root()` | `&Path` | a raiz resolvida |
| `system()` | `&str` | system prompt efectivo (útil para logs/diagnóstico) |
| `turn(&self, log, prompt)` | `async fn(&mut SessionLog, &str) -> Result<TurnOutcome>` | corre **um** turno; ver [03](./03-turnos.md) |
| `session(&self, id)` | `Result<Option<SessionLog>>` | lê `.ktu/sessions/<id>.jsonl` tolerando a última linha parcial |
| `save_session(&self, id, &log)` | `Result<()>` | grava durável (`temp` → `rename`) |
| `sessions_dir()` | `PathBuf` | `<root>/.ktu/sessions` |

Funções soltas úteis:

| Função | Para quê |
|---|---|
| `facade::system_prompt(&dyn Fs, &Path) -> String` | compor o prompt do projecto **antes** de montar a fachada (ex.: passá-lo a um host de subagentes) |
| `facade::system_prompt_in(&dyn Fs, &Path, Option<&Path>) -> String` | o mesmo, com a directriz do **utilizador** (`user_dir`) |
| `facade::final_text(&SessionLog) -> Option<String>` | resposta final do **turno corrente** (o mesmo que `TurnOutcome.text`) |

## 5. Ownership e tempos de vida (a parte que morde)

`Ktu<'a>` guarda `&'a dyn …` — logo:

```rust
let fs = StdFs;                       // 1. adaptadores primeiro
let clock = SystemClock::new();
let logger = StderrLogger;
let metrics = NoopMetrics;
let tool_thread = ToolThread::spawn(…)?;   // o handle vive até ao fim

let ktu = Ktu::builder().root(&root).fs(&fs)… .tools(&tool_thread)… .build()?;  // 2. fachada
let mut log = ktu.session("s1")?.unwrap_or_default();
let outcome = ktu.turn(&mut log, "…").await?;                                   // 3. turnos
```

Regras práticas:

- **Declarar adaptadores antes do `Ktu`** (o borrow checker garante a ordem de *drop*).
- Um `Ktu` por **sessão/configuração**; reutiliza-o para N turnos (a composição é feita uma vez).
- `turn` recebe `&mut SessionLog` — a sessão é explícita (I3); podes ter vários logs com o mesmo `Ktu`.
- O `ToolThread` pode ser partilhado por vários `Ktu` (é `&dyn ToolExecutor`) desde que os turnos
  não corram em simultâneo no mesmo log.
- Nada é `Clone`: para paralelismo, cria um `Ktu` por tarefa (é barato — só lê o projecto).

## 6. Três perfis de montagem

**a) Mínimo (testes, sem efeitos):**

```rust
// ⚠️ cada dependência tem de estar **ligada a uma variável** antes da cadeia:
// `&T::new()` na cadeia cria um temporário que morre no fim da instrução (E0716).
let fs = MemFs::new();
let clock = FixedClock::new(Timestamp::from_millis(0));
let logger = NoopLogger;
let metrics = NoopMetrics;
let provider = FakeProvider::new(script);
let tools = FakeTools::new(specs, responses);
let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
let cancel = CancelToken::new();
let registry = AgentRegistry::new(AgentSpec::new(AgentId::Primary, names, 1), None);

let ktu = Ktu::builder()
    .root("/projeto").fs(&fs).clock(&clock).logger(&logger).metrics(&metrics)
    .provider(&provider).tools(&tools)
    .budget(&budget).cancel(&cancel).registry(&registry)
    .build()?;
```

**b) Real (CLI-like):**

```rust
let fs = StdFs;                                  // unit structs: ok na cadeia, mas explícito é melhor
let clock = SystemClock::new();
let logger = StderrLogger;
let metrics = NoopMetrics;
let provider = LlamaProvider::new(LlamaConfig::new("http://127.0.0.1:8080", "local"))?;
let budget = BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS);
let cancel = CancelToken::new();
let registry = default_registry();               // ⚠️ temporários têm de ser ligados
let plan_spec = plan_mode_spec();

let ktu = Ktu::builder()
    .root(&root).fs(&fs).clock(&clock).logger(&logger).metrics(&metrics)
    .provider(&provider).tools(&tool_thread)
    .budget(&budget).cancel(&cancel)
    .registry(&registry).plan_spec(&plan_spec)
    .memory(&memory).hooks(&plugins)
    .max_steps(24)
    .build()?;
```

**c) Só leitura (revisão de código, plan mode):** o perfil (b) com `.plan_spec(...)` e
`log.set_mode(Mode::Plan)` antes do turno; o `AgentSpec` read-only faz o resto (I15).

## 7. Checklist

- [ ] Todos os adaptadores declarados **antes** do `Ktu`.
- [ ] `plan_spec` definido (mesmo que não uses o modo plano hoje).
- [ ] **Um** `BudgetGate` para todos os turnos e subagentes.
- [ ] O `CancelToken` é o mesmo que a UI/camada de fora-de-banda vai cancelar.
- [ ] `logger`/`metrics` escolhidos consciente-mente (não deixar logs em silêncio).
- [ ] Directórios de runtime criados pela app antes do primeiro `save_session`.
- [ ] `build()` tratado (não ignorado): um guardrail inválido **tem** de parar o arranque.

Próximo: [03 — Turnos](./03-turnos.md).
