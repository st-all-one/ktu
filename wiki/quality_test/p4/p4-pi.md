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
agent: pi 
---

# PI_EPIC — Reimplementação do `knudge` em TypeScript

> **Base:** `PI_RESEARCH.md` (resumo técnico do `/REF/knudge` v0.5.2).
> **Objetivo:** especificar a reimplementação completa do `knudge` (CLI `kd` + servidor MCP +
> biblioteca `core`) em **TypeScript**, preservando os invariantes do sistema original —
> determinismo byte a byte, nota como fonte da verdade, índice derivado reconstruível, contrato
> `stdout`/`--json`.
> **Formato:** plano mestre por fases + 38 issues encadeadas sequencialmente (cada uma com
> dependências, entregável e critérios de aceite rastreáveis ao código Rust).
> **Nome de trabalho do projeto:** `knudge-ts` (pacotes `@knudge/*`).

---

## Sumário

1. [Escopo e não-escopo](#1-escopo-e-não-escopo)
2. [Vantagens do TypeScript nesta reimplementação](#2-vantagens-do-typescript-nesta-reimplementação)
3. [Desvantagens e riscos](#3-desvantagens-e-riscos)
4. [Mapeamento Rust → TypeScript](#4-mapeamento-rust--typescript)
5. [Invariantes que NÃO podem ser quebrados](#5-invariantes-que-não-podem-ser-quebrados)
6. [Arquitetura-alvo](#6-arquitetura-alvo)
7. [Plano mestre (fases e marcos)](#7-plano-mestre-fases-e-marcos)
8. [Estratégia de qualidade e paridade](#8-estratégia-de-qualidade-e-paridade)
9. [Issues encadeadas](#9-issues-encadeadas)
10. [Grafo de dependências](#10-grafo-de-dependências)
11. [Empacotamento e distribuição](#11-empacotamento-e-distribuição)
12. [Riscos residuais e decisões abertas](#12-riscos-residuais-e-decisões-abertas)

---

## 1. Escopo e não-escopo

### 1.1 Escopo

Reimplementar **a superfície e a semântica** do knudge 0.5.2 em TypeScript:

- **Dados:** nota (frontmatter TOON + corpo Markdown), `id` endereçado por conteúdo, schema
  canônico de 31 chaves, enums fechados, claims/proveniência/ontologia.
- **Persistência:** `notas/` (verdade), `eventos/*.jsonl` (auditoria), `.idx/` (derivado),
  escrita atômica, lock advisory, rebuild double-buffer, purge/sweep.
- **Retrieval:** BM25 *fielded*, âncoras, embeddings opcionais, PPR, fusão RRF, confiança
  Beta-Bernoulli/Wilson, idade, drift.
- **Grafo:** arestas explícitas, integridade, SCC, PageRank/PPR, Louvain, ontologia, TMS.
- **Escrita:** protocolo de dedup (Dice + MinHash/LSH), merge, update/supersede, lote.
- **Tarefas:** view derivada, hierarquia, evidência, graph/plan/flow (PERT/CPM).
- **Handoff:** `prime` (estático) e `rewind` (dinâmico).
- **Saúde/manutenção:** `doctor` (13 checks + auditoria), `learn`/`compact`/`prune`, `map`.
- **Interfaces:** CLI de 14 verbos, servidor MCP (JSON-RPC sobre stdio), biblioteca embutível.

### 1.2 Não-escopo (v1)

- Migração de dados do knudge Rust (formato é compatível; veremos como aceite, não como migrador).
- Novas funcionalidades além da paridade 0.5.2 (reranking cross-encoder, ANN) — ficam para v2.
- Interface gráfica, TUI interativa, sincronização em nuvem, ACL.
- Suporte a runtimes exóticos (Deno Deploy, edge workers) — alvo é Node ≥ 20 / Bun.

---

## 2. Vantagens do TypeScript nesta reimplementação

### 2.1 Vantagens de ecossistema

| # | Vantagem | Impacto concreto |
|---|---|---|
| V1 | **Mesmo runtime do ferramental de agentes** | O público-alvo (agentes, MCP, plugins de editor) já vive em Node/TS; integração direta sem FFI. |
| V2 | **SDK MCP oficial em TS** | `@modelcontextprotocol/sdk` elimina a reimplementação do transporte MCP; os 4 gatilhos ficam triviais. |
| V3 | **Distribuição por npm** | `npx @knudge/cli` em uma linha; sem `curl \| bash`, sem checagem de SHA manual (npm valida integridade). |
| V4 | **O contrato de dados já é JSON/JSONL** | Eventos, índice, sugestões e o envelope de saída são JSON nativos; serialização é idiomática. |
| V5 | **Unicode de primeira classe** | `String.prototype.normalize('NFC'/'NFD')` implementa o *fold* de diacríticos exigido (D172) sem crate. |
| V6 | **Ecossistema de teste rico** | `vitest`, `fast-check` (property-based), `zod` (schema runtime), `memfs` (fake FS), `msw` (HTTP fake). |
| V7 | **Velocidade de iteração** | Sem compilação pesada; hot reload; onboarding de contribuidores muito mais barato. |
| V8 | **Uma linguagem para core + CLI + MCP + scripts** | Reduz superfície cognitiva e permite compartilhar tipos entre camadas. |
| V9 | **Tipagem estrutural + union types** | Modela bem enums fechados, TOON AST e resultados discriminados (`created/merged/rejected`). |

### 2.2 Vantagens estratégicas

- **Contribuição:** o custo de entrada cai de "instalar toolchain Rust 1.97 + edição 2024" para
  "`pnpm i`". Aumenta a probabilidade de adoção e de PRs.
- **Extensibilidade:** usuários podem escrever *hooks* e *validators* em TS, no mesmo runtime do
  agente, sem `cmd` externo (o que hoje exige `.knudge/validators.toml` + `HookRunner`).
- **Reuso em produto:** `@knudge/core` vira dependência natural de servidores de agentes,
  extensões de IDE e pipelines de CI.

---

## 3. Desvantagens e riscos

### 3.1 Riscos técnicos (com mitigação)

| # | Risco | Gravidade | Mitigação |
|---|---|---|---|
| R1 | **Determinismo difícil**: ordem de `Map`/objetos, `localeCompare`, `Date.now`, iteração de `Set` | **Alta** | Proibir `localeCompare`/`Intl` no core; ordenar por `sort` com comparador total explícito; nunca ordenar por conversão implícita; lint para `Date.now`/`Math.random` fora de portas. |
| R2 | **Performance CPU-bound**: BM25, PageRank, Louvain, MinHash, dedup são O(N·d) ou piores | **Alta** | `worker_threads` para leitura paralela e blocos pesados; tipar arrays (`Float64Array`, `Int32Array`); *blocking* MinHash/LSH desde o início; benchmarks-gate. |
| R3 | **Startup por invocação**: Node ~40–100 ms vs. ~4–13 ms do binário Rust | **Alta** para CLI | *Bundling* + `bun build --compile` (binário standalone) ou Node SEA; medir e otimizar o caminho de startup; lazy-init de logging. |
| R4 | **Sem verificação exaustiva de enums em runtime** | Média | `zod` nas bordas + discriminated unions; testes de contrato; nunca confiar só nos tipos. |
| R5 | **Atomicidade/rename no Windows** | Média | Adapter FS com fallback (`rename` é atômico no mesmo volume; tratar `EPERM`/`EACCES` com retry); testes em Windows no CI. |
| R6 | **Lock advisory sem `O_CREAT\|O_EXCL` portátil** | Média | Usar `fs.open(path, 'wx')` (exclusivo) + sidecar de inode/mtime; documentar bordas. |
| R7 | **Memória em corpora grandes** | Média | Índice invertido com `Map<string, Int32Array>`; streaming de JSONL; *lazy* de embeddings; medir RSS. |
| R8 | **Contrato de bytes** (TOON canônico, ordem fixa) | **Alta** | Parser/emissor próprio + goldens portados do Rust; nunca usar `JSON.stringify` como contrato. |
| R9 | **Corrida async/await** | Média | Preferir I/O síncrono no caminho de comando (como o Rust bloqueante); async só onde necessário; lint `no-floating-promises`. |
| R10 | **Cadeia de suprimentos npm** | Média | Dependências mínimas, *lockfile* pinado, `--ignore-scripts`, auditoria; core com zero deps de runtime. |
| R11 | **Divergência do binário Rust** | **Alta** | Suíte de conformidade diferencial: rodar `kd` (Rust) e `kds` (TS) sobre o mesmo corpus e comparar bytes/exit codes. |
| R12 | **Disciplina de código** (o Rust proíbe `unwrap`/`panic`/300 linhas) | Média | ESLint com `no-explicit-any`, `no-throw-literal`, `max-lines`, `complexity`; `strict` total; `Result` explícito em vez de `throw` no core. |

### 3.2 Riscos de projeto

- **R13 — Escopo:** paridade completa é grande (≈57k linhas Rust, ~922 testes). Mitigação: fases
  com *vertical slices* usáveis desde a Fase 1.
- **R14 — Duas implementações:** manter Rust e TS em paridade gera custo. Mitigação: congelar o
  contrato 0.5.2 e usar conformance suite como fonte de verdade.
- **R15 — Atração de "só mais uma lib":** risco de inflar dependências. Mitigação: core com zero
  dependências de runtime; CLI pode usar poucas.

---

## 4. Mapeamento Rust → TypeScript

### 4.1 Estrutura de pacotes

| Rust | TypeScript | Observação |
|---|---|---|
| `crates/knudge-core` | `packages/core` (`@knudge/core`) | Zero deps de runtime; puro; portas. |
| `crates/knudge-cli` (`kd`) | `packages/cli` (`@knudge/cli`, bin `kds`) | `commander`/`clipanion`; output. |
| `crates/knudge-mcp` | `packages/mcp` (`@knudge/mcp`) | `@modelcontextprotocol/sdk`. |
| `fuzz/` | `fuzz/` com `fast-check` | Property + fuzz determinístico. |
| `bench/` | `bench/` com benchmark harness próprio | Paridade de métricas. |

### 4.2 Conceitos da linguagem

| Rust | TypeScript equivalente | Cuidado |
|---|---|---|
| `BTreeMap` / `IndexMap` | `Map` + ordenação explícita, ou classe `OrderedMap` | `Map` preserva inserção, **não** ordem canônica; ordenar na emissão. |
| `Result<T, Error>` | `type Result<T> = { ok: true; value: T } \| { ok: false; error: KnudgeError }` | Não usar `throw` no core; `throw` só nas bordas. |
| `enum` fechado | union de literais + guardas `zod` | Validar entrada em runtime. |
| `trait` | `interface` + injeção via construtor | Fakes implementam a mesma interface. |
| `#[derive(Serialize)]` | emissor manual + `zod` para leitura | Contrato de bytes exige emissor explícito. |
| `u64`/`i64` ms | `number` (f64 seguro até 2^53) ou `bigint` | Timestamps em ms cabem em `number`; hashes em `Uint8Array`. |
| `SHA-256` (`sha2`) | `node:crypto` `createHash('sha256')` | Byte-exato; truncar 4 bytes BE. |
| `f64::total_cmp` | comparador total próprio (NaN→0) | Nunca `a - b` cru. |
| `std::fs` | `node:fs` síncrono no caminho quente | Evitar `fs/promises` onde ordem importa. |
| `O_CREAT\|O_EXCL` | `fs.openSync(path, 'wx')` | Lock exclusivo. |
| `rename(2)` atômico | `fs.renameSync` | Mesmo diretório/volume. |
| `git` via `Command` | `node:child_process.spawnSync` sem shell | Sem `exec` com shell. |
| `reqwest` HTTP | `fetch` global + `undici` | Bloqueante não existe; usar `await` no canal de embeddings. |
| `proptest` | `fast-check` | Determinismo com seed fixa. |
| `criterion` | `vitest bench` / harness próprio | Mesmas métricas. |
| `unsafe` banido | ESLint + revisão | TS não tem a garantia; disciplina via lint. |

### 4.3 Estratégia de determinismo

1. **Portas obrigatórias para o mundo impuro:** `Clock`, `Random` (só jitter), `Env`, `Fs`,
   `Git`, `HookRunner`, `Logger`. Nada no core chama `Date.now`, `Math.random`, `process.env`
   ou `node:fs` diretamente.
2. **Ordenação canônica centralizada:** `compareCanonical(a, b)` para scores (total, NaN→0) e
   `sortById` para empates.
3. **Arrays numéricos** (`Float64Array`) para vetores e scores, com operações determinísticas.
4. **Proibição de `Intl`, `toLocaleString`, `localeCompare`** no core (lint customizado).
5. **Seed fixa em testes** e fakes (`FixedClock`, `SeqRng`, `MemFs`, `FaultyFs`, `FakeGit`).

---

## 5. Invariantes que NÃO podem ser quebrados

Copiados do material de referência; cada um vira um teste de aceite global:

| ID | Invariante | Fonte |
|---|---|---|
| I1 | `stdout` = dados; `stderr` = logs; `--json` = `{success, command, data?, error?, warnings?}` | D71/R20 |
| I2 | `id = <type>_<base36(8)>` derivado de `normalize(statement)`; reclassificar não muda o id | D01/D02/D95 |
| I3 | Ordem canônica das 31 chaves; opcionais omitidos, nunca `null` | D04/D05 |
| I4 | `notas/` é a verdade; apagar `.idx/` nunca perde conhecimento (reconstruível byte a byte) | D15/D27/D84 |
| I5 | Dedup: `<0.75` cria, `[0.75,0.92)` merge, `≥0.92` rejeita | D26/D80 |
| I6 | Busca vazia → `[no_results]`, exit 0; EPIPE → exit 0 | D152/D73 |
| I7 | Ciclo de vida é *soft* (`status`); nada é apagado por `forget` | D84 |
| I8 | `doctor`/`learn`/`compact`/`prune` só propõem | D47 |
| I9 | Exit codes por `ErrorKind` (2,3,4,5,6,7,8,9,70) | erros.md |
| I10 | Nenhuma confiança/derivado é persistido | D87 |

---

## 6. Arquitetura-alvo

### 6.1 Diagrama de camadas

```
@knudge/mcp ──┐
              ├── @knudge/core (puro + portas + adapters std)
@knudge/cli ──┘
```

Estrutura interna de `@knudge/core`:

```
packages/core/src/
  error/            # KnudgeError, ErrorKind, code→exit
  time/             # Timestamp (number ms UTC)
  logging/          # redação de segredos
  ports/            # interfaces + fakes
  adapters/         # node:fs, git, http, process hooks
  config/           # 2 níveis, schema, TOML subset
  schema/           # 31 chaves, enums, id, body_hash, claims, provenance, slots
  toon/             # parser + emissor canônico
  jsonl/            # leitura/escrita + rotação
  store/            # notas, eventos, lock, rebuild, purge, sweep
  git/              # worktree, info/exclude, AGENTS.md, sync
  graph/            # arestas, integridade, rank, communities, ontology, tms
  retrieval/        # tokenizer, bm25, anchors, filters, views, rrf, weights
  write/            # protocolo, dedup (dice + minhash/lsh), update, lifecycle
  handoff/          # prime, rewind, manifest, working set
  maintenance/      # diff, learn, compact, prune, proposals
  task/             # hierarquia, progresso, impacto, flow
  health/           # validators, audit, doctor (13 checks)
  lifecycle/        # beta/wilson, fsrs, drift, clusters
  embeddings/       # index, cache, rank_query, suggest, drain
  knudge.ts         # fachada pública (Knudge builder)
```

### 6.2 Regras de engenharia (equivalência ao AGENTS.md)

- `strict: true`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`.
- ESLint proíbe `any`, `throw` de literal, `Date.now`/`Math.random` fora de portas, `Intl`,
  `localeCompare`, `console.log` no core.
- `max-lines: 300` por arquivo (paridade com o Rust).
- Core com **zero dependências de runtime**; CLI/MCP com o mínimo.
- Todo módulo exporta funções puras testáveis; impureza só em `adapters/`.

---

## 7. Plano mestre (fases e marcos)

### Visão geral

| Fase | Tema | Issues | Marco | Entrega demonstrável |
|---|---|---|---|---|
| **F0** | Fundação | 001–005 | **M0** | `@knudge/core` compila, erros, config, portas |
| **F1** | Dados e store | 006–011 | **M1** | `init` + `write`/`read` de nota byte-exata |
| **F2** | Retrieval | 012–018 | **M2** | `kds ask` híbrido com `--json` |
| **F3** | Write-path | 019–021 | **M3** | dedup/merge/supersede + lote |
| **F4** | Grafo e ciclo de vida | 022–024 | **M4** | PPR/comunidades/TMS + confiança |
| **F5** | Tarefas e handoff | 025–027 | **M5** | `task` + `rewind` + `prime` |
| **F6** | Saúde e manutenção | 028–030 | **M6** | `doctor`/`maintenance`/`map` |
| **F7** | Interfaces | 031–033 | **M7** | CLI completa + MCP + `self` |
| **F8** | Qualidade | 034–038 | **M8** | conformidade diferencial + release |

### Critérios de "pronto" por fase (Definition of Done)

1. Testes de unidade + property verdes; *goldens* quando há contrato de bytes.
2. Nenhuma regressão nos invariantes I1–I10 (suíte de invariantes executada no CI).
3. Benchmark da fase dentro do orçamento definido (ver 8.3).
4. Documentação `docs/` da fase + entradas de decisão (`docs/adr/`).

### Orçamento de performance-alvo (paridade funcional, não micro)

| Operação | Rust 0.5.2 (N=1167) | Alvo TS aceitável (N=1167) |
|---|---:|---:|
| `ask` (query) | ~43 ms | ≤ 3× o Rust (≤ ~130 ms) |
| `rewind` | ~134 ms | ≤ 3× (≤ ~400 ms) |
| `doctor` | ~4,2 s | ≤ 2× com workers (≤ ~8 s) |
| `self version` (startup) | ~3,5 ms | ≤ 25 ms (binário compilado) |

---

## 8. Estratégia de qualidade e paridade

### 8.1 Camadas de teste

1. **Unidade** (`vitest`) — funções puras.
2. **Property** (`fast-check`) — `normalize`, `id`, `body_hash`, RRF, confiança, FSRS,
   monotonicidade; seed fixa.
3. **Golden** — TOON, envelope `--json`, saída de `init`, `doctor`, `ask`.
4. **Fuzz** — parser TOON, parser TOML subset, JSONL malformado.
5. **Conformidade diferencial** — corpus comum; para cada comando, comparar
   `{stdout, stderr, exitCode, sideEffects}` entre `kd` (Rust) e `kds` (TS). Divergência = falha.
6. **Invariantes** — I1–I10 como testes executáveis.

### 8.2 Fonte de verdade

O binário Rust 0.5.2 é a **referência normativa**. Onde houver conflito, o TS se ajusta (ou
registra-se a borda em `docs/DIVERGENCES.md`). As ~115 bordas do Rust devem ser portadas e
reproduzidas.

### 8.3 Portões de performance

- Micro-benchmarks de `normalize`/`body_hash`/`note_id`/`toon::parse` com baseline.
- E2E de `ask`, `rewind`, `doctor`, `task` em N=235 e N=1167.
- Falhar o CI se ultrapassar o orçamento acima.

---

## 9. Issues encadeadas

> **Convenção:** cada issue depende da anterior (`←`) salvo indicação. Entregáveis e critérios de
> aceite rastreiam o material de referência. Estimativas em **pontos** (1, 2, 3, 5, 8, 13).

### FASE 0 — Fundação

---

#### TS-001 — Scaffold do monorepo `knudge-ts`

- **Depende de:** —
- **Estimativa:** 3
- **Entregável:** monorepo pnpm + Turborepo com `packages/core`, `packages/cli`, `packages/mcp`;
  `tsconfig` base estrito; `eslint`/`prettier`; `vitest`; CI (GitHub Actions) lint+test+build.
- **Descrição:** criar a fundação de build e as regras de código equivalentes ao `AGENTS.md`
  (max-lines 300, sem `any`, sem `Intl` no core).
- **Aceite:** `pnpm -r build`, `pnpm -r test`, `pnpm lint` verdes; regra de lint que falha ao usar
  `Date.now`/`Math.random` fora de `adapters`/`ports`.
- **Ref:** `Cargo.toml`, `AGENTS.md`, `clippy.toml`, `rustfmt.toml`.

---

#### TS-002 — Primitivas determinísticas

- **Depende de:** TS-001
- **Estimativa:** 5
- **Entregável:** `core/src/determinism`: `compareCanonical`, `compareScore` (total, NaN→0),
  `clamp01`, arrays numéricos determinísticos, codec JSON canônico, `sortById` estável.
- **Descrição:** centralizar todas as decisões de ordenação e aritmética para evitar
  não-determinismo de `Map`/`Set`/`sort` implícito.
- **Aceite:** property tests provam ordem total e estabilidade; nenhum uso de `localeCompare`/
  `Intl` no core; teste que roda 1000× e produz bytes idênticos.
- **Ref:** `wiki/specs/determinismo.md`, `matematica.md` §Convenções.

---

#### TS-003 — Taxonomia de erros e exit codes

- **Depende de:** TS-002
- **Estimativa:** 2
- **Entregável:** `core/src/error`: `KnudgeError`, `ErrorKind`, mapa `code→exit`
  (2,3,4,5,6,7,8,9,70), `retryable` (só `Timeout`), envelope `{success, command, data?, error?, warnings?}`.
- **Aceite:** todos os kinds mapeados; teste por kind; envelope idêntico ao Rust nos goldens.
- **Ref:** `wiki/specs/erros.md`, D71–D73.

---

#### TS-004 — Portas e fakes

- **Depende de:** TS-002, TS-003
- **Estimativa:** 5
- **Entregável:** `core/src/ports/`: `Clock`, `Random`, `Env`, `Fs`, `Git`, `HookRunner`,
  `Logger` + fakes (`FixedClock`, `SeqRng`, `MemFs`, `FaultyFs`, `FakeGit`, `RecordingLogger`,
  `FakeEmbedder`).
- **Descrição:** nenhum módulo de domínio importa `node:fs`/`node:child_process`/`process`
  diretamente. `FaultyFs` injeta falhas em pontos determinísticos.
- **Aceite:** lint proíbe imports impuros no core; suíte de crash test com `FaultyFs`.
- **Ref:** `wiki/specs/ARCHITECTURE.md` §3, D65.

---

#### TS-005 — Configuração em dois níveis + TOML subset

- **Depende de:** TS-004
- **Estimativa:** 8
- **Entregável:** `core/src/config`: parser/emissor TOML subset próprio (sem lib), schema de
  chaves, config global + projeto, `Config::effective` com sanitização de segredos; comandos
  `config get|set|unset|list|promote`.
- **Aceite:** round-trip preserva ordem (diff mínimo); escrita canônica; chave desconhecida →
  warning + sugestão (Levenshtein, sem sugerir a própria string); segredos só no global.
- **Ref:** D61/D91/D97, `wiki/specs/configuracao.md`.

---

**Marco M0:** `@knudge/core` instala, valida config, erros e portas; fakes testáveis.

---

### FASE 1 — Dados e store

---

#### TS-006 — TOON: parser e emissor canônico

- **Depende de:** TS-005
- **Estimativa:** 13
- **Entregável:** `core/src/toon`: parser tolerante + emissor canônico; AST tipada; escapes
  `\n\r\t\0\u{}`; aninhamento ≤32; documento vazio → 0 bytes; `LF` final.
- **Aceite:** goldens portados do Rust; round-trip byte-exato; fuzz do parser sem crash; erros
  com linha/coluna.
- **Ref:** `wiki/specs/TOON.md`, D74/D75.

---

#### TS-007 — Schema, enums e identidade

- **Depende de:** TS-006
- **Estimativa:** 8
- **Entregável:** `core/src/schema`: 31 chaves em ordem canônica; enums fechados; `normalize`
  (NFC + trim + colapso); `noteId(type, statement)`; `bodyHash`; `claims`, `provenance`,
  ontologia; `REQUIRED_KEYS`; slots por espécie; guards `zod`.
- **Aceite:** `id` determinístico e independente de reclassificação; `body_hash` cobre
  statement+corpo; chave desconhecida rejeita no write e alerta no read.
- **Ref:** `llms.txt` §Modelo de dados, D01/D02/D95/D207/D191.

---

#### TS-008 — Store de notas e layout material

- **Depende de:** TS-007
- **Estimativa:** 13
- **Entregável:** `core/src/store`: `notas/<tipo>/<id>.md`, leitura/escrita atômica
  (`tmp+rename`, `fsync` em batch), `read_optional` tolerante, detach/purge, `load_notes`
  paralela preservando ordem de `list_ids`.
- **Aceite:** crash entre tmp e rename não corrompe; leitura tolerante skipa nota malformada com
  orientação; layout derivado do prefixo (D150).
- **Ref:** `wiki/specs/persistencia.md`, D20/D22, E15-T12.

---

#### TS-009 — Log de eventos (JSONL)

- **Depende de:** TS-008
- **Estimativa:** 8
- **Entregável:** `core/src/jsonl` + eventos: `{id, op, note_id?, at, actor?, data?}`; dedup
  on-read por id; rotação por `max_bytes`; checkpoint; `merge=union`.
- **Aceite:** ordem de commit nota→evento; crash deixa nota sem evento; rotação e leitura
  tolerante testadas; idempotência sob replay.
- **Ref:** D21/D28/D31, R13.

---

#### TS-010 — Lock, rebuild double-buffer e sweep

- **Depende de:** TS-009
- **Estimativa:** 8
- **Entregável:** lock advisory por arquivo (`fs.open('wx')` + sidecar inode/mtime, stale 30s,
  reclaim por rename, RAII); rebuild `.idx.new/` + rename; sweep de `*.tmp`/`*.stale`.
- **Aceite:** dois processos não escrevem o mesmo alvo; rebuild nunca expõe índice parcial;
  sweep nunca remove lock vivo.
- **Ref:** D23–D27/D160.

---

#### TS-011 — Adapter Git, worktree, AGENTS.md e `sync`

- **Depende de:** TS-010
- **Estimativa:** 8
- **Entregável:** worktree principal via `git rev-parse --git-common-dir`; `.git/info/exclude`;
  bloco idempotente em `AGENTS.md` + skill `.agents/skill/kd/SKILL.md`; `sync` (add notas+eventos
  + commit com mensagem do último evento); `.gitattributes` `merge=union`.
- **Aceite:** reexecutar não duplica; nunca usa `.gitignore`; worktree compartilha layout;
  `sync` sem shell.
- **Ref:** D29–D32/D60/D162.

---

**Marco M1:** `kds init` + `kds write` + `kds ask --id` produzindo artefatos byte-compatíveis.

---

### FASE 2 — Retrieval

---

#### TS-012 — Tokenizer (fold, stopwords, stemmer PT)

- **Depende de:** TS-008
- **Estimativa:** 8
- **Entregável:** `core/src/retrieval/tokenizer`: `[a-z0-9_]`, fold NFD de diacríticos,
  stopwords PT+EN, fragmentos de 1 char descartados, stemming PT conservador (radical ≥4 bytes).
- **Aceite:** `café≡cafe`; property tests de idempotência; corpus de tokens golden.
- **Ref:** D122/D172/D206, `INDEX_FORMAT="retrieval-v4"`.

---

#### TS-013 — BM25 *fielded* + índice invertido

- **Depende de:** TS-012
- **Estimativa:** 13
- **Entregável:** `core/src/retrieval/bm25`: campo (statement 3.0/tags 2.0/body 1.0),
  IDF Robertson-Sparck Jones por campo, `k1=1.5`, `b=0.75`, `type_weight`, boost de confirmação,
  corte de alta frequência (`df/N≥0.9`, `N≥64`), índice invertido em memória.
- **Aceite:** scores batem com goldens do Rust dentro de tolerância 1e-9; property de
  monotonicidade; benchmark dentro do orçamento.
- **Ref:** `matematica.md` §1, D35–D38/D173.

---

#### TS-014 — Canal de âncoras e globs

- **Depende de:** TS-013
- **Estimativa:** 5
- **Entregável:** matching por path/glob (`?`, `*`, `**` via DP); `--anchor` só entra com
  interseção do working set.
- **Aceite:** globs golden; micro-benchmark do glob; âncora quebrada sinaliza, não apaga.
- **Ref:** D81/D86, E15-T07.

---

#### TS-015 — Filtros determinísticos e views

- **Depende de:** TS-013
- **Estimativa:** 5
- **Entregável:** filtros `type/class/status/tags/anchors/since/until`; views `ready`/`blocked`
  via `depends_on` transitivo; ciclo = `blocked`.
- **Aceite:** testes de transitividade; ciclo tratado sem loop; `--scope` resolvido.
- **Ref:** D41/D53/D149.

---

#### TS-016 — Embeddings: porta, HTTP, cache e drain

- **Depende de:** TS-004, TS-013
- **Estimativa:** 13
- **Entregável:** `core/src/embeddings`: `Embedder` port; adaptador HTTP OpenAI-compatible
  (`fetch`, default `http://127.0.0.1:8889/v1/embeddings`); providers `http`/`lightweight`/`none`;
  índice `.idx/embeddings.jsonl` com `meta`; cache por `(body_hash, model)`; modo `lazy`; estados
  `indexed|pending|stale`.
- **Aceite:** falha de provedor degrada para BM25 com `warnings`; troca de modelo invalida e
  re-embeda; falha marca `pending` e nunca descarta.
- **Ref:** D102/D131/D148/D202/D182.

---

#### TS-017 — RRF, `why`, confiança, idade e drift

- **Depende de:** TS-013, TS-016
- **Estimativa:** 13
- **Entregável:** `core/src/retrieval/rrf`: `Σ peso/(k+rank+1)`, `k=60`, pesos
  `1/2/30/0`; desempate `(score desc, id asc)`; `why` fechado; composição de confiança
  (Beta+Wilson), `age_factor`, `drift_factor`; penalidade de contradição.
- **Aceite:** property de monotonicidade; `channels` com parcelas no `--json`; ordem estável.
- **Ref:** `matematica.md` §2–5, D81/D123/D175/D177/D179/D189/D203/D192.

---

#### TS-018 — Comando `ask` (modos e saída)

- **Depende de:** TS-014, TS-015, TS-017
- **Estimativa:** 13
- **Entregável:** `kds ask`: query/`--id`/`--around`/`--rank`/`--tags`/`--suggest`; filtros;
  `--limit/--brief/--full-content/--with-task/--params/--as-of`; pipe `id|statement|score|why`;
  revelação progressiva; `[no_results]`; `--json` com `channels`.
- **Aceite:** conformance diferencial com Rust para um corpus fixture; `--json` idêntico;
  escopo obrigatório nos modos de rank.
- **Ref:** `plan/implementation/16_cli_surface.md` §3, D161/D151/D152.

---

**Marco M2:** `kds ask` híbrido com paridade de saída.

---

### FASE 3 — Write-path

---

#### TS-019 — Protocolo de escrita + dedup Dice + merge/update/supersede

- **Depende de:** TS-018, TS-011
- **Estimativa:** 13
- **Entregável:** duas fases (propose→decisão); limiares 0.75/0.92; merge (união de
  tags/âncoras/outcomes, `revision++`, corpo não sobrescrito); `--update`/`--clear-anchors`;
  `--link`/`--edge`; `--outcome`; `kd write` rejeita `task`/`epic`.
- **Aceite:** `created|merged|rejected|updated` conforme limiar; supersede gera
  `replaces`/`superseded_by` bidirecional; idempotência sob retry.
- **Ref:** D26/D48/D80/D93/D103/D126/D140/D168.

---

#### TS-020 — Dedup escalável MinHash + LSH

- **Depende de:** TS-019
- **Estimativa:** 8
- **Entregável:** blocking MinHash (64 perm) + LSH (16×4) acima de 256 notas densas;
  candidatos passam pelo Dice exato; propostas idênticas ou superconjunto.
- **Aceite:** teste de equivalência contra peneira exata (recall ≥); benchmark de `doctor`
  denso dentro do orçamento.
- **Ref:** D204, `matematica.md` §Dedup.

---

#### TS-021 — Lote de escrita e `--dry-run`

- **Depende de:** TS-019
- **Estimativa:** 5
- **Entregável:** `write --batch -` (JSONL), `task new --batch`; best-effort com `warnings[]`;
  teto `batch_max=100`; `--dry-run`.
- **Aceite:** lote parcial reporta por item; atômico quando possível; `--dry-run` não toca disco.
- **Ref:** D110/D141.

---

**Marco M3:** ciclo `ask→write` completo com anti-redundância.

---

### FASE 4 — Grafo e ciclo de vida

---

#### TS-022 — Grafo, integridade e ciclos

- **Depende de:** TS-007, TS-019
- **Estimativa:** 8
- **Entregável:** extração de arestas do frontmatter; integridade (quebradas, órfãos,
  bidirecionalidade); SCC (Kosaraju iterativo) sobre `replaces`/`depends_on`; `expand` BFS.
- **Aceite:** membros de ciclo não demovem; `expand` só no explícito; sugestões nunca viram aresta.
- **Ref:** D45/D49/D50/D51.

---

#### TS-023 — PageRank/PPR, Louvain, ontologia e TMS

- **Depende de:** TS-022
- **Estimativa:** 13
- **Entregável:** PageRank/PPR (damping 0.85, ≤32 iter, tol L1 1e-8, dangling); Louvain
  determinístico; classes de equivalência + clausura `broader/narrower` + `claim_conflicts`;
  TMS (`retracted`, `defeated_dependents`, `defeated_by_replacement`).
- **Aceite:** determinismo por property; grafos golden; comunidades estáveis sob permutação de
  entrada.
- **Ref:** D192/D193/D207/D208.

---

#### TS-024 — Ciclo de vida: outcomes, Beta/Wilson, FSRS, drift

- **Depende de:** TS-017, TS-022
- **Estimativa:** 13
- **Entregável:** `lifecycle`: ensaios Bernoulli; posterior + limite inferior Wilson; FSRS-like
  (`ttl_efetivo`, `origin`, `stability_days`); drift de âncoras (`.idx/drift.jsonl`); drift de
  termos JS; demolição por proposta (`Expired|AnchorDecay|Contradicted|Defeated|Drifted`);
  clusters fase 1/2.
- **Aceite:** monotonicidade e conservadorismo (1 sucesso ≈0,21 < 20 sucessos ≈0,84); `forget`
  soft; ciclo protegido.
- **Ref:** `matematica.md` §3–5, D135/D154/D189/D190/D203/D208.

---

**Marco M4:** grafo + confiança + retenção operacionais.

---

### FASE 5 — Tarefas e handoff

---

#### TS-025 — Tarefas como view derivada

- **Depende de:** TS-024
- **Estimativa:** 13
- **Entregável:** `task new/list/show/update/close`; hierarquia `epic ⊃ {issue ⊃ task|task}`;
  marcador de parent no corpo + aresta `results_in`; papel/modo/progresso/impacto derivados;
  evidência obrigatória no close; validators.
- **Aceite:** fechar sem evidência falha; papel/modo derivados corretos; `--ready`/`--blocked`/
  `--sort impact`.
- **Ref:** `wiki/specs/tarefas.md`, D52/D53/D134/D144/D149.

---

#### TS-026 — `task graph`, `plan` e `flow` (PERT/CPM)

- **Depende de:** TS-025
- **Entregável:** `task graph --root|--program`; `task plan --prompt|--submit` (atômico,
  templates `feature|bug|refactor`); `task flow` (cycle/lead/throughput + caminho crítico).
- **Estimativa:** 13
- **Aceite:** plano atômico valida tudo antes de gravar; caminho crítico golden.
- **Ref:** D205, E15-T20.

---

#### TS-027 — Handoff: `prime` e `rewind`

- **Depende de:** TS-024, TS-025
- **Estimativa:** 13
- **Entregável:** `prime [--long]` byte-idêntico; `rewind` (manifest/scope/files/auto); `next:`
  e `fresh:`; orçamento `ceil(len/4)`, default 4000; `context_id` + `--resume` byte-idêntico;
  `diff` sobre eventos.
- **Aceite:** `prime` idêntico entre execuções; `--resume` devolve bytes 1:1; `rewind` dentro do
  orçamento.
- **Ref:** D40/D57/D82/D88.

---

**Marco M5:** gestão de trabalho + retomada de contexto.

---

### FASE 6 — Saúde e manutenção

---

#### TS-028 — `doctor` (13 checks + auditoria + `--fix`/`--explain`)

- **Depende de:** TS-024, TS-027
- **Estimativa:** 13
- **Entregável:** catálogo de checks; auditoria (`body_hash`, âncoras quebradas, locks stale,
  duplicatas, `type: epic`, layout plano, integridade semântica claims/ciclo); `--fix`
  idempotente; `--explain`; `healthy`/`degraded`; `--json` detalhado.
- **Aceite:** `--fix` só corrige o seguro; `body_hash` recalculado; nenhuma remoção de dado sem
  registro; portar bordas H2/H3 do material de usabilidade.
- **Ref:** `llms.txt` §Saúde, D156/D163.

---

#### TS-029 — Manutenção: `learn`, `compact`, `prune`, portão de evidência

- **Depende de:** TS-028
- **Estimativa:** 13
- **Entregável:** `learn` (create/merge/supersede/link a partir de eventos+âncoras);
  `compact` (concat/keep_latest/merge_outcomes); `prune` (motivos); `--verify`/`--dry-run`/
  `--universe`; `proposals.enforce` com `conflict` (4). Tudo **só propõe**.
- **Aceite:** nada aplicado sem aceite; portão de evidência testado.
- **Ref:** D33/D47/D156.

---

#### TS-030 — `map`, clusters e materialização

- **Depende de:** TS-023, TS-029
- **Estimativa:** 8
- **Entregável:** `map --axis anchor|type|classification|scope`, `--semantic`, `--communities`,
  `--members`; `map --write` materializa `notas/MAP.md` + hubs (meta + references); versionado.
- **Aceite:** materialização determinística; hubs referenciam ids válidos; `--universe` exigido
  sem filtro.
- **Ref:** D128/D146/D150/D193.

---

**Marco M6:** saúde e manutenção completas.

---

### FASE 7 — Interfaces

---

#### TS-031 — CLI completa (paridade de 14 verbos)

- **Depende de:** TS-018, TS-021, TS-026, TS-030
- **Estimativa:** 13
- **Entregável:** `kds init|prime|rewind|ask|write|task|map|maintenance|doctor|drain|config|forget|sync|self`;
  `kds` = `help`; help por verbo; listas repetição+vírgula; conjuntos fechados com sugestão;
  `--json` global; exit codes; EPIPE→0.
- **Aceite:** matriz de aceitação (`17_matriz_aceitacao.md`) portada como testes; `stdout` limpo.
- **Ref:** `16_cli_surface.md`, D209/D210/D212/D130/D171/D167.

---

#### TS-032 — Servidor MCP

- **Depende de:** TS-031
- **Estimativa:** 8
- **Entregável:** `@knudge/mcp` com 4 gatilhos (`pre_write`, `pre_edit`, `session_end`,
  `status`) via `@modelcontextprotocol/sdk`; hints = ponteiros (`id+statement+score`), nunca
  corpo; `--stdio`.
- **Aceite:** framing JSON-RPC correto; nenhum corpo de nota vaza para o modelo; teste de
  integração com cliente MCP.
- **Ref:** `wiki/usage/17_mcp.md`, `plan/implementation/18_mcp_transporte.md`.

---

#### TS-033 — `self`, completions e instalador

- **Depende de:** TS-031
- **Entregável:** `self version|setup <claude|cursor|codex|pi>|completions <bash|zsh|fish>`;
  `upgrade`; script `install.sh`/`install.ts` equivalente; verificação de integridade.
- **Estimativa:** 5
- **Aceite:** setup escreve configs idempotentes; completions geradas; instalação sem resíduo.
- **Ref:** D183/D184, README §Quick-start.

---

**Marco M7:** paridade de interface (CLI + MCP).

---

### FASE 8 — Qualidade e release

---

#### TS-034 — Suíte de testes: golden, property, fuzz e invariantes

- **Depende de:** TS-031
- **Estimativa:** 13
- **Entregável:** goldens de TOON/`--json`/`init`/`doctor`/`ask`; `fast-check` nas funções puras;
  fuzz dos parsers; suíte I1–I10; `docs/DIVERGENCES.md` com ~115 bordas portadas.
- **Aceite:** cobertura de módulos-chave ≥ 90 %; toda borda com teste que a trava.
- **Ref:** D76–D78, `wiki/specs/DIVERGENCES.md`.

---

#### TS-035 — Benchmark harness e orçamentos

- **Depende de:** TS-034
- **Estimativa:** 8
- **Entregável:** harness de micro e e2e; corpus sintético determinístico; comparação com
  baselines Rust (`bench/`); portões de CI por orçamento (§7).
- **Aceite:** relatório gerado; falha de CI se exceder orçamento.
- **Ref:** `bench/RELATORIO.md`, `bench/qualidade.md`.

---

#### TS-036 — Conformidade diferencial contra o Rust

- **Depende de:** TS-034
- **Estimativa:** 13
- **Entregável:** harness que roda `kd` (Rust) e `kds` (TS) sobre corpus comum e compara
  `{stdout, stderr, exitCode, sideEffects}`; relatório de divergências.
- **Aceite:** zero divergência nos cenários da matriz de aceitação; divergências documentadas e
  justificadas.
- **Ref:** `17_matriz_aceitacao.md`, `DIVERGENCES.md`.

---

#### TS-037 — Empacotamento standalone e cross-platform

- **Depende de:** TS-031
- **Estimativa:** 8
- **Entregável:** `bun build --compile` (ou Node SEA) para `kds`/`knudge-mcp-ts`;
  publicação npm `@knudge/cli`, `@knudge/core`, `@knudge/mcp`; CI Linux/macOS/Windows.
- **Aceite:** binário roda sem Node instalado; startup ≤ orçamento; testes Windows verdes
  (atomic rename/lock).
- **Ref:** R3/R5/R6.

---

#### TS-038 — Documentação, ADRs e aceitação final

- **Depende de:** TS-036, TS-037
- **Estimativa:** 8
- **Entregável:** `docs/` por subsistema; ADRs TS (numerados a partir do mapeamento);
  matriz de aceitação preenchida; guias de uso equivalentes à `wiki/usage/`.
- **Aceite:** paridade documental; revisão final dos invariantes; release `1.0.0-ts`.
- **Ref:** `wiki/`, `plan/`.

---

**Marco M8:** release `knudge-ts` com conformidade demonstrada.

---

## 10. Grafo de dependências

```
001 ─┬─ 002 ─┬─ 003 ─┐
     │       │       ├─ 005 ─┬─ 006 ─ 007 ─ 008 ─┬─ 009 ─ 010 ─ 011 ─┐
     │       └─ 004 ─┘       │                    └─ 012 ─┐          │
     │                        │                             ├─ 013 ─┬─ 014 ─┐
     │                        │                             │       ├─ 015 ─┤
     │                        │                             └ 016 ─ 017 ─┤
     │                        │                                            ├─ 018 ─ 019 ─┬─ 020 ─ 021
     │                        │                                            │              │
     │                        │                                            │              └─ 022 ─ 023 ─ 024 ─ 025 ─ 026
     │                        │                                            │                               │
     │                        │                                            └────────────── 027 ─ 028 ─ 029 ─ 030
     │                        │                                                                             │
     │                        └──────────────────────────────────────────────────────────── 031 ─┬─ 032
     │                                                                                           ├─ 033
     │                                                                                           └─ 037
     │                                                                                              034 ─┬─ 035
     └─────────────────────────────────────────────────────────────────────────────────────────────────── 036 ─ 038
```

**Caminho crítico:** `001 → 002 → 005 → 006 → 007 → 008 → 009 → 010 → 011 → 012 → 013 → 017 →
018 → 019 → 022 → 024 → 025 → 027 → 028 → 029 → 030 → 031 → 032 → 034 → 036 → 038`.

**Total de pontos:** ~325 (≈ 38 issues). Em time de 2–3 pessoas: ~9–12 meses; solo: ~18–24 meses.

---

## 11. Empacotamento e distribuição

| Canal | Comando | Público |
|---|---|---|
| npm (desenvolvimento) | `npx @knudge/cli ask "..."` | contribuidores, CI |
| Binário standalone | `bun build --compile` → `kds` | usuários finais |
| MCP | `npx @knudge/mcp --stdio` | clientes MCP |
| Lib | `pnpm add @knudge/core` | produtos/servidores |

**Estrutura de pacotes:**

```json
{
  "packages/core": { "name": "@knudge/core", "deps": {} },
  "packages/cli":  { "name": "@knudge/cli",  "bin": { "kds": "dist/main.js" } },
  "packages/mcp":  { "name": "@knudge/mcp",  "bin": { "knudge-mcp-ts": "dist/main.js" } }
}
```

---

## 12. Riscos residuais e decisões abertas

| # | Decisão aberta | Opções | Recomendação |
|---|---|---|---|
| DA-1 | Parser de CLI | `commander`, `clipanion`, hand-rolled | `clipanion` (tipado) com camada de nomes própria |
| DA-2 | Validação de schema | `zod`, `valibot`, own guards | `zod` nas bordas; guards próprios no core |
| DA-3 | Runtime de distribuição | Node SEA, Bun compile, `pkg` | Bun compile (menor startup); fallback Node SEA |
| DA-4 | Async no caminho quente | síncrono, async | síncrono no core/CLI; async só no embeddings |
| DA-5 | Worker threads | usar, não usar | usar só em leitura paralela e blocos O(N²) |
| DA-6 | Localização de mensagens | só PT, i18n | PT (paridade com o Rust) |
| DA-7 | Manter Rust junto | congelar, descontinuar | congelar 0.5.2 como referência; TS como sucessor |
| DA-8 | Texto de `prime`/`help` | copiar literal, reescrever | copiar literal byte a byte |

**Riscos residuais não mitigáveis a curto prazo:** latência de startup em ambientes sem binário
compilado; maturidade de `worker_threads` para determinismo; dependência de um runtime externo
quando não se usa bun/SEA.

---

## 13. Síntese

- **Viável:** o domínio é majoritariamente puro (parsers, scores, grafo, fórmulas) e o contrato de
  dados é JSON/JSONL — território natural de TypeScript.
- **Onde dói:** determinismo, performance CPU-bound, startup e contrato de bytes. Todos têm
  mitigação concreta (portas, workers, binário compilado, parser próprio, conformance diferencial).
- **Maior ativo da migração:** o binário Rust vira **oráculo de conformidade**, permitindo validar
  a reimplementação byte a byte em vez de re-litigar decisões.
- **Rodar as 38 issues em sequência produz um sucessor funcional com paridade demonstrável e
  ecossistema muito mais acessível.**

---

> **Resumo em uma frase:** reimplementar o knudge em TypeScript é factível e vantajoso para
> adoção, mas exige disciplina redobrada de determinismo e um harness de conformidade contra o
> Rust — este documento organiza essa jornada em 8 fases, 38 issues encadeadas e um caminho
> crítico explícito.
