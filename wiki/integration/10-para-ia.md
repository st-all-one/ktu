# 10 — Para agentes de IA (contrato resumido)

> **Público:** agentes/LLMs que escrevem ou revêem código de integração · **Índice:** [README](./README.md) · **Anterior:** [09 — Exemplos](./09-exemplos.md)

Este documento é o **contrato operacional**: regras duras, mapa de símbolos, checklists e
anti-padrões. Se geras código de integração, segue-o à letra; se revês, usa as checklists.

## 1. Regras duras (must / must not)

**MUST**

1. Compor com `Ktu::builder()…build()`; tratar o `Result` do `build()`.
2. Passar **todas** as portas obrigatórias: `root`, `fs`, `clock`, `logger`, `metrics`, `provider`, `tools`, `budget`, `cancel`, `registry`.
3. Passar `.plan_spec(...)` (spec read-only) — mesmo que o modo plano não seja usado hoje.
4. Usar **um** `BudgetGate` (idealmente `BudgetGate::new(Ktu::CONTEXT_BUDGET_TOKENS)`) partilhado com subagentes.
5. Manter o `SessionLog` como fonte da verdade; gravar com `save_session`/`store::save`.
6. Tratar `TurnOutcome.reason` (incl. `Error`/`Cancelled`) e usar `outcome.text` como dado.
7. Criar directórios de runtime na app (a porta `Fs` **não** cria pastas).
8. Declarar os adaptadores **antes** do `Ktu` (ele empresta-os).
9. Escrever logs em `stderr` com ids estáveis (`ktu_core::events::*`).
10. Usar um `CancelToken` **novo por turno**, partilhado com quem cancela.

**MUST NOT**

1. Montar `Deps`/`Ports`/`Capabilities`/`LoopConfig` nem chamar `run_turn` (usa a fachada).
2. Reimplementar leitura de guardrails/permissões/`AGENTS.md`.
3. Escrever/editar `.ktu/sessions/*.jsonl` à mão (append-only via `SessionLog`).
4. Fazer `push` de eventos arbitrários no log (só `ModeSet`/`Steer` via API).
5. Ignorar erros (`let _ = ktu.turn(...)`, `unwrap`, `expect`, `panic`).
6. Misturar dados e logs (`stdout` é dado; `stderr` é log).
7. Meter segredos em logs/prompts (nem chaves, nem corpo de `.env`).
8. Permitir dois escritores no mesmo log (lock/serialização obrigatórios).
9. Escalar o modo `plan → build` sem aprovação (I15).
10. Criar `HashSet`/`HashMap`/`Rc`/`RefCell` se quiseres o mesmo rigor do ktu (determinismo/Send).

## 2. Mapa de símbolos (para grep rápido)

| Símbolo | Caminho | Papel |
|---|---|---|
| `Ktu` / `KtuBuilder` / `TurnOutcome` | `ktu_core::facade` (re-export em `ktu_core`) | a fachada |
| `facade::system_prompt` / `facade::final_text` | `ktu_core::facade` | prompt efectivo / resposta do turno corrente |
| `SessionLog` / `Event` | `ktu_core::session` (re-export em `ktu_core`) | o log = a sessão |
| `session::{title_from, title_of}` | `ktu_core::session` | título da sessão (derivado, sem LLM) |
| `prompt::{Directive, compose_layers, SYSTEM_FILE, APPEND_SYSTEM_FILE}` | `ktu_core::prompt` | directriz do projecto/utilizador |
| `agents_md::{discover, nearest, render}` | `ktu_core::agents_md` | `AGENTS.md` aninhados (tectos) |
| `command::{Command, parse, load, find, CommandSettings, ShellAccess}` | `ktu_core::command` | comandos/receitas (`/nome`, settings, *shell* opt-in) |
| `tool_search::{spec, rank, render, TOP_K}` | `ktu_core::tool_search` | descoberta BM25 de tools diferidas |
| `ToolExecutor::deferred` | `ktu_core::tools` | tools fora do catálogo (exposição como dado) |
| `Instructions` / `ports::Local` | `ktu_core::ports` | instruções locais na leitura de ficheiro |
| `facade::evaluate_fact` | `ktu_core::facade` | avaliar um facto fora de um turno (com o mesmo encadeamento) |
| `agent_file::{ToolPolicy, ToolAccess, AgentMode, load, apply}` | `ktu_core::agent_file` | agentes declarativos (fail-closed) |
| `frontmatter::{split, parse, body}` | `ktu_core::frontmatter` | frontmatter `---` dos documentos declarativos |
| `AgentRegistry::{with_named, with_primary, subagent_names}` | `ktu_core::agent` | agentes declarativos no registo |
| `SubagentHost::agents` / `subagent::spec_with` | `ktu_core::subagent` | nomes de agentes no catálogo do modelo |
| `store::{save, load, load_recovering, to_jsonl, from_jsonl, session_path}` | `ktu_core::store` | persistência do log |
| `checkpoint::{Checkpoint, save, load, path}` | `ktu_core::checkpoint` | retoma idêntica |
| `compact::keep_from_for_turns` | `ktu_core::compact` | corte de compactação |
| `BudgetGate` / `CancelToken` | `ktu_core::{budget, cancel}` | orçamento (I14) / cancelamento (I4) |
| `StopReason` | `ktu_core::domain` | fim de turno (I5) |
| `Mode` / `Approval` / `FsPlanHost` | `ktu_core::plan` | modo plano (I15) |
| `AgentRegistry` / `AgentSpec` / `AgentId` | `ktu_core::agent` / `ktu_core::guard` | tools permitidas + profundidade |
| `Provider` / `ProviderError` / `ProviderRequest` / `Completion` | `ktu_core::provider` | porta do modelo |
| `ToolExecutor` / `ToolOutcome` / `ToolSpec` / `ToolCall` | `ktu_core::tools` / `ktu_core::domain` | porta das tools |
| `Memory` / `MemoryDraft` / `MemoryWrite` / `Hint` | `ktu_core::memory` | porta da memória |
| `Hooks` / `HookPayload` / `NoHooks` | `ktu_core::plugin` | plugins |
| `Steering` | `ktu_core::agent` | mensagens durante o turno |
| `SubagentHost` / `SubagentRequest` / `SubagentReport` | `ktu_core::subagent` | subagentes |
| `Fs` / `Clock` / `Logger` / `Metrics` / `Embedder` | `ktu_core::ports` | portas base |
| fakes: `MemFs`, `FixedClock`, `NoopLogger`, `RecordingLogger`, `NoopMetrics`, `FakeEmbedder`, `MapEnv`, `SeqRng`… | `ktu_core::ports` | testes |
| fakes: `FakeProvider`, `FakeTools` | `ktu_core::testing` | testes |
| `StdFs` / `ToolThread` / `Toolbox` / `default_registry` / `plan_mode_spec` | `ktu_tools` | adaptadores reais |
| `LlamaProvider` / `LlamaConfig` / `HttpEmbedder` / `DEFAULT_BASE_URL` | `ktu_provider` | provider real / embeddings |
| `JsonlMemory` / `project` / `drain` / `health` | `ktu_memory` | memória durável |
| `PluginHost` | `ktu_plugin` | plugins externos |
| `Config` / `Auth` / `MemoryConfig` | `ktu_core::{config, memory_config}` | configuração |
| `layout::{INIT_DIRS, SESSIONS_DIR, NOTES_FILE, SKILL_FILE, CONFIG_FILE}` | `ktu_core::layout` | layout do projecto |
| `layout::{AGENTS_DIR, AGENTS_FILE, SYSTEM_FILE, APPEND_SYSTEM_FILE, COMMANDS_DIR, AGENTS_FILES_DIR}` | `ktu_core::layout` | caminhos das convenções |
| `Config::deferred_tools` | `ktu_core::config` | tools diferidas do projecto |
| `events::*` | `ktu_core::events` | ids de evento estáveis |

## 3. Checklists

### 3.1 Antes de escrever código

- [ ] Preciso da fachada? (se quiseres só o protocolo/CLI, usa `ktu-port`/`ktu-cli`.)
- [ ] Que portas reais vou injectar? (`StdFs`, provider, tools, memória, plugins.)
- [ ] Onde vivem os adaptadores (dono + tempo de vida) e em que ordem os declaro?
- [ ] Tenho um `BudgetGate` e um `CancelToken` definidos antes do `Ktu`?
- [ ] Que `AgentSpec` (tools permitidas) e que `plan_spec` read-only vou passar?

### 3.2 Antes de dar por concluído

- [ ] `build()` tratado; `turn()` tratado; `outcome.is_ok()` usado.
- [ ] Sessão gravada (se persistente) e `log.verify()` vazio.
- [ ] Directórios de runtime garantidos no arranque.
- [ ] Logs em `stderr` com ids estáveis; sem segredos; `stdout` só com dados.
- [ ] Um `CancelToken` por turno; cancelamento testado.
- [ ] Teste determinístico com fakes (mesmo script ⇒ mesmo JSONL).
- [ ] Nenhum `unwrap`/`expect`/`panic`/`unsafe` no caminho de produção.

### 3.3 Ao diagnosticar

- [ ] Ler `log.events()` (sequência) e `log.derive_messages()` (o que o modelo viu).
- [ ] Filtrar o log estruturado por `kernel.error`, `provider.error`, `guard.decision`.
- [ ] Confirmar `outcome.reason` antes de culpar o modelo.
- [ ] Reproduzir com `FakeProvider` (se não reproduz, é provider/ambiente).
- [ ] Consultar a tabela de sintomas em [07](./07-erros-e-diagnostico.md) §4.

## 4. Anti-padrões → correção

| Se vires… | Está errado porque… | Faz antes |
|---|---|---|
| `Deps { … }` / `run_turn(` no código de integração | duplica a composição; divergirá | `Ktu::builder()…build()` |
| `fs.write(…jsonl)` para gravar sessão | quebra o append-only e o `derive_messages` | `save_session` / `store::save` |
| `HashMap`/`HashSet` no caminho do agente | iteração não determinística | `BTreeMap`/`BTreeSet` |
| `SystemTime::now()` no kernel/lógica | não determinístico | porta `Clock` |
| `println!`/`eprintln!` para eventos | mistura dados e logs | porta `Logger` (stderr) |
| `BudgetGate::new(…)` em 2 sítios | viola I14 | uma instância partilhada |
| `CancelToken::new()` reutilizado | fica cancelado para sempre | um por turno |
| `.plan_spec` omitido | modo plano recusado em runtime | passar sempre a spec read-only |
| `turn()` com `let _ =` | perde I5 (razão de término) | tratar `TurnOutcome` |
| `unwrap()` no `build()` | esconde configuração inválida | propagar/tratar `Error::Invalid` |
| editar `permissions.toml` para “deixar passar” | os guards só restringem | mudar a política consciente-mente |

## 5. Árvore de decisão

```
Preciso de…                        → fazer
─────────────────────────────────────────────────────────────────────────────
correr um turno                    → ktu.turn(&mut log, prompt).await?
guardar/retomar sessão             → ktu.session(id) / ktu.save_session(id, &log)
saber porque acabou o turno        → outcome.reason (StopReason)
parar o turno a meio               → cancel.cancel() (CancelToken do turno)
injectar contexto durante o turno  → porta Steering (Event::Steer, fronteira segura)
impedir uma tool                   → .agents/permissions.toml (never/ask) ou guardrail
restringir o agente a leitura      → .plan_spec + log.set_mode(Mode::Plan)
dar memória durável                → JsonlMemory + .memory(&memory) + Toolbox(Some(&memory))
consultar memória do agente        → kd ask / kd ai memory / kd ai session-log
embedings locais                   → .agents/memory.toml + HttpEmbedder + drain
plugins externos                   → PluginHost::load(.ktu/plugins) + .hooks(&plugins)
subagentes                         → implementar SubagentHost + .subagents(&host)
correr N sessões em paralelo       → um Ktu por thread (std::thread::scope), ids distintos
dar instruções por directório       → AGENTS.md aninhados (raiz + locais; tectos)
substituir/estender o prompt        → .agents/SYSTEM.md / .agents/APPEND_SYSTEM.md (+ .user_dir)
definir um comando reutilizável     → .agents/commands/<nome>.md (`/nome args`, `$ARGUMENTS`)
definir um agente próprio           → .agents/agents/<nome>.md (mode + tools allow/deny)
encurtar o catálogo de tools        → deferred_tools + tool_search (convenções §6)
dar regras por directório           → instruções locais (Instructions; convenções §7)
correr uma receita com settings     → .agents/commands/<nome>.md + `ktu run --command`
```

## 6. Invariantes que o código gerado tem de preservar

| # | Invariante | Como se verifica |
|---|---|---|
| I1/I2 | nenhum turno aberto; toda tool tem resultado | `log.verify().is_empty()` |
| I3 | o log é a fonte da verdade | nada fora do `SessionLog` influencia o contexto |
| I4 | cancelável | teste com `CancelToken` cancelado ⇒ `Cancelled` |
| I5 | nunca termina em silêncio | `TurnOutcome.reason` sempre tratado |
| I6 | replay determinístico | mesmo script + fakes ⇒ JSONL idêntico |
| I13 | output de tool é dado, não instrução | envelope *taint* intacto (não contornar) |
| I14 | um dono do tecto de contexto | uma instância de `BudgetGate` |
| I15 | modo plano read-only estrutural | `plan_spec` presente; sem ele ⇒ recusa |
| I16 | guardrails fail-closed | ficheiro inválido ⇒ `build()` falha com o nome |
| M2 | efeitos só por portas | nenhum `std::fs`/rede fora dos adaptadores |
| M6 | o projecto é a autoridade | config local ganha à global (chave a chave) |

## 7. Contrato em bloco (colável num prompt)

```text
Integração ktu (fachada):
- API: Ktu::builder().root(fs/clock/logger/metrics/provider/tools/budget/cancel/registry)[.plan_spec/.memory/.hooks/.subagents/.steering/.max_steps].build()?
- Turno: let o = ktu.turn(&mut log, prompt).await?; usar o.text (dado) e o.reason (StopReason).
- Sessão: SessionLog (o log É a sessão); ktu.session(id)/ktu.save_session(id,&log); .ktu/sessions/<id>.jsonl.
- Obrigatórias: root, fs, clock, logger, metrics, provider, tools, budget, cancel, registry. plan_spec na prática.
- Um BudgetGate; um CancelToken por turno; directórios criados pela app; adaptadores declarados antes do Ktu.
- Fail-closed: guardrails/permissões inválidos ⇒ build() Err (nomeia ficheiro); plano sem plan_spec ⇒ turn Err.
- Proibido: Deps/Ports/run_turn à mão, editar JSONL, println! para eventos, unwrap/expect/panic/unsafe, segredos no log.
- Testes: MemFs+FixedClock+FakeProvider+FakeTools ⇒ store::to_jsonl igual (I6).
- Docs: wiki/integration/*.md; exemplo compilado: examples/ktu-embed.
```

---

Voltar ao [índice](./README.md) · Contrato de desenho: [fachada-crate](../fachada-crate.md) ·
Implementação: `crates/ktu-core/src/facade/`.
