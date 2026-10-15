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
agent: opencode
---

# OPENCODE_EPIC — Reimplementação do `knudge` em TypeScript

> **Base:** [`OPENCODE_RESEARCH.md`](./OPENCODE_RESEARCH.md) (análise técnica do `knudge` v0.5.2 em
> Rust). **Objetivo deste épico:** especificar uma **reimplementação fiel em TypeScript/Node.js**,
> com paridade de contrato byte-a-byte, plano mestre por fases e uma cadeia sequencial de issues.
>
> **Meta de fidelidade:** o `kd-ts` deve ser um **substituto drop-in** para o `kd` Rust no contrato
> de máquina (stdout pipe, envelope `--json`, exit codes, bytes de `notas/` e TOON, framing MCP).
> A estratégia de verificação é **diferencial**: gerar um corpus e uma matriz de comandos com o
> binário Rust e exigir saída **idêntica byte a byte** do binário TS.

---

## 0. Sumário executivo

| Dimensão | Decisão |
|---|---|
| Runtime | **Node.js ≥ 20 LTS**, ESM, TypeScript `strict` |
| Gerenciador | **pnpm workspaces** (monorepo com 3 pacotes) |
| Pacotes | `@knudge/core` (puro), `@knudge/cli` (bin `kd`), `@knudge/mcp` (servidor MCP) |
| Cripto | `node:crypto` `createHash('sha256')` |
| CLI | `commander` com camada própria de envelope/exit; help customizado |
| MCP | `@modelcontextprotocol/sdk` sobre stdio |
| Testes | `vitest` + `fast-check` (property) + **goldens gerados pelo Rust** |
| Lint | `eslint` + `typescript-eslint` (`strict-type-checked`) |
| Build | `tsup`/`esbuild`; binário único via **Node SEA** (fallback `bun build --compile`) |
| Paridade | **Differential harness** contra `target/release/kd` (fonte de ouro) |

**Tese do épico:** um porte para TS é *viável e até vantajoso* no ecossistema de agentes (MCP,
frameworks JS), mas **só se a disciplina de determinismo do Rust for reconstruída explicitamente**
— TS não oferece as garantias de bytes/ordem que o Rust oferece de graça. O épico existe
principalmente para **institucionalizar essas garantias**.

---

## 1. Por que TypeScript (vantagens)

1. **O ecossistema de agentes é TS-first.** O SDK oficial de MCP é TypeScript; LangChain.js,
   Vercel AI SDK, OpenAI Agents SDK e a maioria dos harnesses de agentes são JS/TS. Um
   `knudge` em TS encaixa **sem ponte FFI/subprocesso** em quem já orquestra o agente.
2. **Distribuição trivial.** `npm i -g @knudge/cli` funciona em qualquer SO sem toolchain de
   compilação (sem `rustup`, sem `cc`, sem `gcc`), ao contrário do `cargo build`.
3. **JSON é nativo.** O envelope `--json` e o `params` da CLI deixam de atravessar `serde`; a
   serialização canônica vira função pura, testável diretamente.
4. **Iteração rápida.** Hot reload, `vitest --watch`, e um pool de contribuidores muito maior.
5. **Um só código para múltiplos alvos.** O core puro pode rodar em Node, Bun, Deno, e até no
   browser/edge com adaptadores alternativos (`IndexedDB`, `localStorage`, memória).
6. **Async I/O natural.** O design original já é dominado por I/O; `Promise`/`async` é a forma
   idiomática (com os devidos cuidados de ordenação determinística).
7. **Tipos estruturais + validação em runtime.** Zod no *boundary* (CLI/MCP) dá uma rede de
   segurança que o Rust obtém com o type system; no port, usamos os dois.

## 2. Por que **não** TypeScript (desvantagens e mitigação)

| Desvantagem | Detalhe | Mitigação no épico |
|---|---|---|
| **Determinismo não é nativo** | Ordem de `Object.keys` (chaves "inteiras" são reordenadas!), `Map`/`Set` dependentes de inserção, `Number` sempre `f64`, formatação de float divergente do Rust | Regras duras: nunca iterar objeto direto; usar `Arrays` ordenados/`IndexMap` manual; formatador de escalares próprio (TS-005/TS-002) |
| **Strings UTF-16 ≠ UTF-8** | `"café".length === 4` mas são 4 bytes UTF-8 em 3 code points; hashing exige bytes UTF-8 | Normalizar → `TextEncoder`/`Buffer.from(s,'utf8')` sempre antes do hash (TS-002/TS-003) |
| **Sem enums fechados em runtime** | `type` pode vir como qualquer string de fora | Zod + tabelas de conjuntos fechados com sugestão "você quis dizer" (TS-006/TS-033) |
| **Sem garantia de ordem de execução** | `Promise.all` muda ordem de conclusão | Portas retornam valores; `Promise.all` + **remontagem na ordem de entrada** (TS-008/TS-035) |
| **Performance** | Cold start do Node (~30–50 ms), GC, BM25 O(N) em JS | Wave de otimização + aceitar meta de ≤2× o Rust (TS-036); o gargalo real (dedup O(N²)) já é mitigado por MinHash/LSH (TS-024) |
| **Binário único** | Node SEA/`pkg` é frágil e grande | SEA como alvo oficial + `bun build --compile` como alternativa; CI testa os dois (TS-036) |
| **Supply chain** | npm tem transitivos enormes vs `cargo-deny` | `pnpm audit`, lockfile, mínimo de deps, budget de dependências (TS-001) |
| **Erros como valores** | JS/TS tende a `throw`; ordem/atomicidade importam | API do core retorna `Result<T,E>` parcial (`{ok:true,...}|{ok:false,error}`); `throw` só na borda (TS-008) |
| **Sem `unsafe` proibido** | Não há o que proibir, mas `any` e `!` são equivalentes perigosos | `eslint` bane `any`, `as`, non-null `!`, `@ts-ignore` (TS-001) |

---

## 3. Mapa de porte (Rust → TS)

| Rust | TS | Observação |
|---|---|---|
| `knudge-core` (lib) | `packages/core` | domínio puro, sem `node:fs`/`process` |
| `knudge-core::adapters` (std) | `packages/core/src/adapters/node` (subpath) | único lugar com I/O |
| `knudge-core::ports` + `fakes` | `packages/core/src/ports` (`Clock`,`Rng`,`Env`,`Fs`,`Git`,`HookRunner`,`Logger`) + `fakes` | injeção de dependência |
| `knudge-cli` (bin `kd`) | `packages/cli` (`bin/kd.js`) | `commander` + envelope |
| `knudge-mcp` | `packages/mcp` (`bin/knudge-mcp.js`) | `@modelcontextprotocol/sdk` |
| `sha2` | `node:crypto` `createHash('sha256')` | |
| `thiserror` `enum Error` | `class KnudgeError` + `ErrorKind` union | exit code congelado |
| `BTreeMap`/`IndexMap` | array ordenada + `Map` com ordenação explícita | ordem canônica obrigatória |
| `proptest` | `fast-check` | invariantes + seed fixa |
| goldens Rust (`tests/golden/`) | goldens snapshot (`__snapshots__/`) **gerados pelo Rust** | paridade diferencial |

**Primitivos exatos a replicar (de `schema/hash.rs`, `schema/id.rs`):**

```
short_hash(parts) = u32 big-endian dos 4 primeiros bytes de SHA-256(concat(parts))
hex8(v)           = v.toString(16).padStart(8,'0')      // body_hash
base36_8(v)       = v.toString(36).padStart(8,'0')       // id; v é u32
id                = `${typePrefix}_${base36_8(short_hash([type,'\u001F',normalize(statement)]))}`
type_dir(id)      = id.split('_')[0] === 'container' ? 'epic' : id.split('_')[0]
```

> **Cuidado:** `Number.toString(36)` em JS opera sobre `f64`; para um `u32` de até 2³²−1 os dígitos
> são exatos, mas validar com `>>> 0` e casos-limite (`0xFFFFFFFF`). O alinhamento à direita com
> zeros deve ser idêntico ao `base36_8` Rust (8 posições, preenchidas com `0` à esquerda).

---

## 4. Regras de determinismo (equivalente TS do `clippy.toml` Rust)

O projeto Rust proíbe `HashMap`, `unwrap`, `panic`, `unsafe` e arquivos > 300 linhas. O
equivalente TS vira **lint rules + convenções**:

1. **Proibido em `packages/core`:** `node:fs`, `node:process`, `Date.now`, `Math.random`,
   `performance.now`, `new Date()` — tudo via portas.
2. **Proibida iteração implícita de objeto:** `Object.keys/values/entries` fora de helpers
   revisados; ordem derivada de `CANONICAL_KEYS`/`sort` explícito.
3. **Proibido `any`, `as` (fora de fronteira), `!` (non-null assertion), `@ts-expect-error`.**
4. **Números:** nada de `toLocaleString`/`Intl`; formatador de escalar próprio (inteiro sem `.0`,
   `NaN`/`Infinity` rejeitados, como o TOON Rust).
5. **Ordenação total sempre:** todo comparador termina em `id asc` (tie-break canônico do RRF).
6. **Paralelismo:** `Promise.all` é permitido, mas a **remontagem é na ordem de entrada**
   (espelha `Corpus::load_notes` por faixas de `list_ids`).
7. **Limite de 300 linhas** por arquivo de produção, verificado por script no CI.
8. **Hash sempre sobre `Buffer` UTF-8** de string normalizada.

---

## 5. Plano mestre (fases → issues → marcos)

| Fase | Tema | Issues | Marco (gate) |
|---|---|---|---|
| **A** | Fundação determinística | TS-001 → TS-013 | TOON round-trip byte-exato + store atômico |
| **B** | Grafo e recuperação | TS-014 → TS-022 | `ask` reproduz o pipe/`--json` do Rust |
| **C** | Escrita e ciclo de vida | TS-023 → TS-027 | `write` dedup/merge/supersede idênticos |
| **D** | Handoff, tarefas, saúde | TS-028 → TS-030 | `rewind`/`task`/`doctor` com paridade |
| **E** | Embeddings, superfície, MCP, entrega | TS-031 → TS-036 | **parity suite verde** + pacote publicado |

**Princípios de sequenciamento:** o épico é uma **cadeia linear** (cada issue depende da
anterior) para garantir que a base de determinismo esteja pronta antes do retrieval. Dentro de uma
fase, issues marcadas *(paralelizável)* podem ser feitas por trilhas distintas desde que a
anterior esteja concluída.

**Gate global por issue (Definition of Done do épico):**
- [ ] `pnpm check` verde (types + eslint + testes).
- [ ] Testes cobrindo o comportamento e o bug/regressão (sem `any`/`!`).
- [ ] Se tocar bytes: golden atualizado **a partir do Rust**, não inventado.
- [ ] Se tocar borda de determinismo: linha no equivalente TS de `DIVERGENCES.md` com o teste.
- [ ] Arquivo ≤ 300 linhas; sem deps novas sem justificativa (budget de dependências).

---

## 6. Issues (cadeia sequencial)

### Fase A — Fundação determinística

#### TS-001 — Bootstrap do monorepo e toolchain
- **Depende de:** —
- **Objetivo:** criar o workspace pnpm com `core`/`cli`/`mcp`, TS `strict`, e o gate de qualidade
  equivalente ao `make check`.
- **Entregáveis:** `pnpm-workspace.yaml`; `tsconfig.base.json` (`strict`, `noUncheckedIndexedAccess`,
  `exactOptionalPropertyTypes`); `eslint.config.js` com as regras do §4; `vitest` configurado;
  script `check` (types + lint + test + gate de 300 linhas); CI (GitHub Actions) espelhando
  `fmt/lint/test`; budget de dependências.
- **Aceite:** `pnpm check` verde num projeto vazio; CI reproduz local.
- **Refs Rust:** `Makefile`, `clippy.toml`, `AGENTS.md` §1.

#### TS-002 — Núcleo criptográfico: hash curto, `hex8`, `base36_8`
- **Depende de:** TS-001
- **Objetivo:** replicar exatamente `schema/hash.rs`.
- **Entregáveis:** `shortHash(parts: Uint8Array[]): number`; `hex8(u32): string`;
  `base36_8(u32): string` (right-aligned, 8 chars).
- **Aceite (property/diferencial):** para 10⁴ entradas aleatórias, `hex8`/`base36_8` batem com um
  oráculo Rust exportado; `shortHash` idêntico; casos-limite `0`, `0xFFFFFFFF`, `0x00000001`.
- **Refs Rust:** `schema/hash.rs` (bordas #3/#4).

#### TS-003 — Normalização Unicode e contagem de escalares
- **Depende de:** TS-002
- **Objetivo:** replicar `normalize` (NFC + trim + colapso) e `countScalars` (code points, não
  bytes/UTF-16).
- **Entregáveis:** `normalize(s)`; `countScalars(s)` via `[...s].length`; helpers UTF-8
  (`utf8(s): Buffer`).
- **Aceite:** idempotência de `normalize` (fast-check); equivalência precomposto/decomposto
  (`é` U+00E9 ≡ `e`+U+0301) colapsa; `countScalars("café")===4`.
- **Refs Rust:** `schema/body.rs`, bordas #1/#2/#96.

#### TS-004 — IDs endereçados por conteúdo + `type_dir`
- **Depende de:** TS-002, TS-003
- **Objetivo:** replicar `schema/id.rs` — `note_id`, `is_valid_note_id`, `type_dir`,
  `HISTORICAL_PREFIXES`.
- **Entregáveis:** funções puras com a constante `ID_SEPARATOR = '\u001F'` e prefixos históricos.
- **Aceite:** `note_id` bate com o Rust em toda a matriz de tipos; reclassificar `type` **não**
  reescreve o id; `type_dir("container_x")==="epic"`.
- **Refs Rust:** `schema/id.rs` (D01/D02/D149/D150).

#### TS-005 — Codec TOON (emissor canônico + parser tolerante)
- **Depende de:** TS-003
- **Objetivo:** replicar `toon/{lex,flow,parse,emit}.rs`: gramática linha-a-linha, escalares,
  `flow_list`/`flow_map`, escapes, comentários, 32 níveis, ordem canônica.
- **Entregáveis:** `parseToon`, `emitToon`; formatador de escalar (int sem `.0`, `true`/`false`,
  aspas só quando necessário, `\u{XXXX}`); `splitFrontmatter`.
- **Aceite:** round-trip **byte-exato** do corpus canônico; `1.0 → 1`; chave duplicada é erro;
  proptest `emit(parse(x)) === x` para mapas/listas de escalares.
- **Refs Rust:** `wiki/specs/TOON.md`, `toon/tests.rs` (borda #6).

#### TS-006 — Schema: tipos fechados, chaves canônicas e frontmatter
- **Depende de:** TS-004, TS-005
- **Objetivo:** replicar `schema/{types,keys,frontmatter,value,edge,outcomes}.rs`: 10 tipos
  (`fact, decision, question, task, def, error, snippet, link, meta, risk`) + `epic` derivado;
  enums fechados (`scope`, `classification`, `status`); **31 chaves** em ordem canônica;
  12 arestas + ontologia leve; `claims` SPO e `provenance`.
- **Entregáveis:** `NoteType`, `Scope`, `Classification`, `Status`, `EdgeKind`, `CANONICAL_KEYS`,
  `Frontmatter` com `toValue()` na ordem canônica; validação Zod na fronteira; helper de sugestão
  de enum fechado.
- **Aceite:** validação rejeita chave/tipo desconhecidos; opcionais vazios **omitidos** (nunca
  `null`); round-trip TOON do frontmatter.
- **Refs Rust:** `schema/types.rs`, `schema/keys.rs`, `schema/frontmatter.rs` (D04/D05/D13/D98/D207).

#### TS-007 — JSONL e codec JSON canônico
- **Depende de:** TS-005
- **Objetivo:** replicar `jsonl/` — leitura/escrita JSONL e serialização canônica (ordem de chaves
  determinística, sem depender do `JSON.stringify` de objetos arbitrários).
- **Entregáveis:** `decodeJsonl`, `encodeJsonl`, `canonicalJson(value)`.
- **Aceite:** round-trip estável; ordem determinística comprovada por fast-check.
- **Refs Rust:** `jsonl/`, `jsonl/json`.

#### TS-008 — Portas e fakes (DI determinística)
- **Depende de:** TS-007
- **Objetivo:** replicar `ports/` + `ports/fakes/` com as 7 portas e um `Result<T,E>` parcial.
- **Entregáveis:** `Clock` (+`FixedClock`), `Rng` (+`SeqRng`), `Env` (+`FakeEnv`), `Fs`
  (+`MemFs`,`FaultyFs` para injeção de falhas), `Git` (+`FakeGit`), `HookRunner`
  (+`NoopHookRunner`), `Logger` (+`RecordingLogger`); `KnudgeError`/`ErrorKind` com mapa
  `code()`→`exitCode()` congelado (`2,3,4,5,6,7,8,70`; 101 reservado a panic).
- **Aceite:** core testado só com fakes; nenhum import de `node:fs`/`process` no core (lint).
- **Refs Rust:** `ports/`, `error.rs` (R30–R35).

#### TS-009 — Store: layout, escrita atômica e nota-antes-evento
- **Depende de:** TS-008
- **Objetivo:** replicar `store/` — layout `notas/<tipo>/<id>.md` (com `type_dir`), escrita
  **tmp + rename** no mesmo diretório, `fsync`, e ordem **nota antes do evento**.
- **Entregáveis:** `Store` com `read/readOptional/write/remove/listIds/resolvePath`; escrita
  atômica (tmp determinístico + `rename`); tolerância a layout plano legado.
- **Aceite:** injeção de crash (`FaultyFs`) deixa nota sem evento (recuperável) e nunca arquivo
  pela metade; `MAP.md` na raiz não é nota.
- **Refs Rust:** `store/`, `store/tests/commit` (bordas #9/#10/#54/#70).

#### TS-010 — Lock advisory e varredura de resíduos
- **Depende de:** TS-009
- **Objetivo:** replicar `store/lock.rs` — lock por `open('wx')`, stale 30 s, reclaim por `rename`,
  fallback para `mtime`; `sweepResidues` só `*.tmp`/`*.stale` frescos.
- **Entregáveis:** `Lock`; `sweepResidues`.
- **Aceite:** dois escritores no mesmo alvo sem *lost update*; `NotFound` do `rename` não vira erro
  genérico; lock vivo nunca é roubado.
- **Refs Rust:** bordas #12/#13/#88.

#### TS-011 — Eventos (rotação, checkpoint) e rebuild double-buffer
- **Depende de:** TS-010
- **Objetivo:** replicar eventos append-only com rotação `max_bytes`, checkpoint de eventos e
  rebuild `.idx.new/` + rename atômico.
- **Entregáveis:** `EventLog`, `Checkpoint`, `rebuildIndex`.
- **Aceite:** leitor nunca vê índice parcial (stress); crash durante rebuild preserva o antigo.
- **Refs Rust:** bordas #11/#14.

#### TS-012 — Config em dois níveis (subset TOML) + sanitização
- **Depende de:** TS-011
- **Objetivo:** replicar `config/` — global (template) + projeto (efetivo, precedência), codec
  TOML subset com leitura preservando ordem e escrita canônica; segredos só no global e
  sanitizados em `effective`.
- **Entregáveis:** `Config`, `parseToml`, `emitToml`, `effective()`.
- **Aceite:** defaults + global + projeto batem com o Rust; `strict` só via config (sem flag);
  segredos nunca no projeto.
- **Refs Rust:** `config/`, D61/D91/D94/D97.

#### TS-013 — Binding Git: worktree, exclude, attributes, `sync`, `AGENTS.md`
- **Depende de:** TS-012
- **Objetivo:** replicar `git/` — worktree principal via `git rev-parse`, `.git/info/exclude`
  (nunca `.gitignore`), `.gitattributes` (`merge=union`), `sync` (nota+eventos), bloco `AGENTS.md`
  idempotente e a skill de `init`.
- **Entregáveis:** `GitBinding`, `applyAgentsBlock`, `sync`.
- **Aceite:** sincronização idempotente; bloco `knudge:start/end` não duplica nem sobrescreve texto
  do usuário; `merge=union` aplicado a `events*.jsonl`.
- **Refs Rust:** `git/`, D30/D31/D32/D60/D148.

**Marco A:** corpus canônico sobrevive a `parse → emit` byte-exato; store atômico e idempotente
sob crash-injection.

---

### Fase B — Grafo e recuperação

#### TS-014 — Grafo explícito: arestas, integridade, SCC e `expand`
- **Depende de:** TS-013
- **Objetivo:** replicar `graph/` — arestas no frontmatter, `link()` (sem duplicar/auto-aresta),
  verificação de bidirecionalidade `replaces`/`superseded_by`, SCC (Kosaraju iterativo) e `expand`
  BFS determinístico só no explícito.
- **Entregáveis:** `Graph`, `scc`, `expand(depth, via)`.
- **Aceite:** ordem de `expand` idêntica ao Rust; membros de ciclo nunca demovidos; aresta inválida
  rejeitada.
- **Refs Rust:** `graph/`, D45/D46/D49/D51.

#### TS-015 — Ontologia, claims e proveniência
- **Depende de:** TS-014
- **Objetivo:** replicar `graph/ontology.rs` e `schema/claims|provenance` — classes de equivalência
  (`same_as`), clausura `broader`/`narrower`, detecção de ciclos de hierarquia, conflitos de claim
  `(sujeito, rel)` com objetos divergentes; merge de claims sem duplicar.
- **Entregáveis:** `equivalenceClasses`, `broaderClosure`, `claimConflicts`, `mergeClaims`.
- **Aceite:** `broader` é transitivo e `narrower` seu inverso; merge de claim idempotente.
- **Refs Rust:** D207, bordas #110.

#### TS-016 — Autoridade (PageRank/PPR) e comunidades (Louvain)
- **Depende de:** TS-015
- **Objetivo:** replicar `graph/rank.rs` (PageRank/PPR com *dangling* redistribuído) e
  `graph/communities.rs` (Louvain determinístico, sem RNG).
- **Entregáveis:** `pagerank`, `personalizedPagerank`, `louvain`.
- **Aceite:** soma normalizada = 1; invariante a relabeling; partição determinística e invariante à
  ordem de arestas.
- **Refs Rust:** D192/D193, bordas #101/#102.

#### TS-017 — TMS (defeasible) e drift KL/JS
- **Depende de:** TS-016
- **Objetivo:** replicar `graph/tms.rs` (dependentes transitivos de premissa retratada; derrotados
  por `replaces`) e `lifecycle/term_drift` (JS por âncora, ≥4 notas, limiar).
- **Entregáveis:** `defeatedDependents`, `driftFactor`.
- **Aceite:** premissa retratada não se autoderrota; JS simétrico; `prune` só propõe.
- **Refs Rust:** D208, bordas #111.

#### TS-018 — Tokenizador: fold de diacríticos, stemming PT e stopwords
- **Depende de:** TS-017
- **Objetivo:** replicar `retrieval/token.rs` — `[a-z0-9_]`, **fold NFD→ASCII** no índice derivado,
  stemming PT conservador (mín. 4 bytes, plural normalizado), `content_terms` (stopwords PT+EN,
  fragmentos de 1 char). **Não** tocar `normalize`/`id`/`body_hash`.
- **Entregáveis:** `tokenize`, `foldDiacritics`, `stemPt`, `contentTerms`.
- **Aceite:** `café ≡ cafe`; plural/singular compartilham radical; idempotência do stem; bytes de
  `notas/` inalterados.
- **Refs Rust:** D172/D206, bordas #43/#96/#109.

#### TS-019 — BM25 (IDF por campo, peso por tipo, corte de alta frequência)
- **Depende de:** TS-018
- **Objetivo:** replicar `retrieval/bm25.rs` — `k1=1.5`, `b=0.75`, `statement` peso 3, peso por
  tipo, corte `df/N ≥ max_ratio` só acima de 64 notas; índice invertido com fallback por `df`.
- **Entregáveis:** `Bm25Index`, `scoreWith`.
- **Aceite:** score bate com varredura manual (oráculo); corte no-op em corpus pequeno.
- **Refs Rust:** D35/D37/D173, borda #97.

#### TS-020 — Âncoras: canal e filtro bidirecional
- **Depende de:** TS-019
- **Objetivo:** replicar canal de âncoras com globs `?`/`*`/`**` e filtro que casa nas **duas
  direções** (pedido↔âncora); suporte a `--anchor` sem query textual.
- **Entregáveis:** `AnchorChannel`, `globMatch` (DP de uma linha).
- **Aceite:** `ask --anchor` sem query encontra nota; glob casa nas duas direções.
- **Refs Rust:** D81/D86, bordas #22.

#### TS-021 — RRF: fusão, pesos e atribuição por canal
- **Depende de:** TS-020
- **Objetivo:** replicar `retrieval/rrf.rs` — `Σ peso_c/(k+rank+1)`, `k=60`, `anchor_weight=2.0`,
  `semantic_weight=30`, canal PPR opcional; desempate `(score desc, id asc)`; **parcelas por canal**
  que somam o score (para o `--json`).
- **Entregáveis:** `fuse`, `contributions`.
- **Aceite:** fusão determinística; soma das parcelas = score; ordem estável.
- **Refs Rust:** D81/D124/D151/D179, bordas #7/#65/#105.

#### TS-022 — Filtros, views `ready`/`blocked` e modos `rank`/`tags`/`suggest`
- **Depende de:** TS-021
- **Objetivo:** replicar `recall` completo: filtros `type/class/status/tags/anchors/scope`
  (transitivo `depends_on`) **antes** do BM25; views derivadas; `--rank` (universo conhecimento,
  `(confidence desc, id asc)`), `--tags` (`(count desc, tag asc)` ignorando mortas) e `--suggest`.
- **Entregáveis:** `recall(filters)`, `views`, `rank`, `tagCounts`.
- **Aceite:** canal vetorial intersectado com filtros; `[no_results]` literal; conjuntos fechados
  validados com sugestão.
- **Refs Rust:** D41/D53/D104/D107/D143/D146, bordas #34/#38/#45/#63/#64/#66.

**Marco B:** matriz de comandos de `ask` (query, `--id`, `--around`, `--rank`, `--tags`,
`--suggest`, `--json`, `--brief`, `--full-content`) **byte-idêntica** ao Rust sobre o corpus golden.

---

### Fase C — Escrita e ciclo de vida

#### TS-023 — Write: idempotência, dedup Dice e update/supersede
- **Depende de:** TS-022
- **Objetivo:** replicar `write/` — create idempotente por conteúdo (mesmo `body_hash` →
  `unchanged`; mesmo id com corpo diferente → conflito), protocolo `propose → decide → apply` com
  limiares `0.75/0.92`, merge (tags/âncoras unidas, `revision++`), `update` no lugar ou supersede.
- **Entregáveis:** `WriteEngine`, `propose`, `apply`, `update`, `supersede`.
- **Aceite:** mesma faixa de score → mesma ação do Rust; `write --type task` rejeitado; statement
  vazio → `invalid_input`.
- **Refs Rust:** `write/`, D26/D80/D01/D48, bordas #5/#75/#76.

#### TS-024 — MinHash/LSH para dedup denso
- **Depende de:** TS-023
- **Objetivo:** replicar `write/dedup/merges.rs` com *blocking* MinHash/LSH acima de 256 notas
  densas, **sem** mudar o limiar de merge nem o Dice exato.
- **Entregáveis:** `minhash`, `lshCandidatePairs`, `sieveMatchesReference`.
- **Aceite:** MinHash aproxima Jaccard; propõe todo quase-duplicado num corpus denso; pares
  canônicos e determinísticos; paridade com a peneira exata.
- **Refs Rust:** D204, borda #107; motivado pelo gargalo O(N²) do `RELATORIO.md`.

#### TS-025 — Confiança Beta-Bernoulli, shelf-life FSRS e decay
- **Depende de:** TS-024
- **Objetivo:** replicar `lifecycle/` — posterior Beta-Bernoulli com limite inferior conservador
  (sempre `[0,1]`), shelf-life `foundational=0`/`tactical=365`/`observational=30` com revisão
  espaçada (`base·growth%·reviews`, saturante, nunca encurta), decay de âncoras + drift de âncoras
  (D203), demolição soft por `expired`/`anchor_decay`/`contradicted`/`defeated`/`drifted`.
- **Entregáveis:** `confidenceScore`, `betaPosterior`, `effectiveExpiry`, `demotionCandidates`.
- **Aceite:** sem evidência → 0; limite inferior ≤ média; membros de ciclo excluídos; idade ≤ 0,05.
- **Refs Rust:** D87/D189/D190/D203, bordas #98/#99/#103/#106.

#### TS-026 — Purga com retenção e limpeza de arestas
- **Depende de:** TS-025
- **Objetivo:** replicar purga — `retired_at` derivado de eventos, remoção só após janela de
  retenção, `--purge --force` libera tombstone, e `Store.remove` limpa arestas de entrada e
  `superseded_by` antes de apagar.
- **Entregáveis:** `purge`, `retention`.
- **Aceite:** arestas penduradas limpas; purga de tombstone recente só com `--force`.
- **Refs Rust:** D48/D84, bordas #48/#79.

#### TS-027 — Clusters (estrutural e semântico)
- **Depende de:** TS-026
- **Objetivo:** replicar fase 1 determinística (por `anchor`/`type`/`classification`/scope) e fase 2
  semântica **dentro** do cluster estrutural, off-path, acima do volume mínimo.
- **Entregáveis:** `structuralClusters`, `semanticClusters`.
- **Aceite:** fase 1 sem estatística/embeddings; fase 2 similaridade injetada.
- **Refs Rust:** D47/D128, `retrieval/index`.

**Marco C:** `write` e manutenção (`compact`/`learn`/`prune`) reproduzem as propostas e ações
idênticas ao Rust; dedup denso dentro do orçamento de latência.

---

### Fase D — Handoff, tarefas e saúde

#### TS-028 — Handoff (`rewind`) e manutenção (`diff`/`learn`/`compact`)
- **Depende de:** TS-027
- **Objetivo:** replicar `handoff/` — três modos (manifest/escopo/working set), ranking
  `star*100 + foundational*50 + tactical*20 + observational*10`, orçamento `ceil(chars/4)` default
  4000, `context_id` com `--resume` byte-idêntico, `next:`/`fresh:`; e `maintenance/` (`diff` lê
  eventos, `learn` propõe `create_note`/`merge`/`supersede`/`link`, `compact` propõe).
- **Entregáveis:** `rewind`, `budget`, `contextId`, `diff`, `learn`, `compact`.
- **Aceite:** `--resume` devolve bytes idênticos; corpo só para `foundational`/`decision`; nada
  escrito sem aceite.
- **Refs Rust:** `handoff/`, `maintenance/`, D40/D57/D82/D88, bordas #92.

#### TS-029 — Tarefas: hierarquia, progresso, impacto, `graph`, `flow`, `plan`
- **Depende de:** TS-028
- **Objetivo:** replicar `task/` — `epic ⊃ {issue ⊃ task | task}` (raiz epic, issue opcional),
  progresso/rollup derivado, `--sort impact` (caminho crítico por desbloqueio, `impacto desc,
  created asc, id asc`), `graph` (floresta de programa, papel/modo), `flow` (cycle/lead/throughput +
  PERT/CPM), `plan` (`--prompt`/`--submit` atômico com validação total antes de escrever).
- **Entregáveis:** `tasks`, `hierarchy`, `impact`, `flow`, `plan`.
- **Aceite:** plano inválido não escreve nada; pai com `rank` estritamente menor; entre irmãos ordem
  determinística.
- **Refs Rust:** D52/D93/D105/D109/D115/D116/D119/D127/D134/D149/D205, bordas #23/#31/#52/#53/#108.

#### TS-030 — Saúde: validators, `doctor` (13 checks) e leitura tolerante
- **Depende de:** TS-029
- **Objetivo:** replicar `health/` — catálogo `validators.toml`, resolução
  `explícitos ∪ globais ∪ por_âncora`, `doctor` com 13 checks (inclui integridade semântica
  claims/ciclos e tamanho do índice vetorial) e `--fix` **idempotente**, `audit` como default,
  âncoras por hash, leitura tolerante (chave desconhecida → warning; tipo desconhecido → skip).
- **Entregáveis:** `doctor`, `audit`, `validators`, `tolerantRead`.
- **Aceite:** `--fix` idempotente; checks advisórios não bloqueiam `healthy`; nota ruim não derruba
  o corpus.
- **Refs Rust:** `health/`, D16–D19/D46/D54/D163, bordas #24/#71/#72/#77/#81/#93.

**Marco D:** `rewind`, `task *`, `doctor` com paridade; tarefas e saúde atravessam o corpus golden.

---

### Fase E — Embeddings, superfície, MCP e entrega

#### TS-031 — Embeddings: porta, adaptador HTTP, fila lazy e cache
- **Depende de:** TS-030
- **Objetivo:** replicar `embeddings/` — porta `Embedder`, adaptador `http` (OpenAI-compatible,
  `fetch`), `lightweight` (hash) e `none`; índice `.idx/embeddings.jsonl` com `meta`
  (provider/model/revision/dims) que invalida ao mudar; fila `indexed|pending|stale` derivada do
  `body_hash`; cache por `body_hash` com LRU; *dirty flag*/debounce; purga via `purgeDerived`.
- **Entregáveis:** `Embedder`, `HttpEmbedder`, `EmbeddingIndex`, `queue`, `cache`.
- **Aceite:** nunca bloqueia write/read; mudar modelo força re-embed; falha vira `pending`; auto-drain
  best-effort não toca exit/warnings.
- **Refs Rust:** `embeddings/`, D79/D83/D85/D101/D102/D131, bordas #20/#46/#50/#68/#69.

#### TS-032 — Ranking semântico e sugestões semânticas
- **Depende de:** TS-031
- **Objetivo:** replicar `rank_query` (brute-force cosseno/dot) alimentando o canal vetorial
  **intersectado com filtros**, e `suggest` (banda `low..high`, classifica `duplicate`/
  `contradiction`/`link`, par já ligado nunca sugere).
- **Entregáveis:** `semanticChannel`, `classifyPair`, `suggestions`.
- **Aceite:** `why=semantic` correto; sugestão nunca vira aresta; classificação determinística.
- **Refs Rust:** D102/D158, bordas #26/#42/#86.

#### TS-033 — CLI: 14 verbos, envelope `--json`, exit codes e `init`/`prime`
- **Depende de:** TS-032
- **Objetivo:** replicar a superfície congelada — 14 verbos (`init, prime, rewind, ask, write,
  task, map, doctor, drain, maintenance, config, forget, sync, self`), `kd` sem args = help,
  `prime` **byte-idêntico por versão**, envelope
  `{success, command, data?, error{code,message,retryable}, warnings?}`, stdout=dados/stderr=logs,
  EPIPE→0, listas por repetição/vírgula, conjuntos fechados com sugestão.
- **Entregáveis:** parser `commander` com help customizado; camada de envelope; `emitStdout`;
  logging com `Redactor`.
- **Aceite:** goldens de `prime`/help/`--json`/erros batem com o Rust; `ask` sem modo → uso (2);
  nenhum log vaza para stdout.
- **Refs Rust:** `plan/implementation/16_cli_surface.md`, R20–R23/D71/D73/D167/D171/D210/D212,
  bordas #17/#18/#19/#44/#51/#113/#114/#115.

#### TS-034 — Servidor MCP (stdio JSON-RPC + 4 gatilhos)
- **Depende de:** TS-033
- **Objetivo:** replicar `knudge-mcp` — JSON-RPC 2.0 sobre stdio, **uma linha por mensagem** (sem
  `Content-Length`), `initialize`/`ping`/`tools/list`/`tools/call`; tools `knudge_pre_write`,
  `knudge_pre_edit`, `knudge_session_end`, `knudge_status`; hints **ponteiro** (`id+statement+score`,
  cap 3, dedup por sessão, modo observação); parse inválido → `id:null`; EPIPE/EOF → exit 0.
- **Entregáveis:** `packages/mcp` com `@modelcontextprotocol/sdk`; `HintEngine`.
- **Aceite:** teste de stdio handshake + tools; framing idêntico ao Rust; nenhum corpo de nota no
  contexto.
- **Refs Rust:** `crates/knudge-mcp/`, D68/D71/D73, bordas #21.

#### TS-035 — Suíte de paridade diferencial contra o `kd` Rust
- **Depende de:** TS-034
- **Objetivo:** construir o **oráculo de fidelidade**. Um gerador determinístico cria o corpus
  golden; o binário Rust (`target/release/kd`) executa uma **matriz de comandos** e suas saídas
  (pipe, `--json`, exit, bytes de `notas/`); o TS é validado contra essa matriz.
- **Entregáveis:** `parity/` com: gerador de corpus; matriz de comandos (fan-out por verbo/flag);
  comparador byte-a-byte; relatório de divergências (equivalente TS de `DIVERGENCES.md`); tags de
  `fast-check` com seeds congeladas.
- **Aceite:** 100% de paridade na matriz na versão-alvo; qualquer divergência vira linha catalogada
  com teste que a trava; corpus grande (≥1000 notas) incluído.
- **Refs Rust:** `crates/*/tests/golden/`, `DIVERGENCES.md`, `bench/`.

#### TS-036 — Bancada, otimização e empacotamento
- **Depende de:** TS-035
- **Objetivo:** medir latência (medianas) e qualidade (R@k/MRR/nDCG@k) no TS, otimizar gargalos
  (reconstrução de `Index`+`Graph`, `rewind`, cold start) e distribuir.
- **Entregáveis:** `bench/` TS (e2e + micro + qualidade); alvo **binário único** (Node SEA; fallback
  `bun build --compile`); `install.sh`/`npm publish`; man page/completions; docs.
- **Aceite:** ≤2× o Rust nas ações quentes (ou justificado); qualidade ≥ baseline Rust;
  `npx @knudge/cli` e binário único funcionam em Linux/macOS/Windows.
- **Refs Rust:** `bench/RELATORIO.md`, `bench/qualidade.md`, `install.sh`.

**Marco E (e do épico):** parity suite verde, pacote publicado e binário único instalável.

---

## 7. Grafo de dependências (resumo)

```
TS-001 → 002 → 003 → 004 → 005 → 006 → 007 → 008 → 009 → 010 → 011 → 012 → 013
                                                                              │
        ┌─────────────────────────────────────────────────────────────────────┘
        ▼
TS-014 → 015 → 016 → 017 → 018 → 019 → 020 → 021 → 022
                                                     │
        ┌────────────────────────────────────────────┘
        ▼
TS-023 → 024 → 025 → 026 → 027
                             │
        ┌────────────────────┘
        ▼
TS-028 → 029 → 030
                │
        ┌───────┘
        ▼
TS-031 → 032 → 033 → 034 → 035 → 036
```

Cada issue **só começa** com a anterior no estado "Done" (a cadeia é o contrato). Exceções
explícitas de paralelismo: TS-010/TS-011/TS-012 (independentes entre si após TS-009); TS-016/TS-017
(após TS-015); TS-031 e a bancada TS-036 podem ser adiantadas em trilha paralela se houver folga.

---

## 8. Riscos globais e planos de contingência

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| Divergência de float/format no TOON | Alta | Alto | Formatador próprio + golden do Rust (TS-005/TS-035) |
| Ordem de `Object`/`Map` vazar p/ bytes | Alta | Alto | Lint + revisão + paridade (TS-001/§4) |
| `base36_8`/`hex8` com borda `u32` | Média | Alto | Property vs oráculo Rust (TS-002) |
| Strings UTF-16 vs bytes UTF-8 | Alta | Alto | `normalize`→`Buffer` obrigatório (TS-003) |
| Performance de cold start/GC | Média | Médio | SEA + medição; meta ≤2× (TS-036) |
| Supply chain npm | Média | Médio | Budget de deps, `pnpm audit`, lockfile (TS-001) |
| MCP framing divergente | Média | Alto | Teste de stdio contra o framing Rust (TS-034) |
| Deriva de contrato (Rust evolui) | Média | Médio | Parity suite roda contra uma versão Rust pinada |

---

## 9. Critérios de sucesso do épico

1. **Paridade:** a matriz de comandos produz saída **idêntica byte a byte** ao `kd` Rust (versão
   pinada) — pipe, `--json`, exit codes, bytes de `notas/`, framing MCP.
2. **Determinismo:** nenhuma iteração implícita, nenhum relógio/RNG real no core, verificado por
   lint e por testes determinísticos repetíveis.
3. **Ecossistema:** `kd-ts` e `knudge-mcp-ts` integram-se a um agente JS/TS **sem subprocesso**, e
   rodam no browser/edge via portas alternativas (prova de extensibilidade).
4. **Empacotamento:** instalação por `npm` **e** binário único para os 3 SOs.
5. **Qualidade:** bancada TS ≥ baseline Rust (R@k/MRR/nDCG@k) e latência dentro do orçamento.

---

## 10. Referências

- Análise base: [`OPENCODE_RESEARCH.md`](./OPENCODE_RESEARCH.md).
- Contrato de bytes: `REF/knudge/wiki/specs/TOON.md`.
- Arquitetura: `REF/knudge/wiki/specs/ARCHITECTURE.md`.
- Bordas/determinismo: `REF/knudge/wiki/specs/DIVERGENCES.md`.
- Superfície CLI: `REF/knudge/plan/implementation/16_cli_surface.md`.
- Decisões/políticas: `REF/knudge/plan/03_decisoes-fechadas.md`, R01–R44.
- Performance: `REF/knudge/bench/RELATORIO.md`; qualidade: `REF/knudge/bench/qualidade.md`.
