# Estudo — `kd ai` (memória/logs/sessão do agente no knudge)

> Tangente: acrescentar ao **knudge** (core + CLI) uma área para **cadastrar e consultar** a
> memória/logs/sessão do **agente**, algo como
> `kd ai [memory|session-log] [--add|--init|--anchor|--force]` (default = consulta), na mesma
> filosofia posicional do `kd`. Duas perguntas: **(1) é viável?** **(2) traz ganhos significativos
> ao ecossistema do ktu?** Data: 2026-10-17. Fontes: código do `crates/knudge` (v0.6.1) e
> `crates/ktu-memory`.

---

## 0. O que existe hoje (medido)

### No knudge
| Peça | Formato | Papel |
|---|---|---|
| `notas/<tipo>/<id>.md` | **TOON** (frontmatter) + corpo | fonte da verdade (`Store::note_path`) |
| `eventos/events*.jsonl` | JSONL `{op, note_id, at, data}` | **auditoria** de escrita (não conversa) |
| `corpus` | lê `notas/` **uma vez** → `Index` + `Graph` | retrieval |
| `handoff`/`rewind` | contexto derivado (`context_id` = hash) | handoff de sessão |
| `Knudge`/`KnudgeBuilder` | `knowledge_dir` **configurável** | fachada de incorporação |
| `jsonl::json` | codec JSON canónico | base do JSONL |

**Chave:** o `Store` lê **`.md` com frontmatter TOON**, não JSONL. O `corpus` deriva índice+grafo
desse diretório.

### No ktu
| Peça | Formato | Papel |
|---|---|---|
| `.agents/memory/notes.jsonl` | JSONL `{frontmatter, body}` — **com os tipos do knudge** (`Note`/`Frontmatter`/`Value`) | memória do agente |
| `.ktu/sessions/<id>.jsonl` | JSONL de `Event` (`event: turn_start|assistant|tool_result|guard_decision|turn_end|…`) | log da sessão (**fonte da verdade**, I3) |
| `ktu-memory` | adaptador: lê `notes.jsonl` → `Vec<Note>` → `Index`/`Graph` do knudge | motor de memória |

**Observações críticas:**
1. A **memória do ktu já fala o schema do knudge** (usa `Note`/`Frontmatter`), mas em **JSONL**, não
   em `notas/*.md`.
2. O **log de sessão** é um **domínio diferente**: não são notas; são eventos de conversa. O
   `eventos/` do knudge é auditoria de escrita — **não** conversa.

---

## 1. Pergunta 1 — é viável atualizar o knudge nesta frente?

### 1.1 `memory` (o caso JSONL) — **VIÁVEL, baixo esforço**
A memória do ktu **já** usa os tipos do knudge; só a **codificação em disco** difere (JSONL vs
`.md`). O `ktu-memory/record.rs` já demonstra o codec (`{frontmatter, body}` por linha).

Falta ao knudge: um **`Source`** (fonte de corpus) que leia **JSONL de notas** além do diretório
`notas/`. Isto é:
- **pequeno**: reusa `Note::new`, `Frontmatter::from_value`, `jsonl::json::decode`;
- **limpo**: não muda o schema nem o `Store` de escrita;
- **determinístico**: `list` ordena por id; `Index::build` reordena por id (igual ao `corpus`).

**Veredicto:** viável, aditivo, sem quebra. É o caso que o pedido chama de "focar no JSONL".

### 1.2 `session-log` — **VIÁVEL apenas como projecção (read-only)**
O log de sessão **não** é um corpus de notas: é uma sequência de eventos com semântica própria
(`turn_start`, `tool_result`, `guard_decision`…). Há dois caminhos:

- **(a) Projecção pelo agente** (recomendado): o **ktu** projeta os seus eventos num **documento**
  (ex.: `statement` + `body`) que o knudge indexa como uma nota **derivada** — o knudge **não**
  conhece o schema do agente. O knudge trata-o como **documento opaco**.
- **(b) Domínio novo** (não recomendado): o knudge aprende o schema `Event` do ktu → **acoplamento**
  (o knudge passa a depender do formato de cada agente) + **scope creep** (o knudge vira *store de
  logs de agente*).

**Veredicto:** viável **só** como (a) — fonte **read-only** projetada pelo agente. Como (b), é um
novo domínio que dilui a identidade do knudge ("memória de projecto").

### 1.3 A área `kd ai` — **VIÁVEL (conveniência fina)**
`kd ai memory` = `kd ask` sobre a **fonte JSONL**; `kd ai session-log` = `kd ask` sobre a fonte
projetada. A área é uma **camada fina** sobre o motor existente (`Index`/`recall`/`Graph`), com a
filosofia posicional (`[memory|session-log]`, default consulta; `--add`/`--init`/`--anchor`/`--force`).

**Veredicto:** viável, **se** existir o `Source`. Sem o `Source`, é reimplementação.

---

## 2. Pergunta 2 — traz ganhos significativos ao ecossistema do ktu?

### 2.1 Ganhos
| Ganho | Para o ktu | Para o ecossistema |
|---|---|---|
| Uma **fonte JSONL** no knudge | o `ktu-memory` deixa de reimplementar a leitura | **qualquer** agente pode registar a sua memória |
| `kd ai memory` (consulta unificada) | o `kd` passa a consultar a memória do ktu | uma superfície de consulta para todos |
| `kd ai session-log` (projeção) | **pesquisa entre sessões** ("o que fizemos sobre X?") | idem, para qualquer agente |
| `--anchor` (código ↔ memória) | liga notas ao código (já existe no ktu) | reuso directo |

O ganho **real** não é "o ktu passa a ter isto" (o ktu **já** tem `ktu-memory` e o log), é **o
knudge virar o eixo único de memória** — memória + logs + sessões — para **vários** agentes
(ktu, goose, pi, opencode). Isso é um ganho de **ecossistema**, não de ktu.

### 2.2 Contra-argumentos (crítica)
1. **Para o ktu sozinho, o ganho é moderado:** o `ktu-memory` já usa o motor do knudge **em
   memória** (via a porta `Memory`). A pesquisa do log de sessão pode ser feita **no ktu**, com o
   mesmo motor, **sem** tocar no knudge. Ou seja, o knudge **não é necessário** para o ktu
   consultar a sua memória/log.
2. **Dupla fonte da verdade (risco sério):** se o knudge **ingerir** o log de sessão como notas,
   passa a haver **duas** fontes (o log do ktu + a nota derivada) — viola o princípio do próprio
   ktu ("uma só verdade", I3). A projeção tem de ser **read-only e derivada**.
3. **Scope creep no knudge:** o knudge tem a sua própria disciplina ("cada subsistema com um
   consumidor"). O consumidor aqui é o ktu — mas o ktu **já** tem o motor. O knudge só ganha se
   **mais de um** agente usar a fonte.
4. **A projeção precisa de um schema:** o knudge tem de definir o que é um "documento de log"
   (`statement`? `type=meta`? `type=snippet`?) — decisão de schema que **afeta** o retrieval.

---

## 3. Desenho recomendado (se avançar)

### 3.1 No knudge-core — o mínimo
- **`Source`** (fonte de corpus): `NotesDir` (default, actual) **ou** `Jsonl { path }` (read-only).
  O `corpus` passa a ler de uma `Source`; `Index`/`Graph` inalterados.
- **`kd ai`** (CLI): área fina — `kd ai memory [--add|--init|--anchor|--force]` e
  `kd ai session-log […]`, default = consulta via `ask`. Reusa `--params`/`--json`/envelope.

### 3.2 No ktu — a projecção
- O ktu **projeta** o log de sessão num JSONL de documentos (`{frontmatter, body}`) **derivado**
  (ex.: um documento por turno ou por decisão), com `--anchor` a apontar o código. **Nunca** o log
  como nota canónica.

### 3.3 Regra de ouro
> O knudge **indexa** (derivado); o agente **possui** a fonte. O log de sessão continua a ser do
> ktu (I3); o knudge só o **lê**.

---

## 4. Veredicto

| Pergunta | Resposta |
|---|---|
| **1. É viável?** | **`memory`: SIM** (baixo esforço, aditivo, reusa o schema). **`session-log`: SIM, mas só como projeção read-only** — nunca como domínio novo. **`kd ai`: SIM**, como área fina sobre o `Source`. |
| **2. Ganhos significativos?** | **Para o ktu sozinho: MODERADO** (o `ktu-memory` já usa o motor; a pesquisa do log pode ficar no ktu). **Para o ecossistema: ALTO** (o knudge vira o eixo único de memória+logs para vários agentes). |

### Recomendação
1. **Fazer primeiro o mínimo no knudge:** o **`Source` JSONL read-only** — é o caso "JSONL" e o
   único ganho **estrutural** (elimina a reimplementação da leitura no `ktu-memory`).
2. **`kd ai` como área fina** sobre o `Source` (posicional, default consulta) — só depois.
3. **`session-log` via projeção do agente** (read-only, derivada) — nunca como novo domínio.
4. **Não** acoplar o knudge ao schema `Event` do ktu. O contrato é o **documento** (`{frontmatter,
   body}`), não o evento.
5. **YAGNI condicional:** só promover se **≥2** agentes usarem a fonte; com um só (ktu), o ganho
   não paga a superfície no knudge.

> **Em uma frase.** É viável e é elegante — mas o ganho é de **ecossistema**, não de ktu: o ktu já
> tem o motor. A jogada certa é dar ao knudge uma **fonte JSONL read-only** (o que o pedido chama
> de "caso JSONL") e deixar o **agente projetar** os seus logs — preservando "uma só verdade".

---

## 5. Actualização — visão aprovada

Aprovado pelo dono. A visão completa (o knudge como **eixo único de memória, agent-agnostic**) está
em [o knudge como eixo de memória](./knudge-eixo-memoria.md): o standard é o schema do knudge; as
vias de integração são **nativa** (escreve notas), **adaptada** (`Source` JSONL) e **projetada**
(log → notas derivadas); a consulta é **inteligente** (BM25 + âncoras + grafo + RRF), não
sequencial; e a **concorrência multi-agente** já está resolvida (lock por ficheiro, ids por
conteúdo, `merge=union`, proveniência `agent`/`activity`). Falta a **porta** e o **contrato**.
