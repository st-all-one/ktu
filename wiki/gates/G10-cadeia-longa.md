# G10 — Cadeia longa numa mesma sessão ⭐

- **Veredicto:** ✅ **verde** (7/7) · **Artefactos:** `artifacts/G10/` · **Data:** 2026-10-18
- **Critério:** [`GATES.md`](../implementation_phase2/GATES.md) §G10.

## Cenário

Uma tarefa real de código num módulo pequeno (`.ktu/gates/g10/alvo/`, copiado de
`fixtures/g10-project/`): **corrigir um bug** (`soma` devolve `a` em vez de `a + b`), **acrescentar
um módulo** (`media`) com testes, **documentar** e **verificar** — em **15 turnos da mesma sessão**,
com `cargo test` a ser corrido pelo próprio agente.

```bash
ktu run --batch batch.jsonl --sequential --provider scripted \
    --script script.jsonl --session g10 --max-steps 4
```

O guião (`fixtures/g10-script.jsonl`, 30 passos: 15 chamadas de tool + 15 respostas finais) é um
**provider-gravado**: percorre a tarefa sem modelo, de forma determinística (I6).

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | a cadeia corre de ponta a ponta | exit 0 |
| 2 | ≥15 turnos **na mesma sessão** | `turn_start` = 15 (= itens do lote) |
| 3 | todos os turnos fecham com `StopReason` normal (I1/I5) | `turn_end` = 15, todos `end_turn` (sem estagnação nem erro) |
| 4 | a instrução inicial fica no contexto (I14) | 0 compactações no log |
| 5 | o **artefacto** está correto | `cargo test` verde no repo alvo |
| 6 | a correção foi feita e o módulo ligado | `src/lib.rs` contém `a + b` e `pub mod media;` |
| 7 | o módulo novo tem teste | `media_de_tres` + `media_de_lista_vazia` |

**Não-estagnação:** leituras por ficheiro — `src/lib.rs`=4, `src/media.rs`=1, `README.md`=1,
`Cargo.toml`=2. Nenhum turno terminou por estagnação (o detector do loop só dispara em repetição
**consecutiva** da mesma assinatura).

## Artefactos

| Ficheiro | O que é |
|---|---|
| `g10.jsonl` | o log da sessão (15 turnos, 15 `tool_result`) |
| `src-lib.rs`, `src-media.rs` | o artefacto final do alvo |
| `cargo-test.txt` | a saída de `cargo test` no alvo (verde) |
| `script.jsonl` | o guião corrido (cópia da fixture) |

## Desvios (registados)

- **O modelo de referência não é corrido neste host.** Medição: com Ornith-1.5-9B Q4_K_M (CPU-only,
  `scripts/run-server.sh`), **uma** completion levou ~8 min e o turno de 1 passo excedeu 10 min —
  uma cadeia de 15 turnos seria inviável (horas). O gate prova a **mecânica** da cadeia (contexto,
  tools, correção, fecho de turnos, artefacto correto) com um guião determinístico; a barra de
  **qualidade** contra goose/pi/opencode é a bateria `p1`–`p4` (G4).
- A tarefa é **pequena e fixa** de propósito: o artefacto tem de ser verificável por um teste
  (`cargo test`), e o guião tem de ser revisto como código.
- **Defeito encontrado por este gate:** `--batch --sequential` não acumulava contexto (só o último
  turno ficava no log). Corrigido em `crates/ktu-cli/src/batch.rs` (retoma a partir do 2.º item) e
  travado pelo teste `sequential_batch_accumulates_context_in_one_session`.
