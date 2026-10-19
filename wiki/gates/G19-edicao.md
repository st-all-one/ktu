# G19 — Edição sob concorrência (fila, remédio, âncoras)

- **Veredicto:** ✅ **verde** (8/8) · **Artefactos:** `artifacts/G19/` · **Data:** 2026-10-18
- **Item:** [F4](../implementation_phase3/F4-fiabilidade-edicao.md) ·
  **Critério:** [GATES da Fase 3](../implementation_phase3/GATES.md) §G19.

## Cenário

Mutações concorrentes ao **mesmo** caminho (ao nível das tools, onde a concorrência existe),
uma regra `deny` com `remedy` próprio, e um `edit_file` com âncora errada — tudo pelo binário,
com guião determinístico.

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | o mesmo caminho **serializa** e nenhuma edição se perde | 2 testes das tools verdes |
| 2 | o **processo** partilha uma fila entre sessões | 1 teste do hub verde |
| 3 | caminhos **distintos** correm em paralelo | 2 testes da fila verdes |
| 4 | a recusa traz o remédio **da regra** | `pede autorizacao ao responsavel` |
| 5 | a recusa **não perde** o remédio estrutural do motivo | `respeite o guardrail` |
| 6 | a âncora errada sugere a linha correcta | `3: fn soma(a: u32) -> u32 {` |
| 7 | a edição falhada **não** altera o ficheiro | intacto |
| 8 | a escrita pelo binário deixa o ficheiro final esperado | `"final\n"` |

## Artefactos

`queue-tests.txt` (as duas corridas de teste da fila) · `remedy.txt` (o log com os dois
remédios) · `editing.log` (a sugestão com número de linha) · `final.txt` (o estado final).

## Notas

- **A fila é do processo, não do turno.** O `ktu serve` corre **um actor por sessão** e o
  `--batch --parallel` **uma thread por worker** — cada turno tem a sua T2 e, com a fila dentro
  do `FsTools`, cada um tinha a sua fila: duas sessões do mesmo processo editavam o mesmo
  ficheiro sem se verem. A fila é agora um `Arc` no `Inner` do hub (uma por lote), injectada em
  `execute_turn` → `Toolbox` → `FsTools`; a chave canónica partilhada é o que o teste do hub
  prova (uma só entrada para as duas sessões).
- **A fila é exercitada onde a concorrência existe:** dentro de *um* turno a T2 serve um pedido
  de cada vez. Os testes usam `multi_thread` (8 tarefas ao mesmo caminho nunca estão na secção
  crítica ao mesmo tempo; caminhos distintos sobrepõem-se, provado por barreira; e **duas
  instâncias de tools** que partilham a fila não perdem edições).
- **Fora do âmbito (registado):** dois **processos** no mesmo projecto (o `TurnLock` só trava o
  mesmo id de sessão) e a **escrita atómica** (temp + `rename`) — `pendencias.md` D13/D14.
- **A vez é pedida antes da leitura** no `edit_file`: ler fora da fila e gravar dentro perdia a
  alteração de outra edição ao mesmo ficheiro (o teste `concurrent_edits_...` falharia).
- **A sugestão de âncoras** compara a primeira linha da âncora com cada linha do corpo
  (Levenshtein limitado) e devolve as 3 melhores **em ordem de linha** (N1), sempre com o
  número da linha. Uma âncora ambígua (que existe) não sugere nada.
