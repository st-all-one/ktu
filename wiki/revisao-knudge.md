# Revisão profunda do `knudge` — o epicentro de memória

> Exploração da **instância actual** do knudge (v0.6.1, clonada em `crates/knudge`) com o foco
> pedido: **como funciona, como se integra, o que faz cada parte e o que é realmente benéfico ao
> ktu**. Complementa a [revisão comparativa inicial](./revisao-katu-vs-ktu.md). Data: 2026-10-17.

---

## 0. Papel e escala

O knudge tem **dois papéis** distintos no projecto, que não se devem confundir:

| Papel | O que é | Onde |
|---|---|---|
| **Memória de desenvolvimento do ktu** | a memória do *próprio repositório* ktu (notas, tarefas, decisões) — o agente/humano usa o binário `kd` | `.knudge/` + `kd` (já em uso) |
| **Motor de memória do agente ktu** | a memória que o *agente ktu* usa em runtime durante uma sessão | `crates/ktu-memory` (porta `Memory`) |

| | `knudge` 0.6.1 | `ktu` |
|---|---|---|
| Linguagem | Rust 1.99 (edição 2024) | Rust 1.99 |
| Crates | 3 (`knudge-core`, `knudge-cli`=`kd`, `knudge-mcp`) | 7 |
| Arquitectura | **domínio puro + portas + adaptadores** (D65) | **idêntica** (portas + fakes) |
| Binários | `kd` (CLI) · `knudge-mcp` (MCP stdio) | `ktu` |
| Facade de incorporação | `Knudge`/`KnudgeBuilder` (D214) | — |
| `unsafe` | `#![forbid(unsafe_code)]` | idem |

> **A convergência é total:** o knudge é o *mesmo animal* que o ktu (núcleo puro, portas, sem
> `unsafe`/`panic`, `stdout=dados`/`stderr=logs`, `BTreeMap`/ordem canónica, ids por conteúdo).
> O ktu já adoptou o **padrão** do knudge (D92 nos lints); a questão é **quanto** do **motor** usar.

---

## 1. O que é e como se monta

### 1.1 Camadas

```
knudge-mcp ──┐
             ├── knudge-core (domínio puro + portas + adaptadores std + fachada Knudge)
knudge-cli ──┘
```

- **`knudge-core`** — biblioteca pública (`publish = true`, D214): modelo, schema, TOON, store,
  retrieval, write, task, handoff, grafo, saúde, ciclo de vida, embeddings, config, git.
- **`knudge-cli`** (`kd`) — borda: `clap`, `tracing`, I/O de terminal.
- **`knudge-mcp`** — borda: gatilhos MCP sobre JSON-RPC 2.0 stdio.

### 1.2 Dois níveis de API (D214)

| Nível | Quando | Entrada |
|---|---|---|
| **Fachada** | a mesma montagem da CLI/MCP (adapters `std` + `Project` + config) | `Knudge::open()` / `Knudge::builder().knowledge_dir(".a/b").open()` |
| **Portas + domínio** | injectar `Clock`/`Fs`/`Git`/`Embedder` próprios, determinismo em teste | `Project`, `Store`, `Index`, `recall`, `write`, `ports::fakes` |

A fachada expõe `project()`, `config()`, `store()`, `events()`, `index()`, `corpus()`,
`graph()`, `thresholds()`, `write_context()`, `sweep_residues()` — construída **uma vez por
processo** e reutilizada (`Knudge` não é `Sync`/`Clone` por design).

### 1.3 Portas

`Clock` · `Rng` · `Env` · `Fs` · `Git` · `HookRunner` · `Logger` · `Embedder` — cada uma com fake
(`FixedClock`, `SeqRng`, `MemFs`, `FakeGit`, `NoopHookRunner`, `RecordingLogger`, `FakeEmbedder`).
**É a mesma lista de portas que o ktu tem** (`ktu-core::ports`), com duas a mais: `Git` e `Embedder`.

---

## 2. A maturação: 0.5.4 (REF) → 0.6.1 (actual)

A referência antiga (`REF/knudge`, v0.5.4) e a actual (v0.6.1) têm o **mesmo conjunto de módulos**
e a **mesma API pública** nos sítios que o ktu usa. O que mudou é **dentro** (algoritmos, robustez,
decisões). As decisões que importam para um integrador:

| D | O que muda | Efeito |
|---|---|---|
| **D214** | `knudge-core` publicável + **fachada `Knudge`** + **layout configurável** | integra-se como biblioteca; `knowledge_dir` deixa de ser fixo em `.knudge` |
| **D207** | **claims SPO** + **ontologia leve** (`same_as`/`broader`/`narrower`/`related`) + **proveniência** PROV-lite | contradição precisa; `SCHEMA_VERSION = 2` |
| **D208** | **TMS/defeasible** + **drift KL/JS** | dependentes retratados são derivados; nada apagado |
| **D192** | **PageRank + PPR** (autoridade no grafo) | canal `ppr` na fusão (default `0.0`) |
| **D193** | **Louvain** + comunidades (GraphRAG) | `kd map --communities` |
| **D190** | shelf-life por **curva de esquecimento** (FSRS-like) | cada `outcome` estende o prazo |
| **D189** | confiança **Beta-Bernoulli** (média + limite inferior 95 %) | nunca armazenada; alimenta o canal `stars` |
| **D206** | **stemming PT** conservador | recall melhor em português |
| **D204** | **MinHash/LSH** no dedup acima de 256 notas densas | ~97 % de ganho medido |
| **D216** | **prefixo de instrução por modelo** (`embeddings.prefix`) | `embeddinggemma-2` com prefixos assimétricos |
| **D215** | dreno de embeddings em **lotes limitados** + `--digest` | não manda a fila inteira numa requisição |
| **D191** | **slots mínimos de corpo por tipo** | validação soft (aviso; erro só em `strict`) |
| **D205** | flow metrics + **caminho crítico** (PERT/CPM) | `kd task flow` |
| **D209/D210/D212** | superfície v3; listas por repetição/vírgula; conjuntos fechados listam opções | contrato de CLI |

### 2.1 Migração validada (empírica)

O `ktu-memory` apontava para `REF/knudge` (0.5.4). Apontei-o para a instância nova e **compila e
testa** sem mais alterações:

```toml
# crates/ktu-memory/Cargo.toml — a única mudança
knudge-core = { path = "../../crates/knudge/crates/knudge-core" }   # era ../../REF/knudge/...
```

```
cargo check -p ktu-memory   → Finished (8,3 s)
cargo test  -p ktu-memory   → 7 passed; 0 failed
```

> **Conclusão:** a API que o ktu usa (`Index::build`, `Graph::from_notes_ref`, `recall`,
> `RecallQuery`, `propose`, `DedupDecision`, `DedupThresholds`, `Note`, `Draft::to_note`,
> `jsonl::json::{encode,decode}`, `toon`, `schema`) é **estável** entre 0.5.4 e 0.6.1. A migração
> é **uma linha** — e ganha-se toda a maturação (§2).

---

## 3. Como funciona (o essencial)

### 3.1 A nota é a verdade; o índice é derivado

```
notas/<tipo>/<id>.md   ← FONTE (Markdown + frontmatter TOON)
eventos/events*.jsonl  ← auditoria (append-only, rotaciona)
.idx/ cache/ .locks/   ← DERIVADO e reconstruível (nunca versionado)
```

- **Escrita atómica** (`tmp + rename`) + `fsync` em batch (D20/D22).
- **Nota antes do evento** (D21): um crash deixa nota sem evento — a reconstrução tolera.
- **Lock advisory** por alvo, stale 30 s, reclaim por rename, `LockGuard` RAII (D23–D25).
- **Rebuild** por double-buffer `.idx.new/` + rename atómico (D27).
- **Dedup on-read** por id (idempotente sob `merge=union`).

### 3.2 Determinismo

Sem `unsafe`, sem `panic`/`unwrap` em produção; toda iteração tem ordem estável (`BTreeMap`/
`IndexMap`, **nunca** `HashMap`); ids/hashes derivam de conteúdo (`normalize = NFC + trim +
colapso`); aritmética checada. **É o mesmo contrato do ktu.**

### 3.3 Contrato de bytes (TOON)

O frontmatter é **TOON** (não YAML/JSON), com `CANONICAL_KEYS` (31 chaves, ordem fixa),
`REQUIRED_KEYS` (5), opcionais vazios **omitidos** (nunca `null`), `SCHEMA_VERSION = 2`. Mudanças de
bytes exigem bump de schema/rebuild. **Tratar como ABI.**

---

## 4. O que faz cada parte (e o que é benéfico)

| Módulo | O que faz | Benefício ao ktu |
|---|---|---|
| `schema` | tipos, ids/hashes, **claims SPO**, **proveniência**, **slots** | contrato estável; contradição precisa |
| `toon` | parser/emissor do frontmatter | contrato de bytes determinístico |
| `jsonl` | JSONL + **codec JSON canónico** | já usado pelo ktu (notas) |
| `store` | notas/eventos/lock/rebuild/purge/sweep | durabilidade + auditoria que o ktu reimplementa |
| `retrieval` | **BM25 + âncoras + RRF** + stemming PT + PPR + filtros/views | recall rico; hoje o ktu usa um subconjunto |
| `write` | protocolo idempotente: dedup → create/merge/reject; update/supersede; batch | o ktu reimplementa parte disto |
| `graph` | arestas explícitas, integridade/ciclos, ontologia, TMS, **PageRank/PPR**, **Louvain** | inferência que o ktu não tem |
| `task` | `epic ⊃ {issue ⊃ task \| task}` como **view derivada**; flow/caminho crítico | o ktu não tem hierarquia de trabalho |
| `handoff` | **`rewind`**: contexto de início de sessão com **orçamento** + `context_id` | o ktu tem `prompt.rs`/`budget.rs`; `rewind` é mais rico |
| `health` | **`doctor`** (13 checks) + `audit` + validators + fecho por **evidência** | o ktu não tem saúde de memória |
| `lifecycle` | confiança Beta, shelf-life FSRS, decay/drift, retenção, clusters, demolição | curadoria automática do corpus |
| `embeddings` | porta `Embedder` plugável, cache por `body_hash`, fila lazy, ranking | canal semântico opcional |
| `config` | dois níveis (global/projeto), subset TOML, segredos só no global | o ktu não tem config em ficheiro |
| `git` | `Project`, `onboard`, `info/exclude`, `.gitattributes`, `sync` | versionar a memória sem poluir o repo |
| `knowledge` | (apoio) | — |

### 4.1 Os três subsistemas mais valiosos para o ktu

1. **`store` + `write`** — a durabilidade, o lock, a auditoria e o protocolo de escrita
   (idempotência, dedup, supersede) são exactamente o que o `ktu-memory` hoje **reimplementa** de
   forma mais simples (JSONL append + dedup manual). Adoptar dá robustez testada.
2. **`handoff::rewind`** — o contexto de início de sessão com orçamento de tokens e `context_id`
   reproduzível é o primo directo do `prompt.rs`/`budget.rs` do ktu, mas **com vizinhança**
   (working set, escopo, ficheiros alterados).
3. **`health::doctor`** — 13 verificações + `--fix` idempotente dão ao ktu um "estado de saúde da
   memória" que hoje não existe.

---

## 5. Como se integra

### 5.1 O padrão (que o ktu já segue)

O ktu **não** deve acoplar o kernel ao knudge (firewall). O padrão correcto — e o actual — é:

```
ktu-core::Memory  (porta, tipos do ktu)      ← o kernel só conhece isto
        ▲
        │ implementa
ktu-memory::JsonlMemory                       ← adaptador (borda)
        │ usa
knudge-core (domínio)                         ← motor
```

Isto espelha **exactamente** o D65 do knudge (domínio puro + portas + adaptadores) e o **D214**
(fachada de incorporação). A porta `Memory` do ktu tem os seus próprios tipos
(`MemoryKind`, `Hint`, `MemoryDraft`, `MemoryWrite`) — nunca tipos do knudge.

### 5.2 O gap actual

O `ktu-memory` usa o knudge **apenas como motor de retrieval + dedup** sobre um **JSONL próprio**
(`.agents/memory/notes.jsonl`), e **ignora**:
`Store`/`notas/` (layout), `eventos/` (auditoria), `write::write` (protocolo completo),
`task`, `handoff::rewind`, `health::doctor`, `lifecycle`, `embeddings`, `graph`
(PageRank/PPR/Louvain/TMS/ontologia), `config`, `git`.

Ou seja: usa ~15 % do motor. O resto é reimplementado à mão (ou simplesmente ausente).

### 5.3 Opções de integração

| Opção | O que é | Custo | Alinhamento |
|---|---|---|---|
| **A — Thin (actual)** | porta `Memory` + `Index`/`recall`/`propose` sobre JSONL próprio | já feito | mínimo |
| **B — Store + write** | adaptador passa a usar `Store`/`notas/` + `write` (dedup/supersede/eventos) | médio | alto (durabilidade) |
| **C — Fachada `Knudge`** | o adaptador abre a fachada e usa `store`/`corpus`/`write_context` | médio | alto |
| **D — Full** | expor `task`/`rewind`/`doctor`/`lifecycle`/`embeddings` ao agente | alto | máximo, mas alarga a superfície |

**Recomendação:** **B** agora (durabilidade + protocolo de escrita), **C** como mecanismo (usar a
fachada em vez de montar `Store`/`Index` à mão), e **D** só por decisão explícita, à medida que o
agente precisar (ex.: `rewind` no início de sessão).

---

## 6. O que é benéfico ao ktu como epicentro de memória

### 6.1 Adoptar (alto valor, alinhado ao kernel)

1. **`Store` + `notas/` + `eventos/`** — substituir o JSONL próprio por `notas/<tipo>/<id>.md`
   (a verdade) + `eventos/` (auditoria). Ganha-se atomicidade, lock, rebuild, dedup-on-read e
   auditoria **de graça**. O `layout` configurável (D214) permite `.agents/memory/` ou `.knudge/`.
2. **`write::write`** — adoptar o protocolo (idempotência por id → dedup → create/merge/reject;
   `update`/`supersede`). Elimina a reimplementação em `ktu-memory/src/store.rs`.
3. **`retrieval` completo** — âncoras (canal determinístico), filtros, `views`, stemming PT, e
   opcionalmente o canal `ppr`/`semantic`. O ktu já usa `recall`; falta ligar `working_paths` e
   filtros.
4. **`handoff::rewind`** — o contexto de início de sessão com orçamento + `context_id`. Casa com o
   `prompt.rs`/`budget.rs` do ktu e substitui a montagem manual de contexto.
5. **`health::doctor`** — 13 checks + `--fix` idempotente; dá ao ktu um `ktu doctor --memory`.
6. **`git::sync`** — versionar a memória (notas/eventos) sem poluir o repo (`info/exclude`).

### 6.2 Avaliar (valor real, mas alarga a superfície)

7. **`task`** — a hierarquia `epic ⊃ {issue ⊃ task}` como view derivada poderia substituir/complementar
   o **modo plano** do ktu. Custo médio; valor alto se o ktu ganhar trabalho estruturado.
8. **`graph` avançado** (ontologia, TMS, PageRank/PPR, Louvain) — inferência que o ktu não tem;
   só com um consumidor que a exija (o `map`/`communities`).
9. **`lifecycle`** (confiança, shelf-life, decay, demolição) — curadoria automática; avançado.
10. **`embeddings`** — canal semântico opcional; o ktu já tem a porta `Embedder`? (não) — adoptar a
    porta, não o serviço.
11. **`config` dois níveis** — se o ktu adoptar config em ficheiro (backlog 76), usar o mesmo
    modelo (global/projeto, segredos só no global).

### 6.3 Não adoptar (YAGNI / fora de escopo)

- O **binário `kd`** e o **`knudge-mcp`** como dependências de runtime do ktu (o ktu tem a sua
  porta; o MCP é YAGNI §0).
- A **CLI/envelope** do knudge (borda; o ktu tem a sua).
- O **serviço de embeddings** (`drain service`, `llama.cpp`) — se o ktu usar embeddings, usa a
  **porta**, não o worker.

### 6.4 Distinção crítica (os dois papéis)

- **Memória de desenvolvimento do ktu** (`.knudge/` + `kd`): **já em uso** — é o `kd` CLI que o
  agente usa para `prime`/`ask`/`write`/`task`/`rewind` neste repositório. **Não mudar.**
- **Motor de memória do agente ktu** (`ktu-memory`): hoje *thin*; a recomendação é **engrossar**
  para B/C. **São coisas diferentes** e não se devem fundir (a memória do *projecto ktu* ≠ a
  memória do *agente ktu* em runtime).

---

## 7. Estado actual do `ktu-memory` (verificado no código)

| Peça | O que faz hoje | Gap |
|---|---|---|
| `store.rs` | JSONL append-only + dedup-on-read + `temp→fsync→rename` | não usa `Store`/`notas/`/`eventos/` |
| `convert.rs` | fusão ktu↔knudge (`Draft`, `Note`, `Frontmatter`) | — |
| `record.rs` | codec JSON canónico (do knudge) | — |
| `report.rs` | `export --md` + `health` (contagens/âncoras quebradas) | não usa `health::doctor` |
| `lib.rs` | porta `Memory` (`recall`/`write`) | — |

O adaptador usa `Index::build` + `Graph::from_notes_ref` + `recall` + `propose` — o **subconjunto**
de retrieval/dedup do knudge. É **correcto** (respeita a porta), mas **subutiliza** o motor.

---

## 8. Recomendação

1. **Manter a instância actual em `crates/knudge`** (v0.6.1) e a migração de **uma linha** (já
   validada: `cargo check` + `cargo test` verdes).
2. **Engrossar o `ktu-memory`** para a opção **B/C**: `Store`/`notas/`/`eventos/` + `write` +
   `retrieval` com âncoras/filtros, mantendo a **porta `Memory`** com tipos do ktu.
3. **Adicionar `rewind` e `doctor`** como capacidades do agente (contexto de sessão + saúde da
   memória) — são os dois subsistemas com melhor relação valor/custo.
4. **Não** trazer o `kd`/MCP/config/lifecycle/graph avançado sem um consumidor concreto (§0 do
   `plan.md`).
5. **Registar a decisão** (ADR/knudge): a instância canónica é `crates/knudge` 0.6.1; a
   referência antiga (`REF/knudge` 0.5.4) deixa de ser usada pelo `ktu-memory`.

> **Nota.** O knudge é o único referencial que é **o mesmo arquétipo exacto** do ktu — não é uma
> ideia a colher, é **um motor a montar**. A porta `Memory` já é a fronteira certa; falta apenas
> deixar o motor respirar por dentro dela.
