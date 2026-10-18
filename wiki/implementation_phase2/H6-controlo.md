# H6 — Controlo determinístico

> **Issue:** `task_01s9n99d` · **Âncora:** [permission-store](../backlog/permission-store.md) ·
> **Desenho:** [revisao-proposito](../revisao-proposito.md) §3 (M5) ·
> **Pré-requisitos:** H2, H5.

## Objetivo
O controlador age **sobre** o agente, sem depender da obediência do modelo: permissões por tool,
steering/admissão durável, e política como dado (seam).

## Passos
1. **Permissões por tool** `Always/Ask/Never` **memorizadas** (`21`/G7); integra com `Escalate` (I10).
2. **Steering / admissão durável** (`42`/G14) — `steer` no próximo limite seguro; `queue`;
   inbox durável e idempotente.
3. **Política como dado** (`1`/G8, **seam**) — `Guard::evaluate` sobre factos tipados; recusa
   acionável ligada à regra (`remedy`).

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_003hj4fg` | Permissões por tool + store |
| `task_00wjk7md` | Steering / admissão durável |
| `task_0138yzj6` | Política como dado (seam) + `remedy` |

## Gate / testes
- `Deny`/`Escalate` nunca ignorados por nenhum caminho (incl. subagentes) — fixture adversarial.
- Permissão memorizada sobrevive a `resume`; é **project-bound**.
- Steering chega **sem** matar o turno; inbox idempotente.

## Riscos
- Segundo kernel (viola M2) → reutilizar `Guard::evaluate`; nada paralelo.
- "Agir sobre o agente" virar imprevisível → tudo **logado** (`StopReason`, `guard.decision`).

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **M2:** **nenhum** segundo kernel — reutilizar `Guard::evaluate`; nada de lógica paralela.
- **N1:** a política (dado) usa `BTreeMap` (factos ordenados, "última regra vence" determinístico).
- **N8:** decisões (`Allow`/`Deny`/`Escalate`) são **enums**, não bools.
- **N2:** `remedy` é dado tipado; sem `unwrap` na avaliação.
- **Gate:** contribui para **G14** (piso de guardrails) e **G10** (controlo durante a cadeia).

## Execução (concluída — âmbito reduzido)
- **Permissões por tool:** `ktu-core::permission` (`Always/Ask/Never` em `.agents/permissions.toml`,
  `Store` em `BTreeMap`, project-bound ⇒ sobrevive a `resume`). O `PermissionGuard` é o **último**
  elo da cadeia (`structural → declarative → permission`): `Ask` → `Escalate`, `Never` →
  `Deny(Policy)` — nunca **alarga** (I16). CLI `ktu permission ls|set|unset`.
- **Política como dado:** `Guard::remedy(&Fact)` por elo (structural/declarative/permission/chain);
  o guardrail declarativo passa a aceitar `decision: deny|escalate` e `remedy` próprio. O *seam*
  `Guard::evaluate` sobre `Fact` continua a ser o **único** caminho (M2).
- **Verificado:** `never` → `negado: policy — …`; `ask` → `requer aprovacao: …`; regra declarativa
  `escalate`/`deny` com remédio próprio; permissão sobrevive a `resume`. +5 unit +3 integração.
- **Steering (feito):** o REPL entrega mensagens **durante** o turno (o input é lido numa thread
  dedicada e drenado sem bloquear entre eventos); a porta ganhou `steer` (`Host`/`HttpPort` +
  rota `/v1/session/{id}/steer`); o kernel entrega-as numa **fronteira segura** (início de passo)
  e **loga-as** como `Event::Steer` (*model-visible ⟺ logged*). O enfileiramento durável +
  idempotência ficam em **A7**.
- **Gate:** `make check` + `typos` + `machete` verdes.
