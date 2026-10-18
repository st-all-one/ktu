# 03 — Turnos: `turn`, resultado, cancelamento, modos e guardrails

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [02 — Construtor](./02-construtor.md) · **Seguinte:** [04 — Sessões](./04-sessoes.md)

## 1. A chamada

```rust
let outcome: TurnOutcome = ktu.turn(&mut log, "corrige o bug do parser").await?;
println!("{}", outcome.text);          // resposta final (stdout é dado)
eprintln!("{:?}", outcome.reason);     // razão de término (log)
```

```rust
pub struct TurnOutcome {
    pub reason: StopReason,   // nunca em silêncio (I5)
    pub text: String,         // último passo sem tool_calls; vazio se não houve
}
impl TurnOutcome { pub fn is_ok(&self) -> bool; }   // reason != StopReason::Error
```

`turn` devolve `Err` apenas para erros de **arranque** do turno (hoje: modo plano sem
`plan_spec`). Tudo o resto — provider fora, tool negada, estagnação, cancelamento — é um
`StopReason` dentro do `TurnOutcome` (I5).

## 2. O que acontece por dentro (um passo = uma chamada ao modelo)

1. **hooks** — `SessionStart` (só no primeiro turno da sessão) e `UserInput` (transformável);
2. **recall** — se injectaste `memory` e é o primeiro turno, os *hints* entram no system prompt e
   é emitido `memory.recall`; as memórias são **ponteiros**, nunca o corpo;
3. `Event::TurnStart` + log `turn.start`;
4. por cada passo (até `max_steps`):
   - cancelamento? → `StopReason::Cancelled` (antes de qualquer efeito);
   - **steering** pendente é entregue no início do passo (`Event::Steer`, log `turn.steer`);
   - mensagens **derivadas do log** (I3) e, se não caberem no orçamento, **uma** compactação;
   - pedido ao modelo (`model.request` + hook `ModelRequest`, que pode negar → `Error`);
   - resposta (`model.output` + hook `ModelOutput`, que pode transformar o texto);
   - sem tool calls → resposta final; com tool calls → guardrails e execução (T2);
   - **eco**: se o texto repetir o pedido/resposta anterior, há **um** retry com *nudge*; se
     persistir, a resposta **não é aceite** e o turno fecha com `StopReason::Echo`;
   - **estagnação**: a mesma assinatura de tool calls `max_stagnation` (3) vezes → `Stalled`;
   - **último passo**: as tools são desligadas e o modelo é instruído a resumir; chamadas
     pendentes recebem resultados sintéticos; fecha com `MaxSteps`;
5. `Event::TurnEnd` + log `turn.end`;
6. **memória de fim de turno** — com `memory` e `reason == EndTurn`, a resposta final é gravada
   como nota `meta` (tags `session`, `auto`; log `memory.write`), com o mesmo protocolo
   idempotente de dedup;
7. hook `SessionEnd`.

## 3. `StopReason` — o que fazer com cada um

| Variante | Significado | O que fazer |
|---|---|---|
| `EndTurn` | o modelo respondeu com texto | caminho normal: usar `outcome.text` |
| `MaxSteps` | atingiu o tecto de passos | aumentar `max_steps`, dar melhores tools/prompt, ou aceitar o resumo final |
| `Stalled` | repetiu a mesma assinatura de tools 3× | o modelo está em ciclo: rever prompt/tools ou reduzir o âmbito |
| `Echo` | repetiu o pedido/uma resposta anterior (G5) | o eco **nunca** é aceite: rever o modelo/prompt; `outcome.text` é a nota `[eco: …]` |
| `Cancelled` | o `CancelToken` foi accionado | não é falha: limpar estado, mostrar “cancelado” ao utilizador |
| `Error` | falha do kernel/provider | ver `kernel.error`/`provider.error` no log; `outcome.is_ok() == false` |

Sugestão de política na app:

```rust
match outcome.reason {
    StopReason::EndTurn => mostrar(&outcome.text),
    StopReason::Cancelled => { /* não alarmar */ }
    StopReason::Error => return Err("turno falhou".into()),
    outro => mostrar_parcial(&outcome.text, outro),
}
```

## 4. Cancelamento (I4)

O `CancelToken` que injectas é **o mesmo** que a UI cancela — de qualquer thread, sem esperar pelo
turno:

```rust
let cancel = CancelToken::new();
let token = cancel.clone();                  // CancelToken é clonável e Send
std::thread::spawn(move || { /* sinal do utilizador */ token.cancel(); });
let outcome = ktu.turn(&mut log, prompt).await?;   // fecha com StopReason::Cancelled
```

- O kernel verifica o token no início de cada passo **e** durante o *streaming* do provider
  (`ProviderRequest.cancel`); um cancelamento a meio vira `ProviderError::Cancelled` →
  `StopReason::Cancelled`.
- **Um** token por turno (não recicles um token já cancelado: `is_cancelled()` fica verdadeiro).
- Um turno cancelado **deixa o log consistente** (o `TurnEnd` é sempre escrito) — podes gravar.

## 5. Modos: `build` e `plan` (I15)

```rust
log.set_mode(Mode::Plan);                  // restringir é sempre permitido
let _ = ktu.turn(&mut log, "planeia a migração").await?;
// plan → build exige aprovação explícita do utilizador (I15):
//   Mode::transition(current, Mode::Build, Approval::Approved)
```

- O modo vive **no log** (`Event::ModeSet`), não no `Ktu`: duas sessões com o mesmo `Ktu` podem ter
  modos diferentes.
- A fachada escolhe o `AgentSpec` pelo modo: `build` → `registry.primary()`; `plan` →
  `plan_spec`. A partir dele faz **duas** camadas: `ScopedTools` (tools oferecidas ao modelo) e
  `StructuralGuard` (recusa estrutural). O read-only **não** depende do prompt.
- Sem `.plan_spec(...)`, `turn` devolve `Err(Error::Invalid)` e **não toca no log** — fail-closed.
- Para o modo plano ser útil, o teu executor de tools deve saber servir a tool `plan`
  (`Toolbox::with_plan(FsPlanHost::new(...))`); ver [09](./09-exemplos.md) §6.

## 6. Orçamento de contexto (I14) e `max_steps`

- O `BudgetGate` que injectas é o **único** dono do tecto. `Ktu::CONTEXT_BUDGET_TOKENS` (120 000)
  é o valor de referência; ≈4 bytes por token.
- Se as mensagens derivadas não caberem, o kernel **compacta uma vez** (mantém os últimos 4 turnos
  verbatim + resumo dos anteriores) e volta a medir; se mesmo assim não caber → `StopReason::Error`
  com `kernel.error`. Nunca se “corta evidência” em silêncio.
- `max_steps` é por turno. O último passo corre **sem tools** (só resumo) — por isso um
  `max_steps: 1` serve para um turno puramente textual.

## 7. Guardrails e permissões no caminho de uma tool

Antes de qualquer efeito, a cadeia de guards é avaliada (nesta ordem):

```
StructuralGuard (AgentSpec: conjunto de tools, root, profundidade)
   └── DeclarativeGuard (.agents/guardrails/*.md)
          └── PermissionGuard (.agents/permissions.toml)
```

- `Allow` → a tool corre na T2; `Deny(reason)` → erro tipado devolvido ao modelo **com remédio**;
  `Escalate` → o modelo recebe “requer aprovação” (o front-end só pode **restringir**, nunca alargar).
- Cada decisão fica no log como `Event::GuardDecision { tool, allowed, reason }` + evento
  `guard.decision` — é a evidência da tese (§2.8).
- `DenyReason`: `Capability`, `PathEscape`, `StepCap`, `DepthExceeded`, `Plugin`, `Guardrail`,
  `Policy` — texto estável, contrato de máquina.

## 8. Steering (mensagens durante o turno)

```rust
struct MeuSteering { pendentes: Mutex<Vec<String>> }
impl Steering for MeuSteering {
    fn take(&self) -> Vec<String> { /* drena por ordem de chegada */ }
}
```

- A porta é consultada no **início de cada passo** (fronteira segura): a mensagem entra no contexto
  e é **logada antes** (`Event::Steer`, log `turn.steer`) — *model-visible ⟺ logged*.
- Não mata o turno: o modelo vê a mensagem no passo seguinte.
- O `take()` é chamado pelo kernel: a tua implementação deve ser barata e não bloquear.

## 9. Instruções locais e tools diferidas

Duas afinações do contexto que a fachada liga por dado (ver [convenções](../convencoes.md) §6–§7):

- **Instruções locais** (`.instructions(&adaptador)`): ao ler um ficheiro (`read_file`/`edit_file`),
  o `AGENTS.md` mais próximo é anexado ao **resultado da tool** — uma vez por turno e por fonte
  (evento `context.local`). Sem a porta, nada é anexado.
- **Tools diferidas**: se o teu `ToolExecutor` declarar `deferred()`, o loop oferece `tool_search`
  (BM25, ≤ 8 resultados) e promove o que for encontrado para o **pedido seguinte** (`tool.loaded`).
  A exposição é de **catálogo**, não de permissão: o guard continua a conhecer o conjunto completo.
  A fonte prática é `deferred_tools` em `.agents/config.toml` (`Toolbox::with_deferred`).

## 10. Plugins / hooks

Se injectares `hooks` (`PluginHost`), cada marco do turno passa pelo *dispatcher*:
`SessionStart`, `UserInput`, `ModelRequest`, `ModelOutput`, `ActionBefore`, `ActionAfter`,
`GuardDecision`, `SessionEnd`. Cada *dispatch* fica no log (`Event::PluginDispatch`).

- `Mode::Observe` — só observa; falha passa adiante.
- `Mode::Transform` — transforma o payload (ex.: reescrever o input); falha passa adiante.
- `Mode::Gate` — decide; **timeout/erro é `Deny`** (fail-closed, I11).

## 11. Checklist do fim de um turno

- [ ] `outcome.reason` tratado (incl. `Error`/`Cancelled`).
- [ ] `outcome.text` mostrado ao utilizador (nunca o log — stdout é dado).
- [ ] `log.verify()` vazio (I1/I2: nenhum turno aberto, nenhuma tool sem resultado).
- [ ] Sessão gravada (`save_session`) se for persistente — ver [04](./04-sessoes.md).
- [ ] Eventos relevantes (`guard.decision`, `memory.write`, `kernel.error`) observados no `Logger`.

Próximo: [04 — Sessões](./04-sessoes.md).
