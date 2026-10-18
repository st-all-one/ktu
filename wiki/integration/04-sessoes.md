# 04 — Sessões: o log, persistência, retoma e compactação

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [03 — Turnos](./03-turnos.md) · **Seguinte:** [05 — Portas e adaptadores](./05-portas-e-adaptadores.md)

## 1. O log **é** a sessão (I3)

Não há um objecto “sessão” além do log: tudo o que o modelo viu é reconstruível a partir dos
eventos, e `derive_messages` é a única fonte do pedido ao provider.

```rust
let mut log = SessionLog::new();
ktu.turn(&mut log, "…").await?;      // append-only
for event in log.events() { … }      // inspecção/telemetria
```

| API | Para quê |
|---|---|
| `SessionLog::new()` / `Default` | sessão nova |
| `events() -> &[Event]` | inspecção (telemetria, UI, projeção) |
| `push(Event)` | acrescentar eventos **próprios** (só `ModeSet`/`Steer` na prática) |
| `derive_messages() -> Vec<Message>` | projeção de *wire* (o que o modelo veria) |
| `mode()` / `set_mode(Mode)` | modo da sessão, sempre logado (`Event::ModeSet`) |
| `is_compacted()` / `compaction_from()` / `set_compaction(from)` | corte de compactação |
| `verify() -> Vec<LogViolation>` | I1/I2: turno aberto? tool sem resultado? |

### 1.1 `Event` — vocabulário do log

| Evento | Campos | Projeção para o modelo |
|---|---|---|
| `TurnStart` | `prompt` | mensagem `User` |
| `Steer` | `text` | mensagem `User` |
| `Assistant` | `content`, `reasoning`, `tool_calls` | mensagem `Assistant` |
| `ToolResult` | `tool_call_id`, `name`, `content`, `is_error` | mensagem `Tool` |
| `GuardDecision` | `tool`, `allowed`, `reason` | — (evidência) |
| `TurnEnd` | `reason` | — |
| `ModeSet` | `mode` | — |
| `PluginDispatch` | `hook`, `generation`, `allowed`, `reason` | — |

> `reasoning` (pensamento) é **campo próprio**: nunca é reenviado ao modelo em todos os dialetos
> (o `ktu-provider` decide por *flavor*). Não o mistures com `content`.

### 1.2 Violações

`log.verify()` devolve `LogViolation::OpenTurn` (um `TurnStart` sem `TurnEnd` — só acontece se
gravares um log a meio de um turno) e `LogViolation::MissingToolResult(id)`. **Depois de `turn`
voltar, deve estar vazio.** Usa-o como asserção na tua integração.

## 2. Persistência pela porta `Fs`

```rust
let mut log = ktu.session("s1")?.unwrap_or_default();   // retoma, ou sessão nova
let outcome = ktu.turn(&mut log, "…").await?;
ktu.save_session("s1", &log)?;                          // grava durável
```

- Caminho: `<root>/.ktu/sessions/<id>.jsonl` (uma linha JSON por evento).
- `save` = `write` do `*.tmp` + `rename` atómico; `load` devolve `None` se não existir;
  `load_recovering` **tolera a última linha parcial** (crash a meio de uma escrita) e falha em
  qualquer outra linha malformada.
- `session()`/`save_session()` são açúcar sobre `ktu_core::store`; podes usar o módulo directo
  (`store::session_path`, `to_jsonl`, `from_jsonl`, `load`, `save`) se precisares de outro layout.

```rust
use ktu_core::store;
let jsonl: String = store::to_jsonl(&log)?;      // comparar logs (determinismo, I6)
```

> **A porta `Fs` não cria directórios.** Antes do primeiro `save_session`, garante
> `<root>/.ktu/sessions` (a CLI fá-lo em `ensure_layout`; o exemplo `ktu-embed` também).

## 3. Retomar uma sessão

Retomar = **carregar o log e continuar** a acrescentar. Não há estado escondido: o contexto do
modelo é rederivado dos eventos.

```rust
let mut log = ktu.session("s1")?.unwrap_or_default();
if !log.events().is_empty() { /* é uma retoma: podes logar session.resume */ }
ktu.turn(&mut log, "e agora?").await?;
```

Cuidados:

- **Um escritor por sessão.** Dois processos/tarefas a gravar o mesmo `<id>.jsonl` corrompem-no.
  A CLI usa um lock por ficheiro (`<id>.lock`, `create_new`, *stale* 30 s) e **recusa** quando
  está vivo — ver §5.
- Se o log tiver um turno aberto (`verify()` não vazio), o provider vai receber mensagens órfãs:
  prefere truncar no último `TurnEnd` antes de continuar.

## 4. Compactação e *checkpoint*

A compactação existe para o contexto caber no orçamento (I14). É um **corte** (`compaction_from`):
os eventos anteriores são substituídos, na projeção de *wire*, por um resumo; o log cru fica
intacto.

```rust
use ktu_core::compact::keep_from_for_turns;
if let Some(from) = keep_from_for_turns(log.events(), 4) {
    log.set_compaction(from);          // mantém os últimos 4 turnos verbatim
}
```

O kernel compacta **sozinho** quando o pedido não cabe (uma vez por passo). O que a app costuma
acrescentar é:

1. **compactação forçada** no arranque (ex.: um flag `--compact` que mantém 1 turno);
2. um **checkpoint** durável para que a retoma seja idêntica:

```rust
use ktu_core::checkpoint::{self, Checkpoint};
let dir = ktu.sessions_dir();
let checkpoint = Checkpoint {
    session: "s1".to_owned(),
    turns: log.events().iter().filter(|e| matches!(e, Event::TurnEnd { .. })).count() as u32,
    compacted: log.is_compacted(),
    keep_from: log.compaction_from().and_then(|v| u32::try_from(v).ok()),
    title: ktu_core::session::title_of(&log), // derivado do 1.º pedido (índice, não verdade)
    updated_ms: clock.now().as_millis(),
};
checkpoint::save(&fs, &dir, &checkpoint)?;
// na retoma:
if let Some(cp) = checkpoint::load(&fs, &dir, "s1")? {
    if let Some(keep_from) = cp.keep_from { log.set_compaction(keep_from as usize); }
}
```

> O `Event::TurnEnd` é o marcador de turnos concluídos — usa-o para contagens (`turns`), nunca
> `TurnStart` (um turno cancelado também o tem).

## 5. Título da sessão

Listar sessões por `id` é pouco legível. O título é **derivado** do primeiro pedido, sem LLM:

```rust
use ktu_core::session::{title_from, title_of};

let title = title_from("<pedido>Arranja o bug do parser</pedido>"); // Some("Arranja o bug do parser")
let title = title_of(&log);                                        // o do 1.º `TurnStart`
```

- heurística determinística: ≤ 8 palavras e ≤ 60 caracteres (`session::MAX_WORDS`/`MAX_CHARS`),
  sem *tags* XML, cercas de código nem pontuação de extremidade;
- guarda-se no **checkpoint** (`title`) — é um **índice derivado**, não verdade (I3): pode ser
  recalculado do log a qualquer momento;
- `None` quando não há pedido utilizável — o consumidor usa o `id` como *fallback*;
- a CLI mostra-o em `ktu session ls` (última coluna; `--json` em `title`).

## 6. Concorrência

| Cenário | Recomendação |
|---|---|
| N sessões em paralelo | um `Ktu` + um `SessionLog` por tarefa, ids distintos |
| Mesma sessão, dois escritores | **proibido**: usar lock por ficheiro (como a CLI) ou serializar por canal |
| N turnos na mesma sessão | sequencial (o log é append-only e ordenado) |
| `ToolThread` partilhado | ok — desde que os turnos não sejam simultâneos no mesmo log |

Padrão de lock (o da CLI, reutilizável):

```rust
// ficheiro `<dir>/<id>.lock` com o instante; `create_new` recusa se existir;
// um lock com mais de 30 s é considerado obsoleto e reaproveitado;
// NUNCA apagar um lock alheio (só o obsoleto).
```

## 7. Da sessão para a memória (projeção)

O log pode ser **projetado** em notas derivadas, consultáveis pelo knudge:

```bash
ktu memory project --root .            # .ktu/sessions/*.jsonl → .agents/memory/session-log.jsonl
kd ai session-log "como funciona o dedup"
```

Em Rust: `ktu_memory::project::project(session, events) -> Vec<MemoryDraft>` +
`ktu_memory::project::write(fs, clock, path, drafts) -> MemoryResult<usize>` (idempotente por `id`+`body_hash`,
`activity=derive`, `agent=ktu`). É **read-only** sobre o log e derivado (apagável).

## 8. Checklist

- [ ] Directório `.ktu/sessions` criado pela app antes de gravar.
- [ ] Um escritor por sessão (lock ou serialização).
- [ ] `save_session` chamado mesmo quando o turno termina com `Cancelled`/`Error` (o log fica consistente).
- [ ] `log.verify()` vazio antes de gravar.
- [ ] Retoma aplica o `keep_from` do checkpoint (senão o contexto volta ao tamanho antigo).
- [ ] Sessões efémeras não são gravadas (não passes um id a `save_session`).

Próximo: [05 — Portas e adaptadores](./05-portas-e-adaptadores.md).
