# G16 — *Steering* durável e idempotente (M7)

- **Veredicto:** ✅ **verde** (9/9) · **Artefactos:** `artifacts/G16/` · **Data:** 2026-10-18
- **Item:** [F1](../implementation_phase3/F1-steering-duravel.md) ·
  **Critério:** [GATES da Fase 3](../implementation_phase3/GATES.md) §G16.

## Cenário

Uma linha escrita **durante** um turno, e a mesma sessão retomada duas vezes:

```bash
ktu session s --provider mock --max-steps 4   # "ola\nfaz antes X\n/quit\n"
ktu session s --provider mock --max-steps 4   # "continua\n/quit\n"
ktu session s --provider mock --max-steps 4   # "outra vez\n/quit\n"
```

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | a linha é **admitida** no log durante o turno | `steer_queued` = 1 |
| 2 | a admissão vem **antes** da entrega (sem entrega no mesmo turno) | `steer` = 0 |
| 3 | I1/I2 limpos depois do turno | `ok` |
| 4 | a admissão pendente é **entregue no turno seguinte** | `steer` = 1 |
| 5 | o texto entregue é o que foi admitido | `"text":"faz antes X"` |
| 6 | **exactamente uma vez** (3 corridas na mesma sessão) | `steer` = 1 |
| 7 | I1/I2 limpos no fim | `ok` |
| 8 | o mesmo `id` não duplica (**kernel**) | 2 testes M7 passam |
| 9 | o mesmo `id` não duplica pela **porta HTTP** | 1 teste passa |

## O que o gate prova (e o que não prova)

- **Prova:** a admissão é registada **antes** da entrega; uma admissão pendente sobrevive ao
  fim do turno (e, pelo mesmo caminho, a um *crash*) e é entregue **uma só vez** no turno
  seguinte; repetir a corrida não duplica; o `id` do front decide (M7).
- **Não prova:** que a mensagem sobrevive a um *crash* **antes** da primeira fronteira. A
  admissão é gravada na fronteira do loop (o kernel não tem I/O — a gravação do log é do
  actor), pelo que a janela coberta é *admissão registada → morte antes da entrega*.

## Artefactos

`s.jsonl` (o log da sessão) · `m7-tests.txt` (os testes do kernel e da porta).

## Notas

- **Semântica:** a mensagem entra no passo **seguinte** ao da admissão (e, se o turno acabar
  antes, no turno seguinte — é o *follow-up*). É o preço da admissão durável; antes era
  entregue no mesmo passo em que chegava, sem nunca ficar registada.
- O `id` é da **borda** (`repl-<n>` no REPL, `http-<n>` no servidor quando o cliente não
  envia nenhum): o kernel **não** gera ids (I6).
- A **projeção** (`derive_messages`) descarta um `Steer` cujo `id` já apareceu e o
  `SteerQueued` nunca é projetado — *model-visible ⟺ logged*.
