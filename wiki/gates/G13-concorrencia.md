# G13 — Pipeline concorrente (uso #2)

- **Veredicto:** ✅ **verde** (6/6) · **Artefactos:** `artifacts/G13/` · **Data:** 2026-10-18
- **Critério:** [`GATES.md`](../implementation_phase2/GATES.md) §G13.

## Cenário

N `ktu` independentes no **mesmo** projecto: um lote `--batch` com 4 prompts (paralelo por omissão,
4 *workers*) e dois turnos no **mesmo id de sessão** (lock).

```bash
ktu run --batch batch.jsonl --provider mock --max-steps 2      # 4 itens em paralelo
ktu run ola --provider mock --session ocupada                  # com lock vivo → recusa
```

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | o lote paralelo termina com sucesso | exit 0 |
| 2 | `stdout` ordenado pelo índice de entrada | `[0 [1 [2 [3` |
| 3 | cada item corre na sua sessão (sem corrupção) | 1 `turn_start` por `batch-<i>.jsonl` |
| 4 | I1/I2 limpos em todos os logs | `ok · ok · ok · ok` |
| 5 | lock **vivo** recusa o segundo processo, com razão clara | exit 1, `ktu: sessao ocupada ocupada por outro processo` |
| 6 | lock largado deixa o turno correr | exit 0 |

A integridade (I1/I2) é verificada **no artefacto**: o gate emparelha cada `assistant.tool_calls`
com o `tool_result` do mesmo id e conta os `turn_start`/`turn_end`.

## Artefactos

`batch-0.jsonl` (um dos quatro logs independentes) · `stderr-lock.txt` (a recusa do lock).

## Notas

- O lock é **por id de sessão** (`.ktu/sessions/<id>.lock`): aquisição atómica (`create_new`), lock
  obsoleto (> 30 s) reaproveitado, lock vivo **nunca** é apagado por outro processo. Os itens
  paralelos usam ids distintos (`batch-<i>`), pelo que não colidem por construção.
- Os recursos partilhados (memória) são *append-only* (JSONL) — seguros por construção.
- Travado em `make check` por `crates/ktu-cli/tests/batch.rs` (ordem + isolamento) e
  `crates/ktu-cli/tests/lock.rs` (lock vivo/liberado).
