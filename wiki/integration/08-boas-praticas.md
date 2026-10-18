# 08 — Boas práticas

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [07 — Erros e diagnóstico](./07-erros-e-diagnostico.md) · **Seguinte:** [09 — Exemplos](./09-exemplos.md)

## 1. Composição e tempos de vida

| Prática | Porquê |
|---|---|
| Declarar adaptadores **antes** do `Ktu` | `Ktu<'a>` empresta-os; a ordem de declaração é a ordem de *drop* |
| **Um** `Ktu` por sessão/configuração, reutilizado em N turnos | a composição (guardrails, prompt, orçamento) é feita uma vez |
| Um `SessionLog` por sessão; `turn(&mut log, …)` | o log **é** a sessão (I3) |
| Não clonar/recriar adaptadores por turno | barato em CPU, mas perde estado (memória, plugins, caches) |
| Para paralelismo, **um `Ktu` por tarefa** | não há estado partilhado no `Ktu`; evita disputas no mesmo log |
| Guardar o `ToolThread` vivo enquanto o `Ktu` existir | se a T2 cair, as tools devolvem erro (não pânico), mas o turno degrada |

## 2. Runtime e assincronia

- A fachada é `async`; um runtime **`current_thread`** basta e é o mais determinístico
  (é o que a CLI usa em modo *embedded*).
- **Não bloquear** dentro do turno: o provider e as tools são `await`-áveis; trabalho pesado é do
  provider ou da T2.
- A `ToolThread` cria o seu **próprio** runtime interno (`current_thread`): não lhe passes um
  runtime nem bloqueies a thread do kernel à espera dela.
- Um runtime por processo é suficiente; não cries um por turno.
- `await_holding_lock` está **negado** no workspace: nunca segures um `MutexGuard` através de um
  `await` (drena o buffer de *steering* antes de devolver).

## 3. Controlo: orçamento e cancelamento

- **Um só `BudgetGate`** (I14) para o `Ktu` **e** para o `SubagentHost` — a mesma instância. Dois
  tectos independentes é o anti-padrão que a invariante proíbe.
- `Ktu::CONTEXT_BUDGET_TOKENS` é o valor de referência; ajusta-o à janela do modelo, não ao prompt.
- **Um `CancelToken` por turno**, partilhado com quem cancela (UI, timeout, sinal). Não recicles um
  token cancelado — `is_cancelled()` fica verdadeiro para sempre.
- Cancelar é seguro em qualquer ponto: o turno fecha com `StopReason::Cancelled` e o log fica
  consistente (grava-o).

## 4. Determinismo e testes (I6)

- Testes com **fakes**: `FixedClock` (instante fixo), `MemFs`, `FakeProvider` (script), `FakeTools`,
  `NoopLogger`/`RecordingLogger`, `NoopMetrics`/`RecordingMetrics`.
- Compara o **JSONL** (`store::to_jsonl(&log)`), não estruturas internas: é o contrato de bytes.
- Sem `SystemTime::now`/`Instant::now`/`rand`/`env::var` fora dos adaptadores (o `clippy.toml` do
  ktu nega-os — mantém a disciplina no teu crate se quiseres o mesmo rigor).
- Um teste de integração típico: (a) mesmo script ⇒ logs byte-idênticos; (b) guardrail quebrado ⇒
  `build()` falha; (c) modo plano ⇒ só tools read-only; (d) cancelar ⇒ `Cancelled`.
- Para *property testing*, o ktu usa `proptest`; para um embedder, um script de `Completion` já
  cobre o essencial.

## 5. Logs e observabilidade

| Regra | Detalhe |
|---|---|
| **stdout = dado, stderr = log** | nunca escrever eventos em `stdout` (parte *pipes*) |
| ids de evento **estáveis** | `ktu_core::events::*` (`turn.start`, `guard.decision`, `memory.write`…); nunca texto livre como id |
| nada de segredos | nem chaves, nem `.env`, nem corpo de ficheiro; `LogRecord` tem `fields` para o resto |
| níveis com sentido | `Error` = operação impedida; `Warn` = anomalia que segue; `Info` = marco; `Debug`/`Trace` = diagnóstico |
| métricas opcionais | a feature `instrument` mede `derive_messages`; sem ela, custo zero |

## 6. Segurança

- **Fail-closed**: configuração inválida **para** o arranque; gates de plugin com erro negam.
- **Apagar = mover para o trash** (`.ktu/trash`, recuperável) — nunca `rm` a partir do agente.
- **Modo plano** para revisão: read-only é estrutural (`AgentSpec`), não um pedido no prompt.
- **Permissões explícitas** só onde precisas; o default já nega o que é estruturalmente proibido.
- **Contenção de caminhos**: o `StructuralGuard` nega `PathEscape` (nada fora do *root*); não
  contornes com paths resolvidos à mão na tua tool.
- **Dado ≠ instrução** (I13): o output de tools é envolvido num envelope *taint* pelo kernel;
  não desactives nem “limpes” esse envelope.
- **Sem `unsafe`/`unwrap`/`expect`/`panic`** em código de produção; propaga `Result`.

## 7. O que **não** fazer

| Anti-padrão | Porquê | Alternativa |
|---|---|---|
| montar `Deps`/`Ports`/`Capabilities` à mão | duplica a composição e diverge com o tempo | `Ktu::builder()…build()` |
| chamar `run_turn` directamente | idem, e perde a escolha do `AgentSpec` por modo | a fachada |
| reimplementar a leitura de guardrails/permissões | duas fontes de verdade | a fachada lê-os no `build()` |
| escrever eventos arbitrários no log | o log é o contrato (I3); `verify()` deixa de valer | só `ModeSet`/`Steer` (e via API) |
| editar `.ktu/sessions/<id>.jsonl` à mão | quebra o append-only e o `derive_messages` | `SessionLog` + `save_session` |
| chamar o provider fora do loop | perde guardrails, orçamento, log, cancelamento | `turn` |
| dois escritores no mesmo log | corrupção silenciosa | lock/serialização ([04](./04-sessoes.md) §5) |
| um `BudgetGate` por componente | viola I14 | uma instância partilhada |
| `let _ = ktu.turn(…)` | perde a razão de término (I5) | tratar `TurnOutcome` |
| silenciar o `Logger` | perde a evidência de guardrails/erros | `NoopLogger` só em testes |

## 8. Configuração e versionamento

- O **projecto é a autoridade** (M6): a config local ganha à global, chave a chave. A app resolve a
  precedência; a fachada recebe o resultado.
- Segredos **só** no global (`~/.config/ktu/auth.toml`, 0600) e nunca no log/prompt (`Auth` tem
  `Debug` redigido).
- A **fachada é a API estável**; `run_turn`/`Deps`/`Ports` são internos e podem mudar. Se
  precisares deles, considera isso um sinal de que algo devia passar pela fachada.
- Fixa a versão do ktu (path/git pinado) e corre `make check` do teu lado com o teu gate: os
  invariantes (I1–I17) são o contrato, não a forma dos tipos internos.

## 9. Se contribuíres para o ktu

O projecto tem regras próprias (aplicadas por `make check`):

- ficheiros de produção ≤ **400 linhas** (`crates/*/src`, `examples/*/src`, `xtask/src`);
- `clippy --workspace --all-targets -- -D warnings` + `xtask check-layers` (firewall de camadas);
- `#[allow(...)]` **exige** `reason`;
- determinismo: `BTreeMap`/`BTreeSet`, sem `HashMap`/`HashSet`/`Rc`/`RefCell`;
- `stdout` = dados, `stderr` = logs; exit codes `0/1/2`;
- sem commits automáticos — o versionamento é gesto humano.

## 10. Checklist de revisão

- [ ] A composição é feita uma vez; os turnos reutilizam o `Ktu`.
- [ ] Um `BudgetGate` e um `CancelToken` (por turno) — partilhados correctamente.
- [ ] `TurnOutcome` tratado; sessão gravada; `verify()` vazio.
- [ ] Testes determinísticos com fakes (e um teste de “mesmo script ⇒ mesmo log”).
- [ ] Logs em `stderr` com ids estáveis, sem segredos.
- [ ] Nenhum atalho ao kernel (sem `run_turn`, sem escrita directa no JSONL).

Próximo: [09 — Exemplos](./09-exemplos.md).
