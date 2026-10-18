# H7 — Eixo de memória no knudge

> **Issue:** `task_00104ugh` · **Âncora:** [kd-ai](../backlog/kd-ai.md) ·
> **Desenho:** [knudge-eixo-memoria](../knudge-eixo-memoria.md) · **Pré-requisitos:** H2.
>
> **Estado: concluída.** Knudge **0.6.2** (decisão `D218` + ADR + wiki + `prime`/`init`/`SKILL`);
> `ktu-memory` consome `parse_jsonl`; projeção em `ktu memory project`. Adiado: registo de agentes
> (`.knudge/agents.toml`) — só com ≥2 consumidores.

## Objetivo
Fazer do knudge o **eixo único de memória, agent-agnostic**: uma `Source` JSONL read-only, filtros
por proveniência, contrato de projeção e a área `kd ai`. A memória deixa de ser do agente e passa a
ser do **projecto/equipa**.

## Passos
1. **`Source` JSONL read-only** no `knudge-core` (`NotesDir` default | `Jsonl`), reusando
   `Note`/`Frontmatter`/`jsonl::json`; `corpus` passa a ler de uma `Source`.
2. **`ask --agent`/`--activity`** — filtros de proveniência (a proveniência já é gravada, D207).
3. **Contrato de projeção** (doc) + projeção do log de sessão em **notas derivadas**
   (`activity=derive`), read-only.
4. **`kd ai [memory|session-log]`** — área fina sobre a `Source` (posicional, default consulta).
5. **`ktu-memory` consome a `Source`** — elimina a reimplementação da leitura do JSONL.

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_01htyks4` | `Source` JSONL read-only (knudge-core) |
| `task_01h2dvg9` | Filtros `ask --agent/--activity` |
| `task_012mair7` | Contrato de projeção + projeção do log |
| `task_00kzk1n0` | `kd ai [memory\|session-log]` |
| `task_00221fiu` | `ktu-memory` consome a `Source` |

## Gate / testes
- `Source` JSONL: ler `notes.jsonl` ⇒ mesmas notas que o `ktu-memory` (golden).
- `ask --agent ktu` filtra por proveniência; determinismo `(score desc, id asc)`.
- Projeção é **read-only e derivada**; o log continua a ser do agente (I3).
- Concorrência: dois agentes a escrever notas diferentes não colidem; mesma nota ⇒ dedup.

## Riscos
- Acoplar o knudge ao schema `Event` do ktu → **proibido**: o contrato é o **documento**.
- Dupla fonte da verdade → o knudge **indexa**; o agente **possui** a fonte.
- Scope creep no knudge → **YAGNI condicional:** promover só com **≥2** agentes.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **Nota de fronteira:** o `knudge-core` é **crate externo** — os seus `unwrap`/`HashMap` internos
  **não** estão sob os lints do ktu. O que escrevemos **no ktu** (o adaptador/`Source`) cumpre-os.
- **proptest:** o parse do JSONL de notas (`{frontmatter, body}`) testado com linhas arbitrárias;
  linha malformada = *skip + warning* (nunca panic).
- **N1:** índice/dedup em `BTreeMap`; `ask` determinístico `(score desc, id asc)`.
- **Determinismo:** rebuild do `index/` ⇒ recall idêntico (golden).
- **Gate:** **G15** (eixo de memória).

## Prova (G15 — eixo de memória)

Executado com o knudge 0.6.2 e o ktu do H7, num projecto temporário:

```
$ ktu session s1 --provider mock --max-steps 4   # turno real
$ ktu memory project --root .                    # s1	1
$ kd ai session-log dedup
meta_00ik1r8x|[s1] como funciona o dedup da memoria|0.016

$ sha256sum .ktu/sessions/s1.jsonl               # adb3c8732f8fb1ba
$ ktu memory project --root .                    # re-projectar
$ sha256sum .ktu/sessions/s1.jsonl               # adb3c8732f8fb1ba  (log intocado: read-only)
$ wc -l < .agents/memory/session-log.jsonl       # 1                (idempotente)
$ rm -rf .knudge/.idx && kd ai session-log dedup # id e score idênticos (rebuild determinístico)
```

| Critério de G15 | Prova |
|---|---|
| A `Source` JSONL lê o store do agente | `corpus::tests::jsonl_source_reads_the_same_corpus_as_notas_dir` (mesmo índice e grafo) + `kd ai memory` |
| `ask --agent` filtra por proveniência | `tests/ai.rs::ask_filters_by_agent_and_activity` (+ `--activity`, vírgula, `--params`, `--rank`) |
| A projeção do log é **read-only** e derivada | sha do log inalterado; a nota vai para `session-log.jsonl` com `activity=derive` |
| Recall idêntico após rebuild | `.idx/` apagado ⇒ mesmos hits (ordem do `Source` é por `id`) |
| Mesma nota ⇒ dedup | `project::tests::projection_is_idempotent_on_disk` + knudge `Source` dedup-on-read (D26) |
