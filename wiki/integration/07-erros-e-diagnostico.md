# 07 — Erros e diagnóstico

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [06 — Projecto](./06-projecto.md) · **Seguinte:** [08 — Boas práticas](./08-boas-praticas.md)

## 1. Modelo de erros

Tudo é **valor**, nunca pânico: `ktu_core::Result<T> = Result<T, ktu_core::Error>`.

| `Error` | Display | Quando |
|---|---|---|
| `Io { path, source }` | `io em <path>: <source>` | ler/gravar ficheiros pela porta `Fs` |
| `Json { context, source }` | `json (<context>): <source>` | serializar/desserializar eventos, checkpoints, planos |
| `Invalid { message }` | `entrada invalida: <message>` | fail-closed: portas em falta, guardrails/permissões inválidos, modo plano sem `plan_spec` |

`Error` é `#[non_exhaustive]`: faz sempre `match` com um braço final. Os erros das **portas** são
tipos próprios (não `Error`): `ProviderError`, `MemoryError`, `EmbedError` — o kernel traduz-os em
`StopReason`/`ToolOutcome`.

## 2. Fail-closed vs degradação graciosa

| Situação | Comportamento | Porquê |
|---|---|---|
| guardrail inválido em `.agents/guardrails/` | **aborta o `build()`** (nomeia o ficheiro) | I16: uma regra que não se entende não se ignora |
| permissão com valor desconhecido | **aborta o `build()`** | idem |
| modo plano sem `plan_spec` | **`turn` devolve `Err`** e não toca no log | I15: nunca escalar o modo |
| falta uma porta obrigatória | **aborta o `build()`** (`fachada sem …`) | erro de programação |
| provider fora/rede | `StopReason::Error` (log `provider.error`) | o turno falha alto, não inventa |
| tool falha | `ToolOutcome::error` → mensagem ao modelo | erro de tool ≠ erro de turno |
| memória indisponível no recall | degrada (segue sem *hints*; log de aviso) | o recall é uma ajuda, não um pré-requisito |
| índice de embeddings ilegível | tratado como ausente (lexical) | derivado e descartável |
| plugin mau / manifesto inválido | plugin **desactivado** (log `plugin.lifecycle`) | um plugin não derruba a sessão |
| *gate* de plugin com timeout/erro | `Deny` | I11: fail-closed |
| ficheiro `.env`/segredos | negado por guardrail por omissão | M3 |

## 3. Catálogo rápido

**`StopReason`** (fim de turno — ver [03](./03-turnos.md) §3): `EndTurn`, `MaxSteps`, `Stalled`,
`Echo`, `Cancelled`, `Error`.

**`ProviderError`**: `Unavailable(String)` (processo/rede) · `Invalid(String)` (pedido recusado) ·
`Cancelled` (a meio do passo → `StopReason::Cancelled`).

**`MemoryError`**: `Io { path, source }` · `Schema { message }` (afirmação vazia/longa, espécie
inválida) · `Unavailable { message }` (backend fora — o chamador degrada).

**`EmbedError`**: `Unavailable(String)` · `Invalid(String)` (resposta inválida ou **dimensão
errada** — nunca se comparam vetores de tamanhos diferentes).

**`LogViolation`**: `OpenTurn` (turno aberto) · `MissingToolResult(id)` (tool sem resultado) —
`log.verify()` deve devolver `[]` depois de um turno.

**`DenyReason`** (porque uma tool foi negada): `Capability`, `PathEscape`, `StepCap`,
`DepthExceeded`, `Plugin`, `Guardrail`, `Policy`.

## 4. Diagnóstico: sintoma → causa → correção

| Sintoma | Causa provável | Correção |
|---|---|---|
| `fachada sem porta Fs` (ou `Clock`, `Logger`…) | falta um setter no construtor | completar a cadeia; ver [02](./02-construtor.md) §2 |
| `guardrails invalidos: x.md: frontmatter ausente ou malformado` | frontmatter incompleto (falta `id`/`version`/`fact`/`match`/`reason`) | corrigir o `.md`; ver [06](./06-projecto.md) §3 |
| `guardrails invalidos: x.md: versao de schema nao suportada: 2` | `version` ≠ 1 | usar `version: 1` |
| `erro nas permissoes: …: permissao desconhecida para bash: talvez` | valor fora de `always/ask/never` | corrigir `permissions.toml` |
| `modo plano sem especificacao read-only (plan_spec)` | `log.mode() == Plan` sem `.plan_spec(...)` | passar a spec read-only (ex.: `plan_mode_spec()`) |
| `io em …/sessions/x.jsonl.tmp: No such file or directory` | `.ktu/sessions` não existe (a porta `Fs` não cria pastas) | `ensure_layout` na app ([06](./06-projecto.md) §8) |
| `provider indisponivel: …` e `StopReason::Error` | `llama-server` não está a correr / URL errada | arrancar o servidor ou corrigir `base_url` ([05](./05-portas-e-adaptadores.md) §2.2) |
| turno sempre `MaxSteps` | tecto baixo, ou o modelo entra em ciclo de tools | aumentar `max_steps`; melhorar prompt/tools; ver o resumo do passo final |
| `StopReason::Stalled` | a mesma assinatura de tool calls 3× | o modelo repete-se: rever o prompt, dar melhor contexto |
| `StopReason::Echo` | o modelo repetiu o pedido/resposta anterior | é recusado por desenho (G5): rever modelo/prompt; nunca aceitar o eco |
| `StopReason::Cancelled` inesperado | token cancelado antes/durante, ou reutilizado entre turnos | um `CancelToken` **novo** por turno |
| turno curto, `kernel.error` de orçamento | contexto maior que `BudgetGate` mesmo após compactar | aumentar o tecto, reduzir o prompt ou forçar compactação |
| recall vazio | `notes.jsonl` ausente, ou memória não injectada | `.memory(&memory)` no construtor **e** `Some(&memory)` no `Toolbox` |
| `memory.write` com `rejected` | duplicata forte (≥0.92) | normal: o protocolo de dedup rejeita; ver o `id` devolvido |
| tools `memory_ask`/`memory_write` não oferecidas | `Toolbox::new(..., None)` | passar `Some(&memory)` à T2 |
| `ToolOutcome::error` "tool negada" | guardrail/permissão/`AgentSpec` | ver `guard.decision` no log e `DenyReason` |
| sessão gravada mas contexto “esquecido” na retoma | checkpoint com `keep_from` não aplicado | aplicar `log.set_compaction(keep_from)` ([04](./04-sessoes.md) §4) |
| dois processos corrompem o mesmo JSONL | falta de lock | lock por ficheiro/serialização ([04](./04-sessoes.md) §5) |
| embeddings sempre pendentes | servidor de embeddings fora, ou dimensão ≠ `dimensions` | `ktu memory drain --status`; ver [06](./06-projecto.md) §6.3 |

## 5. Como investigar um turno

1. **Eventos do log** — `log.events()` dá a sequência completa; `log.verify()` diz se ficou
   inconsistente.
2. **Log estruturado** — o teu `Logger` recebe `kernel.error`, `provider.error`, `guard.decision`,
   `tool.result`, `memory.recall`/`memory.write`, `session.compact`, `plugin.dispatch`. Filtra por
   `record.event` (ids estáveis em `ktu_core::events::ALL`).
3. **Projeção de *wire*** — `log.derive_messages()` mostra exactamente o que o modelo viu (útil
   quando “o modelo ignorou a instrução”).
4. **`ktu.system()`** — o prompt efectivo (base + `AGENTS.md` condensado).
5. **Reprodução determinística** — repete com `FakeProvider` (mesmo script) e `MemFs`: se o
   problema não reproduz, é do provider/ambiente.

## 6. Exit codes (quando embutes via processo/CLI)

A CLI usa o contrato do projecto — replica-o na tua app se expuseres CLI:

| Código | Significado |
|---|---|
| `0` | sucesso (`StopReason` != `Error`) |
| `1` | falha (erro de configuração, provider, guardrails, IO) |
| `2` | uso inválido (argumentos/flags) |

`stdout` = **dado** (a resposta final); `stderr` = **log** (eventos). Nunca misturar.

## 7. Checklist

- [ ] `build()` e `turn()` tratados explicitamente (sem `let _ = …`).
- [ ] `outcome.is_ok()` usado para o exit code / estado da tarefa.
- [ ] Eventos `kernel.error`/`provider.error` observados (não silenciados).
- [ ] `log.verify()` vazio antes de gravar.
- [ ] Erros de tool apresentados ao modelo (não abortam o turno).
- [ ] Mensagens de erro próprias **sem segredos** e com o caminho em contexto.

Próximo: [08 — Boas práticas](./08-boas-praticas.md).
