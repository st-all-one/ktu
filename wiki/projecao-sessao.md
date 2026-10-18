# Projecção do log de sessão em notas derivadas

> Contrato do **ktu** para a via *projetada* do [knudge como eixo de memória](knudge-eixo-memoria.md)
> (decisão `D218` do knudge). Implementado por `ktu memory project`; consultável por
> `kd ai session-log`.

---

## 1. Princípio

O log da sessão (`.ktu/sessions/<id>.jsonl`) é a **fonte da verdade** da sessão (I3) e **continua a
ser do agente**. A projecção é **read-only** sobre ele e **derivada**: produz notas no schema do
knudge, que as **indexa**. Nunca há segunda verdade — o log manda, a nota é uma vista.

O knudge **não** conhece o schema `Event`: quem projeta é o **ktu**. O contrato entre os dois é o
**documento** (uma nota do knudge), não o evento.

## 2. A forma da nota

Um **turno** (`TurnStart`…`TurnEnd`) vira **uma** nota:

| Campo | Valor |
|---|---|
| `type` | `meta` |
| `statement` | `[<sessão>] <pedido do utilizador>` — colapsado numa linha e truncado a **120 escalares** |
| `body` | a resposta final do turno + `Ferramentas: a, b` (nomes distintos, ordenados) |
| `anchors` | caminhos tocados pelas ferramentas (`path`/`file_path` dos argumentos JSON), ≤ **8**, ordenados |
| `tags` | `session-log` |
| `provenance` | `agent = ktu`, `activity = derive` |

O prefixo `[<sessão>]` existe para que turnos iguais em sessões diferentes **não colidam** no `id`
(endereçado por conteúdo). Um turno **aberto** (sem `TurnEnd`, ex.: processo morto a meio) não é
projectado.

## 3. Determinismo e idempotência

- A ordem dos turnos é a do log; ferramentas e âncoras são **conjuntos ordenados** (`BTreeSet`).
- `ktu memory project` é **idempotente**: uma nota cujo `id` já exista com o mesmo `body_hash` não é
  reescrita (a escrita é `temp → fsync → rename`, atómica).
- Re-projectar não duplica nem altera notas já gravadas; projectar de novo depois de um turno novo
  acrescenta **só** esse turno.

## 4. CLI

```
ktu memory project [--root P] [--session ID] [--out PATH]
```

| Flag | Efeito |
|---|---|
| `--root P` | raiz do projecto (por omissão, o directório actual) |
| `--session ID` | projecta **só** essa sessão (por omissão, todas as de `.ktu/sessions/`) |
| `--out PATH` | destino do JSONL derivado (por omissão, `.agents/memory/session-log.jsonl`) |

Saída: `stdout` = uma linha por sessão (`<id>\t<notas>`); `--json` =
`{out, written, sessions:[{id, notes}]}`. Sessão inexistente com `--session` ⇒ `exit 1`.

## 5. Como consultar

```bash
ktu memory project                       # projeta todas as sessões
kd ai session-log "por que trocamos o modelo"   # recall entre sessões
kd ai session-log                        # lista as notas derivadas
kd ai session-log --source .agents/memory/session-log.jsonl "parser"
```

A consulta é **inteligente** (BM25 + âncoras + grafo + RRF), não sequencial: responde "o que fizemos
sobre X?" por significado, não por leitura de logs. Detalhe do lado do knudge em
[`fontes.md`](../crates/knudge/wiki/specs/fontes.md) e `wiki/usage/20_ai.md`.

## 6. Proveniência das escritas directas

As notas que o próprio agente grava (`memory_write`) levam também proveniência
(`agent = ktu`, `activity = write`), pelo que `kd ask --agent ktu` / `--activity write|derive`
filtram por **quem** produziu cada nota — o mesmo standard para todos os agentes.

## 7. Testes

| Comportamento | Teste |
|---|---|
| Um turno ⇒ uma nota (afirmação, corpo, âncoras, tags, `meta`) | `ktu-memory::project::tests::one_turn_becomes_one_derived_note` |
| Turno aberto ignorado; ordem dos turnos | `open_turn_is_not_projected_and_turns_are_ordered` |
| Colapso + truncagem a 120 escalares | `statement_is_collapsed_and_truncated_to_120_scalars` |
| Idempotência da escrita | `projection_is_idempotent_on_disk` |
| CLI: nota derivada com proveniência, idempotência, `--session`/`--out`, sessão ausente | `crates/ktu-cli/tests/memory_project.rs` |
