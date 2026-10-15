# EPIC — Reimplementar o `knudge` em TypeScript

> **Base:** `exploration/knudge/knudge.md` (resumo técnico do `REF/knudge` v0.5.4).
> **Objetivo:** descrever **como** reimplementar o knudge — biblioteca de núcleo + CLI `kd` +
> servidor `knudge-mcp` — em **TypeScript/Node**, com vantagens, desvantagens e mitigações, um
> **plano mestre** por fases e uma **cadeia sequencial de issues** executáveis.
>
> **Princípio condutor:** a reimplementação é **byte-compatível no disco**. TOON, ids,
> `body_hash`, o layout de `notas/`, `eventos/` e `.idx/` são **contrato**, não detalhe de
> implementação. Um `knudge-ts` deve ler o que o `kd` Rust escreveu e vice-versa — o que, como
> subproduto, dá ao projeto o *harness* diferencial que ele hoje não tem (o `DIVERGENCES.md` é
> explicitamente o substituto, com 115 bordas catalogadas).

---

## 1. Enquadramento

O knudge é **memória durável por projeto** para agentes de IA: a verdade são notas Markdown com
frontmatter TOON em `.knudge/notas/`, a auditoria é `eventos/events.jsonl`, e o índice `.idx/` é
**derivado e reconstruível** — apagá-lo nunca perde conhecimento.

Superfície: CLI `kd` com **14 verbos** e servidor MCP JSON-RPC 2.0 sobre stdio com 4 tools. O
núcleo é **puro**, com todo o acesso ao mundo mediado por 8 portas (`Clock`, `Rng`, `Env`, `Fs`,
`Git`, `HookRunner`, `Logger`, `Embedder`), e exige **determinismo byte-a-byte**.

### 1.1 O que é *invariante* (não se pode reinterpretar)

| # | Invariante | Consequência para o TS |
|---|---|---|
| 1 | **Contrato de bytes** — TOON canónico na emissão, tolerante na leitura; 31 chaves em ordem fixa; `LF` final; nada de `null`; vazio omitido | Parser/emissor próprios; ordem canónica imposta, não herdada da ordem de inserção |
| 2 | **Identidade por conteúdo** — `id = prefixo_base36(hash(type + U+001F + normalize(statement)))` | Replicar `normalize` (NFC + trim + colapso) e o truncamento de hash **exatamente** |
| 3 | **Verdade × derivado** | `.idx/` sempre descartável; nenhum dado durável só lá |
| 4 | **Idempotência por conteúdo** | Reescrever a mesma afirmação não duplica |
| 5 | **Dedup em duas fases** — `< 0.75` cria, `0.75–0.92` merge, `≥ 0.92` rejeita | Limiares são contrato calibrado, não *tuning* livre |
| 6 | **"Nada sem aceite"** — `learn`/`compact`/`prune` só propõem | Nenhum comando de manutenção escreve |
| 7 | **Contrato de máquina** — stdout = dados, stderr = logs; `--json`; *exit codes* fixos; EPIPE → 0 | Envelope e código de saída replicados um a um |
| 8 | **Determinismo** — mesma entrada, mesma saída byte a byte | Ordem canónica, *tie-breaks* explícitos, sem RNG em decisões |

### 1.2 O que é *idiomático do Rust* (pode ser reinterpretado)

| Rust | Equivalente TS aceitável |
|---|---|
| `BTreeMap` / `IndexMap` | `Map` (ordem de inserção) + ordenação explícita, ou `SortedMap` próprio |
| `#![forbid(unsafe_code)]` | `"strict": true` + `exactOptionalPropertyTypes` + `eslint` restrito |
| `thiserror` no core / `anyhow` na borda | `ErrorKind` + classes de erro tipadas; sem `catch (e: any)` |
| `proptest` | `fast-check` |
| `clippy -D warnings` | `eslint` + `tsc --noEmit` como gate |
| Binário estático | `bun build --compile` ou Node SEA (ver risco D2) |

### 1.3 Números de partida (dimensionamento)

Medidos no `REF/knudge` v0.5.4: **413 ficheiros `.rs`**, **~56 900 linhas** de código
(`knudge-core` 320 ficheiros/39 461 linhas; `knudge-cli` 78/15 814; `knudge-mcp` 15/1 583),
**924 testes** anotados, **201 ADRs** (`D01–D212`) e 23 documentos de especificação em
`wiki/specs/`. O teto de **300 linhas por ficheiro** de produção é um gate de CI e explica o
número alto de ficheiros.

**Leitura para o planeamento:** isto **não** é um port de fim de semana. O alvo realista é um
**MVP paridade-de-bytes em ~15 issues** (fases F0–F8), com o resto (MCP, embeddings, governação,
avaliação) encadeado depois.

---

## 2. Vantagens de reimplementar em TypeScript

| # | Vantagem | Porque importa *neste* sistema |
|---|---|---|
| **V1** | **O ecossistema de agentes é JS/TS-first** | MCP, SDKs de agentes e orquestradores são nativos em TS. O `knudge-mcp` deixa de ser um binário externo a *spawn* e pode ser *in-process* no mesmo runtime do agente. |
| **V2** | **Distribuição por npm** | `npx knudge`, `pnpm dlx`, `npm i -g`. Sem *cross-compilation* por plataforma/arquitetura — o problema que o `install.sh` (18 KB) existe para resolver. |
| **V3** | **Um só idioma na pilha** | CLI + biblioteca + MCP + *tooling* de agente no mesmo runtime. O knudge Rust é deliberadamente multi-crate (`core` publicável + dois pacotes de binário); em TS a mesma separação existe, mas sem fronteira de linguagem. |
| **V4** | **Unicode e JSON nativos** | `String.prototype.normalize('NFC'|'NFD')` e `JSON` são *built-in*; `node:crypto` dá SHA-256. O Rust precisa de `unicode-normalization` + `sha2`. |
| **V5** | **I/O assíncrono trivial** | A fila de embeddings (`indexed|pending|stale`), o *drain* por lotes e o *worker* ocioso passam a ser `async/await` sem `tokio`. |
| **V6** | **Iteração rápida sobre os algoritmos** | Limiares de dedup, pesos de RRF, PPR, Louvain, FSRS e Beta/Wilson são *tuning* experimental. Testes rápidos (`vitest`) + *types* encurtam o ciclo de calibração. |
| **V7** | **Portabilidade de runtime** | Node, Bun, Deno; CI e *serverless* sem configuração. Relevante para o *drain* agendado (hoje via `systemd --user`/`launchd`/cron). |
| **V8** | ***Harness* diferencial barato** | Como o formato é o mesmo, comparar byte a byte com o `kd` Rust vira uma suíte de conformidade. Isto **fecha a lacuna** que o `DIVERGENCES.md` só mitiga com testes locais. |
| **V9** | **Biblioteca para o agente, não só binário** | `knudge-core` é `publish = true` (D214). Em TS, importar o núcleo num agente é uma linha — sem *FFI* nem processo. |

---

## 3. Desvantagens e riscos (com mitigação)

| # | Risco | Gravidade | Mitigação |
|---|---|---|---|
| **R1** | **Latência de arranque** — o `kd` Rust tem piso na casa dos poucos ms; o Node arranca em dezenas. A CLI é invocada **muitas vezes** por sessão de agente. | **Alta** | Compilar para executável único (`bun build --compile` / Node SEA); *lazy imports* agressivos; garantir que `prime`/`help`/`version` **não abrem sessão** (paridade com a separação estático/dinâmico `prime`×`rewind`, D57). Medir contra `bench/` desde a F0. |
| **R2** | **`normalize()` depende da versão de ICU** — pode divergir do `unicode-normalization` do Rust em casos-limite. | **Alta** | Fixar `engines.node`; *goldens* de Unicode (NFC/NFD, escalares, espaços) gerados pelo Rust; documentar a versão de Unicode suportada. É a borda mais perigosa de todo o port — trata-se na **I03**, antes de qualquer lógica. |
| **R3** | **Ordem de propriedades em objetos** — chaves *array-index* vêm primeiro; `Object.keys` não é ordem de inserção. | **Alta** | Proibir objetos como mapas ordenados no `core`; usar `Map`/arrays; regra de *lint* dedicada que falha em `Record<string, T>` no caminho crítico. |
| **R4** | **Ponto flutuante sem `total_cmp`** — o Rust usa ordem total IEEE-754 para *tie-breaks* determinísticos. | Média | Implementar `totalCmp` (replicando o *bit pattern* do `total_cmp`), tratar `-0`/`NaN`, e usá-lo em **todo** o `sort`. Testar com `fast-check`. |
| **R5** | **Ausência de `BTreeMap`** — o Rust proíbe `HashMap` no core para garantir iteração determinística. | Média | `SortedMap` próprio sobre `Map` + ordenação explícita; testes que comparam a ordem de iteração com o Rust. |
| **R6** | **Concorrência não ordenada** — o *event loop* não garante ordem se o I/O for paralelizado. | Média | Recolher resultados **por índice canónico**; nunca depender da ordem de conclusão de `Promise.all`. Regra de *lint*: `Promise.all` proibido em módulos que produzem saída ordenada. |
| **R7** | **Atomicidade e `fsync` de diretório** — `rename` + `fsync` não é uniforme (Windows). | Média | Porta `Fs` com *fallbacks* documentados; testes nos 3 SO; reproduzir as bordas de atomicidade do `DIVERGENCES.md` como casos de teste. |
| **R8** | **Lock advisory** — `open('wx')` cobre `O_CREAT|O_EXCL`, mas o *reclaim* por `mtime` difere por sistema de ficheiros. | Média | Porta `Fs` injetável; testes de stress com FS real (paridade com `stress.rs`); stale de 30 s e ordem externo=container/interno=nota replicados. |
| **R9** | **Desempenho de BM25/PageRank/Louvain** em corpora grandes. | Média | `Float64Array`/`Uint32Array`, índices invertidos em arrays planos; MinHash/LSH (o Rust mediu −94 % no gargalo quadrático); `worker_threads` no *drain*. |
| **R10** | **Divergência de mensagens e *exit codes*** — muitos *frameworks* de CLI impõem o seu próprio *help* e os seus erros. | Média | **Parser de argumentos próprio** (ou camada de tradução integral); *goldens* do `help`, do envelope `--json` e de cada erro. |
| **R11** | **Fragilidade de `Intl`/locale** — `Intl.Collator` depende de ICU e do *build* do runtime. | Média | Proibir `Intl.Collator`/`localeCompare` no `core`; comparação por *code units*. A sugestão Levenshtein (D212) é implementada por nós, sem locale. |
| **R12** | **Semântica de `fsync` e durabilidade percebida** | Baixa | A ordem **nota antes do evento** (D21) é a garantia real: um crash deixa nota sem evento, estado recuperável. Testar com `FaultyFs` injetável. |
| **R13** | **Segurança da *supply chain*** — o Rust tem `deny`/`audit`/`machete` e *pinning* de GGUF com SHA-256. | Média | Minimizar dependências (o knudge tem 8 de produção); `pnpm` com lockfile e `--frozen-lockfile`; auditoria no CI; *pinning* de modelo por revisão + SHA-256 preservado. |
| **R14** | **Tentação de "melhorar" o formato** — a compatibilidade byte-a-byte é restritiva. | Média | Congelar o formato na F1; qualquer extensão entra como **chave aditiva** (como o schema v2 fez com `claims`/`provenance`), nunca como alteração. |

**Conclusão:** o port é **viável e vantajoso na integração** (V1/V2/V9), mas **paga em arranque e
determinismo** (R1/R2/R3). O plano mestre ataca esses três eixos **primeiro**, antes de qualquer
lógica de domínio — se o determinismo não se provar na F1, o resto do épico não vale a pena.

---

## 4. Arquitetura-alvo

```
knudge-ts/                          monorepo (pnpm workspaces)
├─ packages/
│  ├─ core/                         # núcleo puro — NÃO importa node:*
│  │  ├─ error.ts                   # ErrorKind + code()/exitCode()
│  │  ├─ time.ts                    # Timestamp UTC (ms)
│  │  ├─ ports/                     # 8 interfaces + fakes
│  │  ├─ schema/                    # enums fechados, 31 chaves, ids, bodyHash, normalize
│  │  ├─ toon/                      # parser + emissor byte-exatos
│  │  ├─ jsonl/                     # JSONL + JSON canónico
│  │  ├─ config/                    # subset TOML próprio + resolução global×projeto
│  │  ├─ store/                     # notas/, eventos/, lock, rebuild, purge, sweep
│  │  ├─ graph/                     # arestas, ciclos, PPR, Louvain, ontologia, TMS
│  │  ├─ retrieval/                 # tokenização, BM25, âncoras, RRF, views, temporal
│  │  ├─ write/                     # protocolo, dedup 2 fases, MinHash/LSH, supersede
│  │  ├─ handoff/                   # prime/rewind, orçamento, context_id
│  │  ├─ task/                      # hierarquia, derivados, flow (PERT/CPM)
│  │  ├─ lifecycle/                 # FSRS, Beta/Wilson, decay/drift, clusters
│  │  ├─ health/                    # validators, doctor, auditoria, evidência
│  │  ├─ maintenance/               # diff, learn, compact, prune (só propostas)
│  │  └─ embeddings/                # provedor plugável, cache LRU, fila
│  ├─ adapters-node/                # Fs, Git, Logger, Env, Clock, Rng, HookRunner, Embedder
│  ├─ cli/                          # binário `kd`: parser próprio + envelope + exit codes
│  └─ mcp/                          # servidor JSON-RPC stdio (4 tools, HintEngine puro)
├─ test/
│  ├─ golden/                       # bytes de TOON, prime, help, --json, erros, EPIPE
│  ├─ property/                     # fast-check (substituto do proptest)
│  ├─ stress/                       # lock sem lost update, leitor × rebuild, FaultyFs
│  └─ differential/                 # compara byte-a-byte com o `kd` Rust
└─ bench/                           # micro + e2e, paridade com o bench/ do Rust
```

### 4.1 Regras de pureza (espelham D65 e o `AGENTS.md`)

1. **`core/` nunca importa `node:fs`, `node:process`, `node:crypto`, `node:child_process`.**
   Verificado por *lint*, não por disciplina — a regra deve **falhar de propósito** quando violada.
2. **`Date.now()`, `Math.random()`, `process.env`, `process.argv` proibidos no `core`.** Só via
   `Clock`, `Rng`, `Env`.
3. **`Record<string, T>` proibido como mapa ordenado** no `core` (usar `Map`/`SortedMap`).
4. **`Intl` proibido** no `core`.
5. **`any`/`as`/`@ts-ignore` proibidos.** `tsconfig` com `strict`,
   `exactOptionalPropertyTypes`, `noUncheckedIndexedAccess`.
6. **≤ 300 linhas por ficheiro de produção**, gate no CI.
7. **Toda decisão de algoritmo carrega o `Dxx` de origem num comentário** — o port é uma
   *tradução anotada*, não uma reescrita livre.

### 4.2 Mapa de rastreabilidade

Cada módulo TS aponta para a sua especificação Rust. Isto é o que torna o port auditável:

| Módulo TS | Especificação Rust |
|---|---|
| `toon/` | `wiki/specs/TOON.md` (D04/D05/D06/D09/D11/D12/D13/D74/D75) |
| `schema/` | `wiki/specs/modelo-de-dados.md` (D01/D95/D207/D191) |
| `store/` | `wiki/specs/persistencia.md` (D20–D27, D150) |
| `retrieval/` | `wiki/specs/busca.md` + `matematica.md` (D35–D42, D172/D173/D206) |
| `write/` | `wiki/specs/escrita-e-dedup.md` (D26/D80/D204) |
| `graph/` | `wiki/specs/grafo.md` (D45/D49–D51/D192/D193/D207/D208) |
| `task/` | `wiki/specs/tarefas.md` (D93/D113/D144/D205) |
| `lifecycle/` | `wiki/specs/ciclo-de-vida.md` (D43/D154/D189/D190/D203) |
| `handoff/` | `wiki/specs/handoff.md` (D40/D57/D82/D88) |
| `health/`, `maintenance/` | `wiki/specs/saude.md`, `manutencao.md` (D47/D156/D163) |
| `embeddings/` | `wiki/specs/embeddings.md` (D79–D85/D101/D148/D183) |
| `cli/` | `plan/implementation/16_cli_surface.md` (D209–D212) |
| `mcp/` | `wiki/specs/mcp.md` (D68) |
| tudo | `wiki/specs/determinismo.md`, `erros.md`, `DIVERGENCES.md` |

---

## 5. Plano mestre

### 5.1 Fases

| Fase | Tema | Paridade | Issues | Gate de saída |
|---|---|---|---|---|
| **F0** | Fundação: monorepo, portas, erros, determinismo | E01 | I01 | `pnpm check` verde; nenhum `node:*` no `core`; arranque medido |
| **F1** | Contrato de bytes: TOON, schema, ids/hashes | E02 | I02, I03 | **Diferencial verde** em ids/hashes/TOON |
| **F2** | Store: notas, eventos, lock, rebuild | E03 | I04, I05 | Crash-injection passa; leitor nunca vê índice parcial |
| **F3** | Retrieval: tokenização, BM25, âncoras, RRF, views | E06 | I06, I07 | Ranking idêntico ao Rust numa fixture fixa |
| **F4** | Escrita: protocolo, dedup 2 fases, MinHash/LSH | E07 | I08 | Decisões cria/merge/rejeita idênticas |
| **F5** | Grafo: arestas, ciclos, PPR, Louvain, ontologia, TMS | E05 | I09 | PPR/Louvain determinísticos; ordem estável |
| **F6** | Tarefas: hierarquia derivada e flow | E08 | I10 | Derivados idênticos; PERT/CPM confere |
| **F7** | Ciclo de vida: FSRS, Beta/Wilson, drift | E09 | I11 | Confiança e shelf-life idênticos |
| **F8** | Superfície CLI: 14 verbos, help, `--json`, exit | E12/E15 | I12 | **Paridade de CLI**: goldens de bytes e de exit |
| **F9** | MCP: JSON-RPC stdio, 4 tools, `HintEngine` | E12/E14 | I13 | Handshake + framing idênticos |
| **F10** | Embeddings, `drain` e worker | E11 | I14 | Fila degrada sem provedor; cache LRU |
| **F11** | `doctor`, `maintenance`, governação, `sync`/`config`/`forget` | E09/E10/E04 | I15 | 13 checks + `--fix` idempotente |
| **F12** | Conformidade, *bench*, empacotamento | E13/E16 | I16, I17 | Suíte diferencial completa; binário único |

### 5.2 Gates transversais (aplicam-se a *todas* as issues)

| Gate | Critério |
|---|---|
| **G-type** | `tsc --noEmit` sem erro |
| **G-lint** | `eslint` sem aviso, com as regras de pureza ativas |
| **G-test** | Testes unitários + *property* verdes |
| **G-size** | ≤ 300 linhas por ficheiro de produção |
| **G-diff** | Quando a issue toca bytes: **diferencial contra o Rust verde** |
| **G-doc** | O `Dxx` de origem citado no código e na nota da issue |
| **G-perf** | Nenhuma regressão face ao *baseline* da fase anterior |

### 5.3 Estratégia de verificação (herdada e reforçada)

| Camada | O que trava | Como |
|---|---|---|
| **Golden** | Bytes de TOON, `prime`, `help`, `--json`, erros, EPIPE | Ficheiros de referência gerados pelo Rust, comparados byte a byte |
| **Property** | Round-trip TOON, RRF, decay/confiança, ids sob normalização | `fast-check` |
| **Stress** | Lock sem *lost update*, escritas concorrentes, leitor × rebuild | FS real, processos paralelos |
| **Crash-injection** | Nota-antes-de-evento, `tmp+rename`, rebuild *double-buffer* | `FaultyFs` (porta `Fs` que injeta falhas) |
| **Diferencial** | **Toda** a paridade de formato e de ranking | *Corpus* sintético + o próprio corpus do projeto, executando `kd` (Rust) e `kd-ts` e comparando |
| **Bordas** | As 115 entradas do `DIVERGENCES.md` | Cada uma vira um caso de teste nomeado — o port **não inventa** bordas novas |
| **Bench** | Latência de arranque, throughput de BM25, *drain* | `bench/` com *baselines* congelados |

---

## 6. Cadeia sequencial de issues

> **Encadeamento:** cada issue **só começa** quando a anterior está *Done*. O campo **dep** é
> bloqueante. *Definition of Done* comum: gates G-type/G-lint/G-test/G-size/G-doc verdes, mais
> G-diff quando aplicável.
>
> **Caminho crítico:** I01 → I02 → I03 → I04 → I06 → I08 → I12 é o **MVP** (um `kd` que
> cria, busca e escreve notas byte-compatíveis). Tudo o resto é encadeado depois.

### I01 — Fundação: monorepo, portas e modelo de erro
**dep:** — · **Fase:** F0

- **Objetivo:** esqueleto executável com os invariantes de engenharia **antes** de qualquer lógica.
- **Entregáveis:** *workspace* `pnpm`; `tsconfig` estrito; `eslint` com as 7 regras de pureza;
  `vitest` + `fast-check`; as **8 portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`,
  `Logger`, `Embedder`); `ErrorKind` com `code()` e `exitCode()` (`2,3,4,5,6,7,8,9,70`;
  `0` sucesso, `101` reservado a panic); utilitários de determinismo (`totalCmp`, `SortedMap`,
  `clamp01`).
- **Tarefas:** T01 *workspace* e scripts; T02 `ports/` com fakes (`FixedClock`, `SeqRng`, `MemFs`,
  `FaultyFs`, `FakeGit`, `NoopHookRunner`, `RecordingLogger`, `FakeEmbedder`); T03 `error.ts` com
  `retryable` só para `Timeout`; T04 gate de 300 linhas no CI; T05 regras de *lint* de pureza
  (provadas a falhar de propósito); T06 **medição de arranque** de um `kd-ts --version` mínimo.
- **Aceite:** os *exit codes* batem com `wiki/specs/erros.md`; nenhum import de `node:*` em
  `core/`; arranque medido e registado como *baseline* da R1.

### I02 — TOON: parser e emissor byte-exatos
**dep:** I01 · **Fase:** F1

- **Objetivo:** replicar o contrato de bytes (`wiki/specs/TOON.md`).
- **Entregáveis:** `toon/parse.ts` e `toon/emit.ts`; emissão em **ordem canónica** das 31 chaves;
  escalares (`Bool`, `Int`, `Float`, `Str` cru ou com aspas); *flow* listas/mapas; blocos
  indentados; comentários; escapes `\n\r\t\0\u{}`; aninhamento ≤ 32; tab é erro; chave duplicada
  é erro; sem `null`; lista/mapa vazio omitido; documento vazio → 0 bytes; `LF` final.
- **Tarefas:** T01 gramática; T02 regra de aspas (cru só quando seguro); T03 regra de inteiros
  (`1.0` → `1`); T04 tolerância na leitura (Postel); T05 `fast-check` de round-trip byte-exato.
- **Aceite:** round-trip idêntico num *corpus* de fixtures geradas pelo Rust; `emit(parse(x)) === x`
  para todo o *corpus*.

### I03 — Schema, ids endereçados por conteúdo e `body_hash`
**dep:** I02 · **Fase:** F1

- **Objetivo:** identidade determinística **igual** à do Rust — a issue de maior risco do épico.
- **Entregáveis:** `schema/` com os enums fechados (`NoteType` 10, `Scope` 3, `Classification` 3,
  `Status` 6, `EdgeKind` 12) e as chaves (`CANONICAL_KEYS` 31, `REQUIRED_KEYS` 5); `normalize()`
  (NFC + trim + colapso de espaços); `noteId()` (`base36(hash(type + U+001F + normalize(statement)))`);
  `bodyHash()` (SHA-256, `hex8`); `shortHash()` (4 primeiros bytes, big-endian); validação com
  sugestão Levenshtein; extensões aditivas `claims`/`provenance` e *slots* mínimos por espécie.
- **Tarefas:** T01 enums e conjuntos fechados; T02 `node:crypto` **só no adaptador**, interface no
  `core`; T03 `normalize` com ICU fixado e *goldens* de Unicode; T04 milhares de pares
  `(entrada → id/hash)` extraídos do Rust; T05 verificar que **reclassificar `type` não muda o id**.
- **Aceite:** **G-diff verde** para ids e `body_hash` em todo o *corpus*, incluindo NFC/NFD,
  espaços múltiplos, acentos e *emoji*; sugestão Levenshtein idêntica para os valores inválidos do
  Rust.
- **Risco:** R2. Se este gate falhar, **parar** e resolver antes de prosseguir — todo o resto
  depende da identidade.

### I04 — Store: persistência, escrita atómica e eventos
**dep:** I03 · **Fase:** F2

- **Objetivo:** `notas/<tipo>/<id>.md`, `eventos/events.jsonl`, *layout* material por tipo.
- **Entregáveis:** `store/notes.ts`, `store/events.ts`, `store/commit.ts`; escrita `tmp + rename`
  no mesmo diretório; `fsync` em lote; **nota antes do evento**; rotação `events-NNNN.jsonl` por
  `max_bytes`; `id` de evento derivado do conteúdo; dedup *on-read*; `purge_derived` e
  `sweep_residues` (remove `*.tmp`/`*.stale` velhos, **nunca** `*.lock`).
- **Tarefas:** T01 porta `Fs` em Node com `rename`/`fsync`/`mtime`; T02 commit e ordem; T03
  rotação e *append*; T04 *sweep* de resíduos com `warn`; T05 `notas/MAP.md` e notas-hub
  materializadas.
- **Aceite:** crash entre nota e evento (via `FaultyFs`) deixa **nota sem evento**; ficheiro
  meio-escrito nunca é visível; *layout* por tipo idêntico ao do Rust.

### I05 — Locks, rebuild *double-buffer* e resiliência
**dep:** I04 · **Fase:** F2

- **Objetivo:** concorrência correta e índice que nunca aparece parcial.
- **Entregáveis:** `store/lock.ts` (lock advisory por ficheiro-alvo, stale 30 s, *reclaim* por
  `rename`, ordem externo=container/interno=nota para evitar *deadlock* ABBA, libertação RAII);
  `store/rebuild.ts` (`.idx.new/` + `rename` atómico); `.idx/events.checkpoint`.
- **Tarefas:** T01 lock e testes de stress; T02 rebuild *double-buffer*; T03 teste "o leitor vê o
  índice antigo ou o novo, nunca um parcial"; T04 recuperação a partir de `mtime` quando o
  conteúdo não é legível.
- **Aceite:** o lock impede *lost update* em FS real com N processos; crash a meio do rebuild
  preserva o índice anterior; *deadlock* ABBA impossível por construção da ordem.

### I06 — Retrieval lexical: tokenização, BM25 e views
**dep:** I05 · **Fase:** F3

- **Objetivo:** o canal lexical determinístico.
- **Entregáveis:** `retrieval/tokenize.ts` (ASCII → *fold* de diacríticos por termo inteiro →
  *stopwords* PT+EN e fragmentos de 1 caractere → *stemming* PT conservador com radical ≥ 4 bytes);
  `retrieval/bm25.ts` (`k1=1.5`, `b=0.75`, IDF por campo, pesos `statement 3.0`/`tags 2.0`/
  `body 1.0`, `type_weight` de 0.70 a 1.20, boost `×(1+0.1·conf)`, corte de alta frequência quando
  `df/N ≥ 0.9` **e** `N ≥ 64`); `retrieval/views.ts` (`ready`/`blocked` por `depends_on`
  transitivo, ciclo ⇒ `blocked`); índice em `.idx/retrieval.jsonl` reconstruível.
- **Tarefas:** T01 tokenizador + *goldens*; T02 BM25 com `Float64Array`; T03 views estáticas
  byte-idênticas e dinâmicas com `now_ms`; T04 reconstrução do índice a partir de `notas/`.
- **Aceite:** para uma *query* fixa sobre uma fixture fixa, a **ordem e os scores** batem com o
  Rust; `.idx/retrieval.jsonl` é byte-idêntico.

### I07 — Fusão RRF, âncoras e canais de busca
**dep:** I06 · **Fase:** F3

- **Objetivo:** a cascata completa, com degradação graciosa.
- **Entregáveis:** `retrieval/rrf.ts` (`Σ peso_c/(k + r_c + 1)`, `k=60`; pesos `lexical 1.0`,
  `anchor 2.0`, `semantic 30.0`, `ppr 0.0`; desempate `(score desc, id asc)`);
  `retrieval/anchors.ts` (globs `?`/`*`/`**`, correspondência bidirecional, `ask --anchor` sem
  *query*); canal **temporal** `--as-of <TS>` (reconstrói o conjunto ativo a partir do log de
  eventos e corre o pipeline sobre esse subconjunto); **revelação progressiva** (1º hit com corpo,
  2º–5º truncado a 280 caracteres, resto só a linha; `--brief` desliga); campo `why` com
  vocabulário fechado (`file_match > anchor_match > tracker_match > stars > semantic > recent >
  universal`); envelope `channels` no `--json`.
- **Tarefas:** T01 RRF e desempate; T02 âncoras; T03 temporal; T04 revelação progressiva; T05
  degradação: canal ausente → resultado parcial + `warnings` (nunca erro).
- **Aceite:** ranking idêntico ao Rust com todos os canais ligados e com canais desligados;
  `--as-of` reproduz o estado histórico; `why` nunca inventa valor.

### I08 — Escrita: protocolo idempotente e dedup em duas fases
**dep:** I07 · **Fase:** F4

- **Objetivo:** o protocolo de escrita com dedup calibrado.
- **Entregáveis:** `write/propose.ts` (recall lexical de candidatos + score Dice); `write/decide.ts`
  (`< 0.75` cria, `0.75–0.92` *merge*, `≥ 0.92` rejeita); `write/merge.ts` (união de
  tags/âncoras/outcomes, `revision++`, corpo nunca sobrescrito); `write/update.ts` (mesma chave →
  edita no lugar; chave nova → novo `id` + `replaces`/`superseded_by` bidirecionais);
  `write/blocking.ts` (*peneira* exata de *postings* quando pequeno/esparso; **MinHash 64
  permutações + LSH 16 bandas × 4 linhas** acima de 256 notas densas, com verificação Dice exata
  dos candidatos); ciclo de vida *soft* (`forget`/`restore` só mudam `status`); rejeição de
  `--type task`/`epic`.
- **Tarefas:** T01 score Dice e limiares; T02 peneira exata; T03 MinHash/LSH com propriedade
  "proposta idêntica ou superconjunto"; T04 *merge* e `revision`; T05 *update*/*supersede*;
  T06 rejeição de tarefas com mensagem e *exit* corretos.
- **Aceite:** para um *corpus* de pares, a decisão **cria/merge/rejeita** é idêntica à do Rust,
  com e sem MinHash/LSH; `propose_merges` só propõe.

### I09 — Grafo: integridade, autoridade, comunidades, ontologia e TMS
**dep:** I08 · **Fase:** F5

- **Objetivo:** o grafo explícito e as inferências sobre ele.
- **Entregáveis:** `graph/edges.ts` (12 arestas de frontmatter; `link` sem duplicar, rejeita id
  inválido e auto-aresta); `graph/integrity.ts` (referências quebradas, órfãos, bidirecionalidade);
  `graph/cycles.ts` (SCC de Kosaraju **iterativo** sobre `replaces`; membros não são demovidos);
  `graph/pagerank.ts` (iteração de potência, *damping* 0.85, ≤ 32 iterações, tolerância L1 `1e-8`,
  *dangling* redistribuído); `graph/communities.ts` (Louvain determinístico, ≤ 8 níveis e ≤ 64
  passos, empate mantém a comunidade); `graph/ontology.ts` (classes de equivalência `same_as`,
  clausura `broader`/`narrower`, ciclos de hierarquia, `claim_conflicts`); `graph/tms.ts`
  (retratados por premissas `forgotten`/`superseded`, derrotados por `replaces`); sugestões
  textuais **só** em `.idx/suggestions.jsonl`.
- **Tarefas:** T01 arestas; T02 integridade e ciclos; T03 PPR com *tie-break* por `totalCmp`;
  T04 Louvain com ordem de visita determinística; T05 ontologia e conflitos de *claims*; T06 TMS
  sem apagar nada.
- **Aceite:** PPR e Louvain produzem **a mesma ordem** que o Rust na fixture; nenhuma sugestão
  vira aresta; o TMS só *sinaliza*.

### I10 — Tarefas: hierarquia derivada e fluxo
**dep:** I09 · **Fase:** F6

- **Objetivo:** trabalho como *view* derivada, nunca como estrutura armazenada.
- **Entregáveis:** `task/hierarchy.ts` (`epic ⊃ { issue ⊃ task | task }`, profundidade ≤ 4, pai
  por rank estritamente menor); `task/markers.ts` (marcador durável
  `<!-- knudge:parent <id> blocks <n> -->` + aresta `results_in` derivada); `task/derived.ts`
  (papel, modo, progresso por folhas fechadas, `epic_of`, impacto por dependentes transitivos
  abertos); `task/flow.ts` (cycle/lead/throughput + **caminho crítico PERT/CPM**); fecho por
  evidência (`--outcome success|partial|failure|abandoned`); escopo obrigatório com *exit* 2.
- **Tarefas:** T01 hierarquia e marcadores; T02 derivados; T03 impacto transitivo; T04 PERT/CPM;
  T05 escopo obrigatório e mensagens.
- **Aceite:** todos os derivados idênticos ao Rust; `blocks` 1-based; `task close` sem evidência
  falha.

### I11 — Ciclo de vida: FSRS, confiança e drift
**dep:** I10 · **Fase:** F7

- **Objetivo:** shelf-life, confiança derivada e decaimento.
- **Entregáveis:** `lifecycle/shelf.ts` (`foundational` nunca expira; `tactical` 365 dias;
  `observational` 30); `lifecycle/fsrs.ts` (cada sucesso estende o prazo em 50 % e repõe o
  relógio; `retention(t) = limiar^(t/ttl)`, `limiar = 0.5`); `lifecycle/confidence.ts`
  (Beta-Bernoulli com `success=1`, `partial=0.5`, `failure/abandoned`=1 falha; posterior
  `Beta(1+s, 1+f)`; **limite inferior de Wilson a 95 %, Z=1.96**; `confidence_score = base +
  lower_bound + feedback(0.2) + tarefa + recência(≤0.05)`, `clamp01`, `× drift_factor`);
  `lifecycle/usage.ts` (uso derivado em `.idx/usage.jsonl`, **só estende**);
  `lifecycle/drift.ts` (drift de âncoras `= 1 − fração válida`, `drift_factor = 1 − 0.5·drift ∈
  (0.5, 1]`, persistido em `.idx/drift.jsonl`; drift de termos por **JS base 2** entre metades
  antiga e nova, propõe revisão quando `JS ≥ 0.5` com ≥ 4 notas); `lifecycle/clusters.ts`
  (fase 1 estrutural; fase 2 semântica *complete-link* *off-path*); demolição **só propõe**
  (`Expired`/`AnchorDecay`/`Contradicted`/`Defeated`/`Drifted`), ciclos protegidos.
- **Tarefas:** T01 shelf-life; T02 FSRS; T03 Beta/Wilson com *goldens* numéricos; T04 uso;
  T05 drift de âncoras e de termos; T06 clusters; T07 propostas de demolição.
- **Aceite:** `confidence_score` idêntico ao Rust dentro de uma tolerância de bit para as fixtures
  numéricas; nada é removido antes da janela de retenção.

### I12 — Superfície CLI (`kd`) e contrato de máquina
**dep:** I11 · **Fase:** F8

- **Objetivo:** os **14 verbos** com paridade de bytes e de *exit code*.
- **Entregáveis:** `cli/parse.ts` (**parser próprio**, com listas por repetição ou vírgula, texto
  livre nunca dividido, `--params '<json>'` e `-` lendo do *stdin*); `cli/envelope.ts`
  (`{success, command, data?, error?, warnings?}`); `cli/output.ts` (stdout = dados, stderr =
  logs, EPIPE → *exit* 0); `cli/help.ts` (`kd` sem argumentos = `kd help`, *exit* 0); os verbos
  `init prime rewind ask write task map maintenance doctor drain config forget sync self`;
  `prime` **byte-idêntico por versão** (e `--long` com a gramática TOON e o schema); `rewind` com
  os modos Manifest/Scope/Files/Auto, `next:`/`fresh:`, orçamento `ceil(len/4)` com
  `DEFAULT_BUDGET=4000`, e `--resume <context_id>` devolvendo bytes idênticos; `kd self` com
  `completions` e `setup` para os agentes-alvo.
- **Tarefas:** T01 parser; T02 envelope e *exit*; T03 EPIPE e `(no_results]`; T04 `prime`;
  T05 `rewind` e `context_id`; T06 `ask`/`write`/`task`; T07 `map`/`doctor`/`drain`/`maintenance`/
  `config`/`forget`/`sync`; T08 `self` e *completions*.
- **Aceite:** **G-diff verde** para `prime`, `help`, `--json` de cada verbo, mensagens de erro e
  *exit codes*; `rewind --resume` devolve bytes idênticos.
- **Nota:** é o fim do **MVP**. A partir daqui o `knudge-ts` é utilizável como substituto.

### I13 — Servidor MCP (`knudge-mcp`)
**dep:** I12 · **Fase:** F9

- **Objetivo:** o servidor MCP local e reativo.
- **Entregáveis:** `mcp/server.ts` (JSON-RPC 2.0 sobre stdio, **uma linha JSON por mensagem**, sem
  `Content-Length`; `PROTOCOL_VERSION = "2025-06-18"` com lista de versões suportadas;
  `initialize`/`ping`/`tools/list`/`tools/call`; EOF e EPIPE → *exit* 0);
  `mcp/hint-engine.ts` (**puro**) com os 4 gatilhos: `knudge_pre_write` (quase-duplicados),
  `knudge_pre_edit` (working set antes de editar um ficheiro), `knudge_session_end` (`learn`) e
  `knudge_status`; *hints* como **ponteiros** (`id + statement + score`), com *cap* e modo de
  observação; *dedup* por sessão.
- **Tarefas:** T01 *framing* e handshake; T02 as 4 tools; T03 `HintEngine` puro com testes de
  tabela; T04 *cap*, dedup e modo observação; T05 tolerância a mensagem malformada.
- **Aceite:** um cliente MCP real faz o handshake e recebe *hints* ponteiro; nunca é enviado
  corpo de nota.

### I14 — Embeddings, `drain` e worker
**dep:** I13 · **Fase:** F10

- **Objetivo:** o canal vetorial opcional, com degradação graciosa.
- **Entregáveis:** `embeddings/provider.ts` (porta `Embedder`; provedores `http`
  (OpenAI-compatible, default `http://127.0.0.1:8889/v1/embeddings`), `lightweight` (hash
  determinístico para CI) e `none`); `embeddings/http.ts` (cliente HTTP/1.1 com *timeout* e
  *retry* idempotente — **sem inferência in-process**); `embeddings/index.ts`
  (`.idx/embeddings.jsonl` com cabeçalho `meta`; mudar modelo **invalida** e força re-*embed*);
  `embeddings/cache.ts` (por `body_hash` + modelo, LRU com teto, *pass-through* na falha);
  `embeddings/queue.ts` (estado derivado `indexed|pending|stale`; falha marca `pending`, nunca
  descarta; modo `lazy` drena 1 lote *best-effort* após cada verbo); `embeddings/suggest.ts`
  (classifica pares em `duplicate`/`contradiction`/`link`, *advisory*); `drain` e o *worker*
  ocioso.
- **Tarefas:** T01 porta e `lightweight`; T02 cliente HTTP; T03 índice e invalidação por modelo;
  T04 cache LRU; T05 fila e dreno por lotes; T06 `suggest`; T07 *worker*.
- **Aceite:** sem provedor, o sistema funciona (cai para BM25) com `warnings`; o estado
  `indexed|pending|stale` é derivado corretamente; nenhum *embedding* é descartado por falha.

### I15 — `doctor`, `maintenance` e governação
**dep:** I14 · **Fase:** F11

- **Objetivo:** saúde, manutenção e configuração.
- **Entregáveis:** `health/doctor.ts` (13 *checks* + auditoria; `healthy` só com **zero achados**;
  `--fix` corrige o reversível — `body_hash`, âncoras quebradas, locks *stale*, duplicatas,
  *layout* — e é **idempotente**; `--explain` com esperado/encontrado/ação);
  `health/validators.ts` (`.knudge/validators.toml` com `cmd`/`scope`/`severity`/`timeout`);
  `maintenance/learn.ts`, `compact.ts`, `prune.ts` (**read-only**, só propõem) e `diff.ts`
  (lê a auditoria de eventos, nunca o git global); portão de evidência com `gate=passed|failed` e
  `proposals.enforce`; `config/` (subset TOML próprio, resolução global × projeto com precedência,
  segredos só no global) e `config promote` (regras governadas no bloco `<!-- knudge:rules -->`
  do `AGENTS.md`, com teto, desligado por default); `git/` (`info/exclude`, `AGENTS.md`
  idempotente, `sync` com `merge=union`); redação tipada de segredos no log
  (`«REDACTED:tipo»`); hooks (`pre-record`, `post-record`, `pre-prune`, `pre-compact`, com
  *timeout* e *kill* do grupo, sem *shell*).
- **Tarefas:** T01 *checks* e `--fix`; T02 validators; T03 `learn`/`compact`/`prune`/`diff`;
  T04 config e `promote`; T05 git, `sync`, `AGENTS.md`; T06 redação de segredos; T07 hooks.
- **Aceite:** `doctor --fix` executado duas vezes não altera nada na segunda; nenhum comando de
  manutenção escreve; `promote` nunca auto-edita.

### I16 — Conformidade diferencial e *bench*
**dep:** I15 · **Fase:** F12

- **Objetivo:** provar a paridade de forma sistemática e medir.
- **Entregáveis:** suíte `test/differential/` que executa o `kd` Rust e o `kd-ts` sobre o mesmo
  *corpus* e compara bytes de `notas/`, `eventos/`, `--json` e `exit code`; *runner* que reproduz
  as **115 bordas** do `DIVERGENCES.md` como casos nomeados; `bench/` com micro e e2e, incluindo
  latência de arranque, throughput de BM25 e dreno; métricas de qualidade de retrieval
  (Recall@k, MRR, nDCG@k).
- **Tarefas:** T01 *harness* de *corpus* sintético; T02 comparação de bytes; T03 as 115 bordas;
  T04 *bench* e *baselines*; T05 relatório de divergências (e proposta de correção *upstream*).
- **Aceite:** zero divergência não explicada; cada divergência encontrada é ou corrigida ou
  registada com justificação — este é o entregável científico do épico.

### I17 — Empacotamento, distribuição e desempenho
**dep:** I16 · **Fase:** F12

- **Objetivo:** tornar o `knudge-ts` instalável e rápido o suficiente.
- **Entregáveis:** executável único (`bun build --compile` e/ou Node SEA) para Linux/macOS/Windows;
  publicação npm (`knudge-ts` + `@knudge/core`); `engines` fixo e verificação de ICU no arranque;
  *lazy imports* auditados; *worker* de *drain* empacotado; `install.sh` equivalente; *pipelines*
  de CI (type + lint + test + size + diferencial + *bench*).
- **Tarefas:** T01 executável único; T02 pacotes npm e *lockfile*; T03 CI completo; T04
  otimização de arranque até paridade aceitável; T05 documentação de instalação e migração.
- **Aceite:** `npx knudge --version` e o binário compilado funcionam nos 3 SO; latência de
  arranque dentro do orçamento definido na I01; a suíte diferencial corre no CI.

---

## 7. Critérios de sucesso do épico

| # | Critério | Medida |
|---|---|---|
| C1 | **Compatibilidade de disco** | O `kd-ts` lê o corpus do `kd` Rust sem aviso; o `kd` Rust lê o corpus do `kd-ts` sem aviso |
| C2 | **Compatibilidade de máquina** | `prime`, `--json`, `help`, mensagens de erro e *exit codes* byte-idênticos |
| C3 | **Determinismo** | Mesma entrada → mesma saída byte a byte, em execuções repetidas e em máquinas diferentes |
| C4 | **Pureza** | Zero imports de `node:*` no `core`; zero `Date.now`/`Math.random`/`process.*` fora das portas |
| C5 | **Cobertura de bordas** | As 115 entradas do `DIVERGENCES.md` reproduzidas como testes |
| C6 | **Desempenho** | Latência de arranque e throughput dentro do orçamento fixado na I01 |
| C7 | **Degradação** | Sem provedor de *embeddings*, sem git e com `.idx/` ausente, tudo funciona com `warnings` |
| C8 | **Valor acrescentado** | O *harness* diferencial encontra **pelo menos uma** divergência real do Rust |

---

## 8. Riscos globais e decisões abertas

| # | Questão aberta | Impacto | Quando decidir |
|---|---|---|---|
| Q1 | **Runtime-alvo**: Node puro, Bun ou SEA? | Define R1/R2 e o empacotamento | F0 (I01), com medição |
| Q2 | **Repositório**: port *in-place* com suíte diferencial, ou projeto novo a jusante? | Define a governação e o *fork* das decisões | F0 |
| Q3 | **Fonte de verdade das decisões**: `D01–D212` continuam canónicas no lado Rust? | Evita divergência de especificação | F1 |
| Q4 | **Numérico**: tolerância de *float* no `confidence_score` e no RRF — bit-exato ou épsilon? | Define o que é "paridade" | I06/I11 |
| Q5 | **ICU**: fixar a versão de Node como requisito duro ou *shim* de normalização próprio? | Resolve R2 de vez | I03 |
| Q6 | **Escopo do MCP**: 4 tools ou expor também `write`/`task`? | Amplia a superfície e o risco | I13 |
| Q7 | **MinHash/LSH**: portar já ou adiar até o *bench* mostrar necessidade? | Poupa trabalho na F4 | I08 |
| Q8 | **Submissão *upstream***: as divergências encontradas voltam ao `REF/knudge`? | Valor para o projeto original | I16 |

---

## 9. Plano de corte mínimo (MVP)

Se o objetivo for **valor cedo**, o corte é:

```
I01 → I02 → I03 → I04 → I05 → I06 → I08 → I12
```

que entrega um `kd-ts` capaz de **inicializar, buscar, escrever, atualizar e esquecer** notas
byte-compatíveis, com o contrato de máquina intacto — e já com o *harness* diferencial da I03/I06
a provar a paridade. Sem grafo, tarefas, ciclo de vida, MCP nem *embeddings*.

**Sequência recomendada de entrega:**

| Entrega | Issues | O que já se pode fazer |
|---|---|---|
| **E-alfa** | I01–I03 | Ler e escrever o formato; provar identidade |
| **E-beta** | I04–I06 | Persistir e buscar |
| **E-gama** | I07–I08 | Buscar bem e escrever sem duplicar |
| **MVP** | I09–I12 | Substituir o `kd` em uso real |
| **E-1.0** | I13–I17 | Paridade completa e distribuição |

---

## 10. Referências

- Resumo técnico de base: `exploration/knudge/knudge.md`
- Visão e princípios: `REF/knudge/README.md`, `REF/knudge/wiki/usage/01_filosofia.md`
- Arquitetura: `REF/knudge/wiki/specs/ARCHITECTURE.md`
- Contrato de bytes: `REF/knudge/wiki/specs/TOON.md`
- Fórmulas: `REF/knudge/wiki/specs/matematica.md`
- Bordas (base da suíte diferencial): `REF/knudge/wiki/specs/DIVERGENCES.md`
- Decisões: `REF/knudge/wiki/specs/adrs/` (201 ficheiros, `D01–D212`)
- Superfície CLI: `REF/knudge/plan/implementation/16_cli_surface.md`
- Determinismo e erros: `REF/knudge/wiki/specs/determinismo.md`, `REF/knudge/wiki/specs/erros.md`
- Avaliação: `REF/knudge/bench/RELATORIO.md`, `REF/knudge/bench/qualidade.md`
