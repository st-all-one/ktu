---
question_id: Q4
same_session: true
compacted: true
question_issue: "Com base em /exploration/knudge/[agent]/knudge.md, crie um aquivo EPIC.md ao lado dele, descrevendo como reimplementar em typescript, vantagens e desvantagens, contneod no mínomo um plano mestre e diversas issues, encadeads sequencialmente" 
expected_answer_ideia: "Deve ser capaz de construir a cadeia de tarefas, vantagens, desvantagens globais e a cadeia de tarefas deles"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: goose
---

# GOOSE_EPIC — Reimplementação do knudge em TypeScript

> **Base:** `GOOSE_RESEARCH.md` (não versionado) (síntese técnica de `knudge v0.5.2`, commit `2ef623c`).
> **Objetivo:** especificar a reimplementação do knudge em **TypeScript**, com **plano mestre**, **issues encadeadas sequencialmente** e análise honesta de **vantagens e desvantagens**.
> **Autor:** goose 🪿

---

## 0. Sumário executivo

O knudge é um sistema de **memória durável por projeto** para agentes de LLM: CLI `kd` + servidor MCP, offline, sem banco/servidor, com **notas Markdown como fonte da verdade** e **índice estritamente derivado**. O que torna a reimplementação em TypeScript *interessante* — e *arriscada* — é que o sistema se apoia em **três pilares não-funcionais** que o Rust entrega barato e o JS entrega caro:

1. **Determinismo byte-a-byte** (§5.4 do research) — goldens, proptest, handoff reprodutível.
2. **Paralelismo de CPU** (BM25, Louvain, PageRank, LSH) — Rust usa `std::thread::scope`; JS é single-thread + `worker_threads`.
3. **Piso de inicialização ~4 ms** por invocação de CLI — o modelo "sem daemon" depende disso.

**Conclusão antecipada:** a reimplementação é **viável e desejável** para *portabilidade, ecossistema MCP e velocidade de iteração*, mas exige **duas disciplinas novas** que o Rust não exigia: (a) uma **camada de matemática determinística** própria (porque `Math.log/exp` não são estáveis *cross-engine*), e (b) uma **camada de ordenação canônica explícita** (porque `Map`/`Object` do JS têm semânticas de ordem que o Rust proíbe). Nenhuma das duas é bloqueante — mas ambas são **requisitos de projeto**, não detalhes de implementação.

---

## 1. Escopo

### 1.1 Em escopo
- Os 3 artefatos: biblioteca de núcleo (**`@knudge/core`**), CLI (**`kd`**), servidor MCP (**`knudge-mcp`**).
- Contrato de bytes TOON, identidade endereçada por conteúdo, store crash-safe.
- Retrieval híbrido, grafo, ciclo de vida, escrita/dedup, tarefas, embeddings opcionais.
- Bancada de avaliação (Recall@k/MRR/nDCG@k) e verificação de determinismo.

### 1.2 Fora de escopo (não-objetivos)
- **Não** é um porte 1:1 linha-a-linha. É uma reimplementação **guiada por contrato**.
- **Não** replica `unsafe`/Miri (não há UB em JS; a garantia é outra — ver §5.4).
- **Não** busca paridade de *performance* absoluta; busca paridade **funcional** e **de contrato**.
- **Não** reimplementa o worker de embeddings `llama.cpp` no mesmo processo — segue sendo sidecar HTTP.

### 1.3 Estratégia de compatibilidade de contrato (decisão fundacional)

| Opção | Descrição | Prós | Contras |
|---|---|---|---|
| **A. Paridade byte-a-byte** | `kd-ts` produz bytes idênticos ao `kd` Rust (goldens cruzados) | Migração sem retrabalho; validação cruzada forte | Exige matemática determinística custom; trava decisões antigas (inclusive os achados de UX ruins do `usabilidade_humana.md`) |
| **B. Re-baseline (RECOMENDADA)** | Mesmo **formato** TOON e mesmos **ids** (interop de corpus!), mas **superfície de saída** re-baselinada | Corrige de saída os achados H1–H8; libera a UX | Goldens novos; precisa de um *conformance corpus* |

> **Recomendação:** **B**. O **formato no disco** (`notas/`, TOON, `id`, `body_hash`) *precisa* ser bit-compatível para que corpus Rust e TS coexistam no mesmo repositório — isso é o que preserva a "fonte da verdade é sua". A **saída de CLI** pode ser re-baselinada para corrigir os defeitos de usabilidade humana catalogados em `GOOSE_SUGGESTION.md` (ex.: `init` que ensina comando válido, `rejected` distinguível, `--path` no `ask`).

---

## 2. Vantagens da reimplementação em TypeScript

| # | Vantagem | Por que importa aqui |
|---|---|---|
| V1 | **Ecossistema MCP nativo** | O domínio *é* memória para agentes; o SDK MCP/referências estão em TS. `knudge-mcp` fica "cidadão de primeira classe" em vez de binário à parte. |
| V2 | **Distribuição por `npm i -g`** | Substitui o `install.sh` (verificação SHA, PATH, completions). Um `npx knudge` funciona em qualquer máquina com Node. |
| V3 | **Portabilidade sem cross-compile** | Mesmo artefato em Linux/macOS/Windows; e, via `bun`/`deno compile`, binário único. |
| V4 | **Iteração rápida** | Sem ciclo de compilação de minutos do Rust `lto=fat`; feedback em segundos. |
| V5 | **JSON/JSONL/HTTP nativos** | O envelope `--json`, `events.jsonl`, `params` JSON e provedores de embeddings HTTP mapeiam direto. |
| V6 | **Tipos de união fechados** | Os enums fechados do knudge (`NoteType`, `Status`, `EdgeKind`) viram *discriminated unions* — checagem exaustiva em compile-time, como o Rust. |
| V7 | **Base de contribuidores maior** | Mais gente escreve TS do que Rust; o projeto é "ferramenta de dev", onde TS é a língua franca. |
| V8 | **Prototipagem de algoritmos** | Ajustar BM25/RRF/Louvain é mais rápido de experimentar; a bancada `bench/` vira script TS. |

---

## 3. Desvantagens e riscos (com mitigação)

| # | Desvantagem | Gravidade | Mitigação proposta |
|---|---|---|---|
| D1 | **`Math.log/exp/pow` não são determinísticos *cross-engine*** (IEEE-754 não especifica ULP de transcendentais) | 🔴 **Alta** | Não usar `Math.*` no núcleo. Implementar `libm` determinístico em TS (port de algoritmos de precisão documentada: `log`, `exp`, `ln_1p`, `log2`), ou *pin* de runtime + goldens por versão. Afeta BM25 (`ln_1p` no IDF), retenção (`exp`/`log`) e KL (`log2`). |
| D2 | **Sem inteiros de 64 bits** | 🟠 Média | `BigInt` para hashing/base36/timestamps e máscara de 64 bits (`fnv1a`, `splitmix64`). Nunca usar `Number` para hash. |
| D3 | **Piso de startup maior** (~40–80 ms vs ~4 ms) | 🔴 **Alta** | Compilar com `bun build --compile`/`deno compile` (startup ~5–15 ms); ou mover trabalho para o servidor MCP persistente. O design "sem daemon" sofre mais no Node puro. |
| D4 | **CPU-bound single-thread** (BM25, Louvain, LSH em corpora ~1k+) | 🟠 Média | `worker_threads` para leitura+parse paralela (paridade com `thread::scope`, remontando em ordem canônica); manter a mitigação LSH do dedup; opcional: `napi-rs` para *hot loops*. |
| D5 | **`Map`/`Set` preservam ordem de inserção; `Object` tem ordem peculiar** | 🟠 Média | O Rust **proíbe** `HashMap` para determinismo. Em TS, criar um `SortedMap`/`CanonicalIndex` que ordena por chave canônica; nunca depender de ordem de inserção em agregações. |
| D6 | **Falta de abstrações de custo zero** (I/O atômico, `fsync`, lock advisory) | 🟠 Média | `fs.open('wx')` para lock create-exclusive (o repo já usa); `fs.rename` atômico; `fs.fsync` via fd; evitar `proper-lockfile` (magia) em favor de lock explícito próprio. |
| D7 | **Locks/paralelismo entre processos** | 🟡 Baixa | O Rust usa lock advisory por arquivo; em Node, replicar com arquivo-lock `O_EXCL` + stale por mtime (30 s), com a mesma ordem container→nota (evita ABBA). |
| D8 | **Empacotar dependências nativas** (embeddings/llama.cpp) | 🟡 Baixa | Manter embeddings como **serviço HTTP externo opcional** (já é o desenho). Nada de binário nativo no pacote. |
| D9 | **Superfície de tipos em runtime** (TS não valida em runtime) | 🟢 Baixa | `zod` na fronteira (TOON, JSONL, config, entrada de CLI); tipos só no núcleo puro. |
| D10 | **Divergência de comportamento Unicode** | 🟡 Baixa | `String.normalize('NFC'/'NFD')` cobre o que o Rust usa (`unicode-normalization`); travar com *conformance corpus* de casos (é o que `DIVERGENCES.md` faz). |
| D11 | **`async` contamina o núcleo puro** | 🟠 Média | Manter o núcleo **síncrono e puro** (como o Rust): I/O só pelas portas; `async` fica na borda (adapters/CLI/MCP). |
| D12 | **Risco de "segunda implementação" divergir** | 🔴 **Alta** | **Conformance corpus** (TS-E16) comparando `id`, `body_hash`, TOON emitido e top-k do `ask` contra goldens do Rust. Sem esse gate, as duas versões divergem em silêncio. |

### 3.1 Veredito

TypeScript é **adequado** para a camada de **orquestração, CLI, MCP, I/O e algoritmos**, e **inadequado sem mitigação** para (i) matemática transcendental bit-estável, (ii) throughput CPU-bound extremo e (iii) startup de CLI. Nenhum é bloqueante, mas **D1 e D12 são os riscos que definem o projeto** — se o objetivo for *paridade byte-a-byte* com o Rust, o custo sobe muito; se for **interop de corpus + re-baseline de saída**, o custo é razoável.

---

## 4. Arquitetura-alvo (mapeamento Rust → TypeScript)

| Rust | TypeScript | Nota |
|---|---|---|
| `knudge-core` (puro + ports + adapters) | `packages/core` (puro) + `packages/core/adapters` | núcleo **síncrono**; `async` só na borda |
| `knudge-cli` (`clap`) | `packages/cli` (`commander`/`yargs`) | binário `kd` |
| `knudge-mcp` (JSON-RPC stdio) | `packages/mcp` (`@modelcontextprotocol/sdk`) | servidor MCP |
| `#![forbid(unsafe_code)]` | — (nenhum UB) | garantia troca de natureza (§5.4) |
| `BTreeMap`/`BTreeSet` | `SortedMap<string,T>` próprio | ordem canônica explícita (D5) |
| `f64` + `total_cmp` | `number` + `compareTotal()` | IEEE-754 duplo nos dois (ok); comparador próprio para NaN/`-0` |
| `sha2`, `unicode-normalization` | `@noble/hashes`, `String.normalize` | sync, auditado |
| `std::thread::scope` | `worker_threads` + reassembly canônico | paridade de *determinismo*, não de thread |
| `clap`, `tracing`, `serde_json` | `commander`, `pino`, `JSON`/`zod` | envelope `--json` preservado |
| `proptest`, `miri`, `fuzz` | `fast-check`, (n/a), `jazzer.js`/`jsfuzz` | ver §5.4 |

**Padrões preservados (não negociáveis):**
- **Portas e adaptadores (D65):** `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`, `Embedder` como interfaces + implementações `std` + `fakes` (`MemFs`, `FaultyFs`, `FixedClock`…).
- **Fonte da verdade:** `notas/<tipo>/<id>.md`; `.idx/` derivado e descartável.
- **Escrita crash-safe:** `tmp + rename` (D20), `fsync` batch (D22), **nota antes do evento** (D21), lock `O_EXCL` (D23–D25), rebuild double-buffer `.idx.new/` + rename (D27).

---

## 5. Plano mestre

### 5.1 Fases

| Fase | Nome | Issues | Resultado |
|---|---|---|---|
| **F0** | Fundação & determinismo | TS-E01, TS-E02 | Monorepo + primitivas canônicas |
| **F1** | Contrato de bytes | TS-E03, TS-E04 | TOON + schema + ids bit-compatíveis |
| **F2** | Persistência | TS-E05, TS-E06 | Portas/fakes + store crash-safe |
| **F3** | Inteligência | TS-E07, TS-E08, TS-E09, TS-E10, TS-E11 | Retrieval, grafo, escrita, ciclo de vida, tarefas |
| **F4** | Superfície | TS-E12, TS-E13 | CLI `kd` + servidor MCP |
| **F5** | Validação & entrega | TS-E14, TS-E15, TS-E16 | Bancada, distribuição, conformidade |

**Encadeamento:** as issues são **sequenciais** — cada uma depende **exclusivamente** da anterior (`TS-E0N depends_on TS-E0(N-1)`). Isso permite paralelizar por *fork* apenas após estabilizar cada contrato, e garante que nenhum gate de aceite dependa de trabalho futuro.

### 5.2 Stack

| Área | Escolha | Justificativa |
|---|---|---|
| Runtime | Node 22 LTS (dev) + `bun`/`deno compile` (release) | startup e binário único |
| Monorepo | pnpm workspaces + `tsc` project references | isolamento dos 3 pacotes |
| Tipos | TypeScript `strict` + `noUncheckedIndexedAccess` + `exactOptionalPropertyTypes` | espelha as invariantes do Rust |
| Validação runtime | `zod` | fronteira (TOON/JSONL/config/CLI) |
| Hash | `@noble/hashes` (sha256) + BigInt (fnv1a/splitmix64) | sync, auditado, portátil |
| Testes | `vitest` + `fast-check` | unit + property |
| Lint/format | `biome` | rápido, zero-config |
| CLI | `commander` | verbos/subcomandos (Docker-like) |
| MCP | `@modelcontextprotocol/sdk` | padrão do ecossistema |
| FS | `node:fs/promises` + `node:fs` (fd para `fsync`) | atômico |

### 5.3 Gates de aceite (todo o projeto)

1. **Determinismo:** rodar a suíte 2× em runtimes diferentes (Node e Bun) → saída byte-idêntica (ou falha explícita, dado §5.4).
2. **Interop de corpus:** `id`/`body_hash`/TOON emitido batem com goldens do Rust (TS-E16).
3. **Sem `unwrap` equivalente:** `strict` + `noUncheckedIndexedAccess`; sem `any` no núcleo; erros como valor (`Result<T,E>`) na fronteira do domínio.
4. **Piso de startup:** `kd --help` < 20 ms (Bun compilado).
5. **Regressão:** todo bug encontrado vira teste (paridade com `AGENTS.md` §6 do repo).

### 5.4 A questão do determinismo em TS (núcleo do risco D1)

O Rust garante determinismo **dentro da mesma build**; o JS adiciona um eixo a mais: **o motor** (V8 vs JavaScriptCore) e a **versão**. Pontos de atenção concretos:

- **Somas/ordenação:** determinísticas se a ordem canônica for explícita → **resolvido por disciplina** (D5).
- **IEEE-754 básico** (`+ - * /`, `sqrt`): determinístico (a spec exige corretamente arredondado). **ok**.
- **Transcendentais** (`Math.log`, `Math.exp`, `Math.pow`, `Math.log2`): **NÃO** especificados → podem diferir em ULP entre motores. **Solução:** `packages/core/src/math/libm.ts` com algoritmos determinísticos (ex.: `log`/`exp` com séries e redução de argumento documentadas), cobertos por vetores de teste.
- **`total_cmp` (Rust, ordem total IEEE):** reimplementar `compareTotal` tratando `-0 < +0`, `NaN` ordenado.
- **String→bytes:** sempre `TextEncoder`/UTF-8 explícito; nunca depender de `length` UTF-16 para contagem de bytes.

> Se a decisão for **não** implementar `libm` próprio, então a garantia vira "**determinístico dado um runtime pinado**", e os goldens passam a ser por-versão. Isso deve ser uma **decisão registrada** (equivalente a um `Dxx`).

---

## 6. Issues (encadeadas sequencialmente)

> Formato: **ID · Título** · `depends_on` · Objetivo · Entregáveis · Critérios de aceite · Risco.
> Cada issue depende **apenas** da anterior. Entregáveis são artefatos verificáveis.

---

### TS-E01 · Monorepo, toolchain e contratos de tipo
**depends_on:** — (fundação)
**Objetivo:** montar o workspace que espelha a separação do Rust (núcleo puro / binário CLI / servidor MCP).
**Entregáveis:**
- `packages/core`, `packages/cli`, `packages/mcp` (pnpm workspaces + project references).
- `tsconfig` estrito compartilhado; `biome`; `vitest`; CI básica.
- `Result<T,E>` + `ErrorKind` (mapa código→exit idêntico ao §9 do research).
- Regra de lint: **proíbe `Math.log/exp/pow/log2`** no `core` (aponta para `math/libm`).
**Aceite:** `pnpm build && pnpm test` verde; `Result`/`ExitCode` com testes; lint falha se `Math.*` transcendental aparecer no core.
**Risco:** baixo.

---

### TS-E02 · Primitivas determinísticas (ordem canônica + math estável + hashing)
**depends_on:** TS-E01
**Objetivo:** entregar as fundações de determinismo (resposta a **D1**, **D2**, **D5**).
**Entregáveis:**
- `SortedMap`/`CanonicalIndex` (ordem por chave canônica; `BTreeMap`-like).
- `compareTotal()` (ordem total IEEE: `-0`, `NaN`).
- `math/libm.ts`: `log`, `exp`, `ln1p`, `log2`, `pow` determinísticos + vetores de teste.
- `hash.ts`: `sha256` (`@noble/hashes`), `fnv1a64`/`splitmix64` em BigInt, `base36(8)`.
- `unicode.ts`: `normalize` (NFC + trim + colapso), `foldDiacritics` (NFD + descarte de marcas).
**Aceite:** proptest (`fast-check`) de idempotência de `normalize`; `compareTotal` consistente com a ordem total; `libm` reproduz vetores de referência dentro de tolerância declarada **e** é estável entre Node e Bun; `base36`/hash batem com vetores de referência.
**Risco:** 🔴 **alto** (é a issue que decide se a promessa de determinismo sobrevive).

---

### TS-E03 · Codec TOON (contrato de bytes)
**depends_on:** TS-E02
**Objetivo:** parser/emissor TOON com as 8 seções da spec (`wiki/specs/TOON.md`).
**Entregáveis:**
- `toon/parse.ts`, `toon/emit.ts`, `toon/splitFrontmatter.ts`.
- Gramática (indent 2, flow list/map, escapes, comentário `#`, aninhamento ≤32, chave duplicada = erro).
- Escalares (`Bool`/`Int`/`Float`/`Str`), regra "`1.0 → 1`" (D09), zero bytes p/ vazio (D11), LF final (D12).
**Aceite:** `emit(parse(x)) === x` no corpus canônico; goldens de escapes/zero-bytes/comentários; fuzz de parser não-trava (sem `eval`).
**Risco:** médio (parsing é onde bugs de bytes se escondem).

---

### TS-E04 · Schema, IDs endereçados por conteúdo e validação
**depends_on:** TS-E03
**Objetivo:** o modelo de dados e a identidade bit-compatíveis (§4 do research).
**Entregáveis:**
- 31 chaves canônicas na ordem exata; `REQUIRED_KEYS` (5); 12 arestas; `claims` (SPO) e `provenance` (PROV-lite).
- Enums como *discriminated unions*: `NoteType`, `Scope`, `Classification`, `Status`, `EdgeKind`.
- `noteId(type, statement)`, `bodyHash(statement, body)`, `shortHash`, `SCHEMA_VERSION=2`.
- `zod` schema + `validate()` (chaves conhecidas, limites: `statement ≤ 120` escalares, forma de claims/prov).
- Data contract soft por tipo (slots mínimos, D191).
**Aceite:** `id`/`body_hash` batem com vetores do Rust (base do TS-E16); round-trip de nota; `validate` rejeita chave desconhecida.
**Risco:** médio (qualquer desvio de normalização quebra a interop).

---

### TS-E05 · Portas, adaptadores e fakes
**depends_on:** TS-E04
**Objetivo:** isolar o domínio do mundo externo (D65), mantendo o núcleo **síncrono**.
**Entregáveis:**
- Interfaces: `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`, `Embedder`.
- Adapters `std`: `SystemClock`, `ThreadRng`, `StdFs`, `StdGit`, `ProcessHookRunner`, `PinoLogger`, `HttpEmbedder`.
- Fakes: `FixedClock`, `SeqRng`, `FakeEnv`, `MemFs`, `FaultyFs`, `FakeGit`, `NoopHookRunner`, `RecordingLogger`, `FakeEmbedder`.
**Aceite:** testes do core rodam 100 % com fakes (nunca tocam o SO); `FaultyFs` injeta falha de escrita.
**Risco:** baixo.

---

### TS-E06 · Store crash-safe (notas, eventos, lock, rebuild)
**depends_on:** TS-E05
**Objetivo:** persistência com as garantias de atomicidade (§7 do research).
**Entregáveis:**
- Escrita `tmp + rename` (D20); `fsync` em batch (D22); ordem **nota antes do evento** (D21).
- Lock `O_EXCL` por arquivo-alvo, ordem container→nota, stale 30 s, reclaim por rename (D23–D25).
- `events.jsonl` append-only + rotação `events-NNNN.jsonl`; `EventLog`; checkpoint `.idx/events.checkpoint`.
- Rebuild double-buffer `.idx.new/` + rename atômico (D27); varredura de resíduos `*.tmp`/`*.lock` (R10).
- `Corpus.loadNotes` (leitura canônica, com caminho paralelo preparado para TS-E07).
**Aceite:** crash-injection (`FaultyFs`) prova atomicidade e rebuild; leitor nunca vê índice parcial; lock não deadloca em ABBA.
**Risco:** médio.

---

### TS-E07 · Retrieval: tokenização, BM25, RRF e views
**depends_on:** TS-E06
**Objetivo:** o motor de busca híbrido (§5.1–§5.3 do research).
**Entregáveis:**
- Tokenização: ASCII `\w` → fold de diacríticos (D172) → stopwords PT+EN → **stemming PT** (D206); índice guarda radical, snippet guarda termo cru.
- **BM25** com IDF por campo, pesos de campo `3/2/1`, pesos de tipo, boost de confirmação, **corte de alta frequência** (D173, `ratio=0.9`, `N≥64`).
- `Postings` (índice invertido) + `Index` com `INDEX_FORMAT` versionado.
- **RRF** (`k=60`, pesos `lex=1, anchor=2, semantic=30, ppr=0`) + desempate `(score desc, id asc)`.
- Filtros determinísticos, views `ready`/`blocked`, `snippet`, `why`, canal de âncoras (glob DP), consulta temporal (`--as-of`).
- `worker_threads` para leitura+parse paralela, **remontando em ordem canônica**.
**Aceite:** Recall@k/MRR/nDCG@k ≥ baseline do Rust (TS-E14 mede); determinismo do top-k entre execuções; degradação graciosa (canal ausente → `warnings`).
**Risco:** 🟠 médio-alto (aqui o custo CPU do JS aparece).

---

### TS-E08 · Grafo, autoridade e comunidades
**depends_on:** TS-E07
**Objetivo:** projeções do grafo (§5.7 do research).
**Entregáveis:**
- Arestas explícitas, `integrity` (referências quebradas, órfãos, bidirecionalidade `replaces↔superseded_by`), ciclos por SCC.
- **PageRank/PPR** (damping 0,85, ≤32 iters, tol L1 1e-8, ordem canônica).
- **Louvain determinístico** (≤8 níveis, ≤64 passos, empate mantém comunidade).
- **Ontologia leve** (equivalência, clausura broader/narrower, ciclo hierárquico, `claim_conflicts`).
- **TMS** (retracted, defeated dependents, defeated_by_replacement) — derrotar sem remover.
**Aceite:** PageRank invariante a relabeling (fast-check); Louvain é função pura do grafo; integridade bate com goldens.
**Risco:** médio.

---

### TS-E09 · Escrita, dedup (sieve + MinHash/LSH), merge e update
**depends_on:** TS-E08
**Objetivo:** o protocolo de escrita (§6 do research).
**Entregáveis:**
- Validação de forma → **fase 1**: recall lexical → decisão calibrada `<0.75` cria / `0.75–0.92` merge / `≥0.92` rejeita.
- Dedup **escalável**: peneira exata (paridade byte) para corpus pequeno/esparso; **MinHash/LSH** (64 perm, FNV-1a+splitmix64 em BigInt, 16×4) acima de `MIN_LSH_CORPUS=256` e vocabulário denso — **recall ≥** garantido pelo Dice exato.
- **Merge** (união de tags/anchors/outcomes, `revision++`, claims/prov sem duplicar), **update/patch** (`--params`), **supersede** caminhável, `outcome`, lote JSONL (teto 100).
- `--dry-run` expondo `missing_slots`.
**Aceite:** idempotência sob retry; LSH ⊇ peneira exata (proptest); thresholds calibrados batem com o Rust.
**Risco:** 🟠 médio-alto (LSH em BigInt é o ponto mais "não-óbvio").

---

### TS-E10 · Ciclo de vida, confiança e drift
**depends_on:** TS-E09
**Objetivo:** retenção e consolidação barata (§5.4–§5.6 do research).
**Entregáveis:**
- **Beta-Bernoulli + limite inferior de Wilson** (`z=1.96`) → confiança derivada; `confidence_score` (base + evidência + feedback 0,2 + tarefa + recência 0,05; `drift_factor`).
- **Retenção FSRS-like** (`R(t)=limiar^(t/ttl)`, `growth=50%`, reset de origem), renovação por uso (`usage.jsonl`).
- **Drift de âncoras** (`.idx/drift.jsonl`) e **drift de termos KL/JS** (`log2` do `libm` de TS-E02; `DRIFT_THRESHOLD=0.5`, `MIN_DRIFT_NOTES=4`).
- Clusters estruturais + semânticos (complete-link), comunidades (reusa TS-E08), `plan.demotion_candidates` (5 motivos), supersessão/retirada.
**Aceite:** monotonicidade da confiança (fast-check); `prune` **só propõe** (nunca grava); JS/KL dentro de tolerância.
**Risco:** médio (depende fortemente do `libm` de TS-E02).

---

### TS-E11 · Tarefas, hierarquia, fluxo e handoff
**depends_on:** TS-E10
**Objetivo:** o subsistema de trabalho (§5.9 do research).
**Entregáveis:**
- Hierarquia `epic ⊃ {issue ⊃ task | task}` como view derivada; progresso/impacto/papel.
- **Flow**: cycle/lead/throughput + **caminho crítico PERT/CPM** (`L(id)=dur+max deps`), memoizado, ciclos → vazio, desempate determinístico.
- **Rewind/handoff**: manifest/escopo/working set, orçamento de tokens, `context_id`; corpos **sob demanda** (corrige o gargalo §8 do research) e `impact` pré-computado fora do `sort_by`.
**Aceite:** caminho crítico determinístico; `rewind` lê corpos apenas dos itens selecionados; orçamento respeitado.
**Risco:** médio.

---

### TS-E12 · Superfície CLI `kd`
**depends_on:** TS-E11
**Objetivo:** os ~10 verbos e seus contratos de máquina (§9 do research), **com as correções de UX** do `GOOSE_SUGGESTION.md`.
**Entregáveis:**
- Verbos: `init`, `prime`, `rewind`, `ask`, `write`, `task`, `map`, `drain`, `doctor`, `maintenance`, `config`, `forget`, `sync`, `self`.
- Contrato: `stdout=dados`, `stderr=logs`, `--json` envelope, pipe `id|statement|score|why`, exit codes idênticos, EPIPE→0.
- **Correções de UX (re-baseline):** `init` ensina comando válido (`--summary`); `rejected`/`merged` distinguíveis; `ask --id --path`; `doctor` renderiza `fixed:`; sem `(Dnn)` nas mensagens.
**Aceite:** matriz de aceite (pipe/`--json`/erro/exit) por verbo; comandos de `próximos:` rodam de fato; `--help` < 20 ms.
**Risco:** médio (re-baseline exige golden novo).

---

### TS-E13 · Servidor MCP `knudge-mcp`
**depends_on:** TS-E12
**Objetivo:** JSON-RPC sobre stdio com 4 gatilhos (§9 do research).
**Entregáveis:**
- `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`.
- Hints como **ponteiros** (`id + statement + score`), nunca corpos.
- `kd self setup <claude|cursor|codex|pi>` configurando o cliente.
**Aceite:** handshake MCP válido com um cliente de referência; gatilhos retornam ponteiros; `self setup` idempotente.
**Risco:** baixo (é a vantagem V1).

---

### TS-E14 · Bancada de qualidade e performance
**depends_on:** TS-E13
**Objetivo:** reproduzir a metodologia de avaliação (§8 do research).
**Entregáveis:**
- Harness micro + e2e + **qualidade** (Recall@k/MRR/nDCG@k sobre corpus PT-BR sintético rotulado, baseline versionado).
- Corpus sintético determinístico (`~70% conhecimento / ~20% tarefas / ~10% épicos`).
- A/B do auto-drain e do índice derivado; relatório com `min/mediana/p95/máx/desvio`.
**Aceite:** números reproduzíveis em 2 execuções; baseline de qualidade versionado; gargalos O(N²) monitorados.
**Risco:** médio (sem `criterion`; medição é observação, não gate — paridade com o repo).

---

### TS-E15 · Distribuição e empacotamento
**depends_on:** TS-E14
**Objetivo:** entregar `kd` e `knudge-mcp` como pacotes/binário único (vantagens V2/V3).
**Entregáveis:**
- Publicação npm (`@knudge/core`, `knudge`, `@knudge/mcp`).
- Binário único via `bun build --compile` (Linux/macOS/Windows); completions bash/zsh/fish.
- `install.sh` de paridade (verificação de integridade, PATH, sem destruir instalação anterior).
**Aceite:** `npx knudge` funciona sem instalar; binário compilado roda `kd ask` num projeto de fixture; completions carregam.
**Risco:** médio (empacotamento cross-platform é chato).

---

### TS-E16 · Conformidade e paridade com o Rust
**depends_on:** TS-E15
**Objetivo:** o gate que impede a "segunda implementação" de divergir em silêncio (**D12**).
**Entregáveis:**
- **Conformance corpus**: notas geradas nas duas versões; comparar `id`, `body_hash`, TOON emitido (byte-a-byte) e top-k do `ask`.
- Golden cruzado das mensagens de erro/exit codes (re-baselinadas onde a decisão B foi aplicada — documentadas como borda, à moda de `DIVERGENCES.md`).
- Interop real: um `.knudge/` escrito pelo Rust é lido pelo TS e vice-versa, sem perda.
**Aceite:** corpus de conformidade 100 % verde; divergências remanescentes catalogadas e justificadas.
**Risco:** 🔴 alto (é onde a decisão §1.3 se prova ou falha).

---

## 7. Grafo de dependências (resumo)

```
TS-E01 → TS-E02 → TS-E03 → TS-E04 → TS-E05 → TS-E06
     → TS-E07 → TS-E08 → TS-E09 → TS-E10 → TS-E11
     → TS-E12 → TS-E13 → TS-E14 → TS-E15 → TS-E16
```

Cadeia estritamente linear. Marcos de valor:
- **Após TS-E06:** corpus legível/gravável, bit-compatível → *interop provado cedo*.
- **Após TS-E11:** paridade funcional completa do núcleo.
- **Após TS-E13:** produto utilizável (CLI + MCP).
- **Após TS-E16:** substituto confiável do Rust.

---

## 8. Cronograma / esforço relativo (ordem de magnitude)

| Fase | Issues | Esforço relativo | Comentário |
|---|---|---|---|
| F0 Fundação & determinismo | E01–E02 | ██████ (alto) | `libm` determinístico é o item mais delicado |
| F1 Contrato de bytes | E03–E04 | █████ | parsing + ids bit-exatos |
| F2 Persistência | E05–E06 | ████ | atômico + lock + rebuild |
| F3 Inteligência | E07–E11 | ██████████ (maior) | 5 subsistemas com matemática |
| F4 Superfície | E12–E13 | █████ | CLI + MCP |
| F5 Validação & entrega | E14–E16 | ██████ | bancada + conformidade |

*(estimativa relativa, não em semanas — a calibração depende de quantos devs e se a meta é §1.3-A ou §1.3-B.)*

---

## 9. Riscos (registro priorizado)

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| **R1** — não-determinismo transcendental (D1) | alta | alto | `libm` próprio (TS-E02) **ou** runtime pinado + goldens por versão |
| **R2** — divergência de interop Rust↔TS (D12) | média | alto | Conformance corpus (TS-E16) desde cedo, não só no fim |
| **R3** — startup lento inviabiliza "sem daemon" (D3) | média | alto | `bun/deno compile`; medir `--help` como gate |
| **R4** — CPU-bound em corpora grandes (D4) | média | médio | `worker_threads` + LSH; opcional `napi-rs` |
| **R5** — ordem não-canônica por `Map`/`Object` (D5) | média | médio | `SortedMap` obrigatório; lint |
| **R6** — escopo crescer para "porte 1:1" | média | médio | Adotar §1.3-B (re-baseline de saída) explicitamente |

---

## 10. Definição de pronto (Definition of Done)

- Contratos de bytes (`id`, `body_hash`, TOON) **bit-idênticos** ao corpus Rust (TS-E16 verde).
- Determinismo provado por execução 2× em Node **e** Bun (com `libm` próprio) — ou garantia documentada por versão.
- Cadeia TS-E01→E16 com **todos** os critérios de aceite satisfeitos e testes de regressão.
- CLI e MCP utilizáveis, com a matriz de aceite (pipe/`--json`/exit) por verbo.
- Bancada de qualidade com baseline versionado; `--help` < 20 ms no binário compilado.
- Achados de usabilidade humana (`GOOSE_SUGGESTION.md`) resolvidos na superfície re-baselinada.

---

## 11. Nota de fecho

A reimplementação em TypeScript **não é um exercício de tradução** — é uma **segunda especificação executável** do knudge. O que decide o sucesso não é a linguagem, e sim **duas apostas**: (i) uma camada de matemática determinística própria, e (ii) um *conformance corpus* que force as duas versões a concordarem no que importa (`id`, `body_hash`, bytes). Com essas duas, TypeScript troca ~5 ms de startup por **portabilidade, ecossistema MCP e velocidade de iteração** — um trade defensável. Sem elas, a reimplementação vira uma **segunda verdade** — exatamente o problema que o knudge existe para resolver.

---

*Baseado em `GOOSE_RESEARCH.md` (knudge v0.5.2, commit `2ef623c`) e nas specs `wiki/specs/*`, `plan/implementation/*` e `bench/RELATORIO.md`. Complementa `GOOSE_SUGGESTION.md` (correções de UX incorporadas em TS-E12).*
