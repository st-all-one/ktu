# G15 — Eixo de memória (knudge)

- **Veredicto:** ✅ **verde** (9/9) · **Artefactos:** `artifacts/G15/` · **Data:** 2026-10-18
- **Critério:** [`GATES.md`](../implementation_phase2/GATES.md) §G15.

## Cenário

A memória **sobrevive** à sessão e é consultável; o índice é derivado; a projeção do log é
*read-only*; o lado do knudge (`ask --agent`, fonte JSONL) cumpre o contrato (I17).

```bash
ktu run "le a.txt" --provider mock --session s1     # grava (gatilho session_end)
ktu run "conteudo" --provider mock --session s2     # sessão nova: recorda
ktu memory list · ktu memory export · ktu memory project
rm -rf .agents/memory/.idx && ktu memory export     # derivado descartável
cargo test --manifest-path crates/knudge/Cargo.toml -p knudge-cli --test ai
```

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | a sessão grava memória **sem depender do modelo** | `memory.write` no log (gatilho `session_end`) |
| 2 | a nota tem **proveniência** | `notes.jsonl` com `"agent":"ktu"` |
| 3 | `ktu memory list` mostra a nota | 4 linhas |
| 4 | **sessão nova recupera** | `memory.recall` no log da sessão 2 |
| 5 | apagar o índice derivado não perde nada (I17) | `export` igual antes/depois (314 bytes) |
| 6 | a projeção escreve notas derivadas com proveniência | `session-log.jsonl` com `"agent":"ktu"` |
| 7 | a projeção **não altera o registo** (fonte read-only) | `notes.jsonl` inalterado |
| 8 | knudge: `ask --agent` filtra por proveniência (fonte JSONL) | `knudge-cli --test ai`: 7 passed |
| 9 | knudge: fonte JSONL read-only, dedup e linhas más | `knudge-core corpus`: 15 passed |

## Artefactos

`notes.jsonl` (o registo, com proveniência) · `session-log.jsonl` (a projeção derivada) ·
`stderr-recall.txt` (o log de diagnóstico com `memory.recall`) · `knudge-ai.txt` /
`knudge-corpus.txt` (as saídas dos testes do knudge).

## Notas

- O **recall** acontece no **arranque** de sessão (log ainda vazio) e entra no *system*, não no
  registo de eventos: é o que mantém o prefixo estável (cache preservada) — a evidência está no log
  de diagnóstico (`memory.recall`), que o gate captura.
- A escrita é **automática** no fim de um turno bem sucedido (`StopReason::EndTurn`); o modelo
  também pode gravar com a tool `memory_write` (uma afirmação por nota, com `dedup` do knudge:
  < 0,75 cria · 0,75–0,92 *merge* · ≥ 0,92 rejeita).
- Dois agentes a escrever não colidem: o registo é *append-only* e o `dedup` é por conteúdo; a
  proveniência (`agent`, `activity`) distingue as escritas — coberto pelos testes do knudge
  (`ask --agent ktu` vs `--agent goose`).
- **Desvio:** o binário `kd` não é reimplementado no gate; a parte do knudge é provada correndo os
  testes do workspace do knudge (é lá que o contrato vive).
