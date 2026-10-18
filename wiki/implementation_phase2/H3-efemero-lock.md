# H3 — Modo efémero (default) + lock de turno

> **Issue:** `task_017rrpwn` · **Âncora:** [modo-efemero](../backlog/modo-efemero.md) ·
> **Desenho:** [cli-e-config](../cli-e-config.md) §1.3/§1.4 · **Pré-requisitos:** H0, H2.

## Objetivo
Inverter o default para **efémero de 1 turno** (uso #2: pipelines) e aplicar o **lock por sessão**,
para coexistência de REPL + turnos efémeros no mesmo projecto.

## Passos
1. **Efêmero por omissão** — `ktu` e `ktu <prompt>` correm 1 turno **sem persistir**;
   `ktu run --session <id>` persiste; `--resume` implica persistente.
2. **Lock por id** — `.ktu/sessions/<id>.lock` (`O_CREAT|O_EXCL`, stale 30 s, retry+jitter);
   o segundo processo espera ou recusa com `StopReason` claro.
3. **Coexistência** — efémero (sem id) e REPL (com id) não colidem; recursos partilhados são
   **append-only** (JSONL).

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_002h06sv` | Efêmero por omissão |
| `task_01w4atj1` | Lock por id (stale + retry) |
| `task_00k45q5a` | Teste de concorrência (log íntegro) |

## Gate / testes
- `ktu <prompt>` não cria `.ktu/sessions/`; `--session` cria.
- Dois `run` no **mesmo** id ⇒ um espera/recusa; o log fica **íntegro** (I1/I2).
- Mudança de comportamento documentada no `prime`/`help`.

## Riscos
- **Quebra de comportamento** (hoje `run` persiste) → documentar; `--session` mantém o antigo.
- Lock a bloquear demais → stale + retry; nunca apaga lock alheio (tática do knudge).

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **Determinismo:** o lock usa as portas `Clock`/`Rng` (stale/jitter) — **nunca** `Instant::now`
  nem `thread::sleep` directo (proibidos pelo `clippy.toml`).
- **N1:** nada de `HashMap` no registo de locks.
- **N2:** aquisição/libertação sem `unwrap`; falha = `StopReason`/erro tipado.
- **Gate:** parte de **G13** (pipeline concorrente).

## Execução (concluída)
- **Implementado:** `SessionTarget { Ephemeral, Persistent(String) }` (`session_mode.rs`);
  `RunConfig.session` passou de `String` para `SessionTarget`. `ktu run` **sem** `--session` é
  efémero (não carrega nem grava log/checkpoint; `ensure_layout` deixou de criar `.ktu/sessions`);
  `--session <id>` persiste; `--resume`/`--compact` implicam `default`. Lock de turno (`lock.rs`):
  `.ktu/sessions/<id>.lock` com `create_new` (atómico), **stale 30 s** (via porta `Clock`), recusa
  se vivo — nunca apaga lock alheio; guarda RAII.
- **Verificado:** 3 testes unitários do lock + 3 de integração via binário (lock vivo recusa, lock
  largado passa e grava, efémero não persiste). Comportamento documentado no `prime` e no
  `--session`.
- **Nota:** o plano admitia "espera **ou** recusa"; optou-se por **recusar** (sem retry/wait) — o
  lock obsoleto (30 s) é reclamado, e um lock vivo não bloqueia para sempre.
- **Gate:** `make check` + `typos` + `machete` verdes.
