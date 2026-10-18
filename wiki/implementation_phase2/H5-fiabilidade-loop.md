# H5 — Fiabilidade do loop

> **Issue:** `task_00htbspv` · **Âncora:** [consolidado](../backlog/consolidado.md) ·
> **Desenho:** [revisao-proposito](../revisao-proposito.md) §5.1 · **Pré-requisitos:** H0.

## Objetivo
Tornar o ktu usável com **qualquer** modelo (não só o de referência): edições correctas, sem turnos
falhados, com orçamento protegido. É o **Tier 1** da análise crítica.

## Passos
1. **`edit` multi-bloco atómico** (`75`) + **line-endings/BOM** (`96`).
2. **Fila de mutação de ficheiro** (`95`) + **truncagem sistemática** (`94`/G3).
3. **Continuação de leitura** (`70`) + **recusa acionável** (`68`).
4. **Deteção de eco** (`65`) + **prompt de `MaxSteps`** (`106`).
5. **Tool calls declaradas + reparação de input** (`67`/G15).

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_01e6z9mc` | `edit` multi-bloco + line-endings/BOM |
| `task_01smpnn6` | Fila de mutação + truncagem |
| `task_01jy2n6o` | Continuação de leitura + recusa acionável |
| `task_0057o5zl` | Deteção de eco + `MaxSteps` |
| `task_00ueo4nn` | Tool calls declaradas + reparação |

## Gate / testes
- `edit` multi-bloco: aplica tudo ou **nada** (atómico); CRLF/BOM normalizados.
- Truncagem: output grande → ponteiro + prévia (dentro do teto, I14).
- Eco: resposta repetida do modelo é detectada (`StopReason`).
- Tool call em texto / JSON quebrado reparado quando **inequívoco**.

## Riscos
- Reparação a "adivinhar" → só quando **inequívoca**, profundidade limitada.
- Truncagem a comer evidência → ponteiro + ficheiro, nunca cortar (I14).

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **proptest (`13`):** parsers (tool-call declarada, line-endings/BOM, reparação) testados com
  entradas arbitrárias.
- **N9:** onde a ordem é contrato (dedup, resultados), `sort()` **estável**; `sort_unstable` só
  onde a ordem não é observável.
- **N1:** dedup/estado em `BTreeMap`/`BTreeSet`.
- **N2:** a edição atómica nunca usa `unwrap`; falha parcial ⇒ rollback (nada aplicado).
- **Gate:** **G10** (cadeia longa) e **G11** (multi-modelo).

## Execução (concluída — âmbito reduzido)
- **`edit` multi-bloco + BOM/CRLF:** `ktu-tools::text` (puro) detecta BOM/*line-ending*, calcula em
  LF e **restaura**; `apply_edits` aplica N blocos **atomicamente** (erro com o bloco que falhou).
- **Truncagem:** `ktu-tools::truncate` (linhas + bytes, sem linhas parciais); `bash` guarda o
  output completo em `.ktu/out/<tool>-<hash>.txt` e devolve o ponteiro; `read_file` continua com
  `offset` e devolve `read.missing` legível.
- **Recusa acionável:** `DenyReason::remedy()` — todos os motivos têm remédio; a mensagem de
  negação para o modelo inclui-o.
- **Eco + `MaxSteps`:** `StopReason::Echo` (nudge + 1 retry; nota visível — **nunca** aceite); o
  passo final de `MaxSteps` corre **sem tools** com pedido de resumo (nunca executa tools).
- **Tool calls declaradas + reparação:** `ktu-core::agent::toolcall` decodifica chamadas em texto
  (`<tool_call>`, cerca, JSON) **contra o catálogo fechado** (o texto declarado não entra no log) e
  repara argumentos guiados pelo schema (profundidade ≤ 6, só inequívoco).
- **Adiado (A6):** fila de mutação por caminho (as tools são sequenciais — sem concorrência);
  `remedy` por regra; âncoras próximas no `edit`.
- **Gate:** `make check` + `typos` + `machete` verdes; +4 testes de loop (eco/`MaxSteps`/declaradas)
  e +11 unitários (`text`, `truncate`, `toolcall`).
