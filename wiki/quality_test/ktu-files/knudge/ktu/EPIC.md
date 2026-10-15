# EPIC: Reimplementação do knudge em TypeScript

> **Objetivo**: reimplementar o knudge (CLI Rust) preservando semântica, comportamento e
> arquitetura, usando TypeScript + Node.js.
> **Métricas de sucesso**: `kd --help` equivalente, mesma ontologia, TOON byte-idêntico, suite de
> testes equivalente, benchmark sem regressão >2× em operações quentes.

---

## 1. Contexto e Motivação

O knudge resolve memória durável para agentes de IA por projeto com três pilares:
notas em Markdown como verdade, índice derivado reconstruível e busca híbrida sem servidor.

Rust foi escolhido pela determinismo, segurança de memória e binário único. TypeScript oferece:

- **Ecosistema NPM** (CLI, MCP SDK, embeddings, parsing)
- **Onboarding** maior (mais devs conhecem TS que Rust)
- **Tipagem estruturada** + discriminated unions para o modelo de domínio
- **Async nativo** para I/O (embeddings HTTP, fila de drain)

## 2. Vantagens e Desvantagens

### Vantagens

| Aspecto | TypeScript | Impacto |
|---|---|---|
| **Tempo de desenvolvimento** | Sem borrow checker, sem lifetimes | Iteração 2–3× mais rápida |
| **Ecosistema** | `@modelcontextprotocol/sdk`, `commander`, `zod` | Menos código boilerplate |
| **Testes** | Vitest com mocks nativos | Menos ceremony que Rust traits |
| **Async** | `Promise`/`async-await` nativo | Fila de embeddings mais simples |
| **Tipagem** | Discriminated unions + zod runtime | Contrato em compile-time e runtime |
| **Deploy** | `npx`, `npm i -g`, `bun build --compile` | Distribuição sem toolchain Rust |
| **Contribuição** | Barreira de entrada menor | Mais contribuidores potenciais |

### Desvantagens

| Aspecto | Risco | Mitigação |
|---|---|---|
| **Determinismo** | `Map`/`Set` iteram em ordem de inserção, não canônica | Usar `BTreeMap`-like ou ordenar explicitamente; `BigInt` para hashes |
| **Performance** | GC pauses, JIT cold start | Benchmarks por fase; lazy heavy paths; `bun` se necessário |
| **Binário único** | Node runtime ou `bun build --compile` (beta) | Aceitar dependência de runtime ou usar SEA (Single Executable App) |
| **Concorrência** | Event loop, sem threads nativos simples | `worker_threads` apenas para embeddings; fila em memória |
| **Overhead de runtime** | RAM 40–80 MB vs 5–10 MB | Aceitar trade-off; otimizar cold start (`--quick`) |
| **Tipagem em runtime** | TS apaga tipos em runtime | Validar boundaries com zod (TOON, envelope JSON) |
| **`unsafe` equivalente** | `node:buffer`, FFI não necessário | Manter em wrapper isolado |
| **Stdlib** | Menos rica que Rust std | Usar libs maduras (`minimatch`, `gray-matter`, `simple-git`) |

### Veredito

TypeScript é viável e preferível quando **velocidade de iteração** e **acessibilidade** importam
mais que **binário único** e **RAM mínima**. O knudge é I/O-bound (FS + rede opcional), não
CPU-bound, então o overhead de runtime é aceitável.

---

## 3. Arquitetura Alvo

```
@knudge/mcp ──┐
              ├── @knudge/core  (domínio puro + ports)
@knudge/cli ──┘
```

| Package | Papel |
|---|---|
| `@knudge/core` | Modelo, TOON, retrieval, lifecycle, **ports** (interfaces TS) |
| `@knudge/cli` | Binário `kd`: commander, envelope `--json`, adaptadores concretos |
| `@knudge/mcp` | Servidor MCP via `@modelcontextprotocol/sdk` |
| `@knudge/fs` | Adaptador concreto: `node:fs`, `simple-git`, `node:crypto` |

**Convenções**:
- Monorepo com `npm workspaces` ou `pnpm`
- ESMstrict, `"module": "NodeNext"`
- Testes: Vitest + prop-based com `fast-check`
- Lint/format: ESLint + Prettier (regras espelham `Cargo.toml` limits)

---

## 4. Plano Mestre

### Fase 0 — Fundação (setup, portas, TOON)

**Objetivo**: repositório válido, TOON byte-idêntico, CI.

- [ ] **0.1** Monorepo + workspaces + ESLint + Prettier + Vitest
- [ ] **0.2** Portas (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`) como interfaces
- [ ] **0.3** TOON writer/parser byte-idêntico (31 chaves, ordem canônica, normalize, id)
- [ ] **0.4** Proptest-equivalente: `fast-check` para TOON round-trip + dedup score
- [ ] **0.5** CI (lint, test, build, TOON golden tests contra corpus Rust)

**Critério de aceite**: `npm test` passa; TOON output idêntico ao corpus de `REF/knudge`.

### Fase 1 — Core (modelo, lifecycle, grafo)

- [ ] **1.1** Modelo de domínio (`Note`, `NoteId`, `NoteType`, `Lifecycle`, `Anchor`, `Edge`)
- [ ] **1.2** Erros como valores (`KnudgeError`, `ErrorKind`, código → exit code)
- [ ] **1.3** Lifecycle (shelf-life, decay, purge, confiança derivada, clusters)
- [ ] **1.4** Grafo leve (arestas `same_as`, `broader`, `narrower`, `related`; TMS)
- [ ] **1.5** Âncoras (`--anchor` com glob, repetíveis, detecção de quebradas)

**Critério**: testes unitários cobrem todas as decisões Dxx relevantes.

### Fase 2 — Retrieval (BM25, filtros, RRF)

- [ ] **2.1** Indexador BM25 (tokenização, stemming PT/EN opcional)
- [ ] **2.2** Filtros estruturados (tags, tipos, classes, status, escopo, âncoras, tempo, `--as-of`)
- [ ] **2.3** RRF (fusão de canais: lexical, anchor, recent, stars, body)
- [ ] **2.4** `.idx/` persistente (JSON compressível ou SQLite opcional)
- [ ] **2.5** Rebuild incremental (nota → bump de índices afetados)

**Critério**: recall ≥ 0.95 vs implementação Rust em corpus de teste.

### Fase 3 — CLI (ask, write, task, sync, doctor)

- [ ] **3.1** `kd init` (funda `.knudge/`, `AGENTS.md`, `.agents/skill/kd/SKILL.md`)
- [ ] **3.2** `kd ask` (BM25 + RRF, `--brief`, progressivo, `--full-content`, `--rank`, `--tags`, `--suggest`, `--around`)
- [ ] **3.3** `kd write` (dedup, `--dry-run`, `--batch`, `--update`, `--outcome`, `--claim`, proveniência)
- [ ] **3.4** `kd task` (epic/issue/task, fechamento com evidência, `--graph`)
- [ ] **3.5** `kd rewind` (manifest, orçamento, `context_id`)
- [ ] **3.6** `kd doctor` (13 checks + `--fix` reversível + `--explain`)
- [ ] **3.7** `kd forget` (soft-delete → restore → purge)
- [ ] **3.8** `kd sync` (commit no git via `simple-git`)
- [ ] **3.9** `kd prime` (protocolo estático, byte-idêntico por versão)
- [ ] **3.10** `kd --help` / `--json` envelope / `--version`

**Critério**: golden tests de CLI contra fixtures; `--json` envelope compatível.

### Fase 4 — MCP + Embeddings

- [ ] **4.1** Servidor MCP (`knudge-mcp`) com gatilhos (`pre_write`, `pre_edit`, `session_end`, `status`)
- [ ] **4.2** Transporte JSON-RPC 2.0 sobre stdio
- [ ] **4.3** Provedor de embeddings HTTP (llama.cpp + GGUF) como adaptador
- [ ] **4.4** Cache persistente + fila com auto-drain (`kd drain`)
- [ ] **4.5** Degradação graciosa (sem embeddings → BM25 + `warnings[]`)

**Critério**: cliente MCP se conecta; round-trip de ask/write funciona via stdio.

### Fase 5 — Qualidade e Migração

- [ ] **5.1** Benchmarks (`bench/`) comparando TS vs Rust
- [ ] **5.2** Fuzzing (TOON, JSONL) com `fast-check` ou `jazzer`
- [ ] **5.3** Guia de migração Rust → TS (diff de API, schema TOON, fixtures)
- [ ] **5.4** Wiki de usage (00–19) atualizada para TS
- [ ] **5.5** `llms.txt` atualizado

**Critério**: diferenças de API documentadas; benchmarks publicados.

---

## 5. Issues Encadeadas (Sequência)

```
EPIC-000 Fase 0: Fundação
  └─ ISSUE-001 Monorepo + toolchain
       └─ TASK-001.1 pnpm workspace + ESLint + Prettier
       └─ TASK-001.2 Vitest setup + exemplo de teste
  └─ ISSUE-002 Portas (interfaces TS)
       └─ TASK-002.1 Clock, Rng, Env, Logger
       └─ TASK-002.2 Fs, Git, HookRunner
  └─ ISSUE-003 TOON byte-idêntico
       └─ TASK-003.1 Parser (31 chaves, ordem canônica)
       └─ TASK-003.2 Writer + normalize (NFC, trim, colapso)
       └─ TASK-003.3 ID derivation (type + 0x1F + statement, base36)
       └─ TASK-003.4 Golden tests vs corpus Rust
  └─ ISSUE-004 Determinismo + proptest
       └─ TASK-004.1 fast-check para TOON round-trip
       └─ TASK-004.2 dedup score deterministic
       └─ TASK-004.3 iteração ordenada (BTreeMap-like)
  └─ ISSUE-005 CI
       └─ TASK-005.1 GitHub Actions (lint, test, build)
       └─ TASK-005.2 TOON golden no CI
       └─ TASK-005.3 coverage report

EPIC-100 Fase 1: Core (modelo, lifecycle, grafo)
  └─ ISSUE-101 Modelo de domínio
       └─ TASK-101.1 Note, NoteId, NoteType, Lifecycle
       └─ TASK-101.2 Discriminated unions + zod validation
       └─ TASK-101.3 Migração de decisões Dxx relevantes
  └─ ISSUE-102 Erros como valores
       └─ TASK-102.1 KnudgeError + KindError
       └─ TASK-102.2 exit code mapping
       └─ TASK-102.3 EPIPE → 0
  └─ ISSUE-103 Lifecycle
       └─ TASK-103.1 shelf-life, decay, purge
       └─ TASK-103.2 confiança derivada
       └─ TASK-103.3 clusters
  └─ ISSUE-104 Grafo leve
       └─ TASK-104.1 Arestas same_as, broader, narrower, related
       └─ TASK-104.2 TMS (truth maintenance)
       └─ TASK-104.3 sugestões de aresta/contradição
  └─ ISSUE-105 Âncoras
       └─ TASK-105.1 --anchor com glob (minimatch)
       └─ TASK-105.2 repetíveis
       └─ TASK-105.3 deteção de âncoras quebradas

EPIC-200 Fase 2: Retrieval
  └─ ISSUE-201 BM25
       └─ TASK-201.1 tokenização + stemming opcional
       └─ TASK-201.2 index build
       └─ TASK-201.3 index query
  └─ ISSUE-202 Filtros estruturais
       └─ TASK-202.1 tags, tipos, classes, status
       └─ TASK-202.2 escopo, intervalo temporal
       └─ TASK-202.3 --as-of histórico
  └─ ISSUE-203 RRF
       └─ TASK-203.1 fusão lexical, anchor, recent, stars, body
       └─ TASK-203.2 parâmetro k do RRF
       └─ TASK-203.3 desempate determinístico
  └─ ISSUE-204 Persistência do .idx/
       └─ TASK-204.1 JSON compressível ou SQLite
       └─ TASK-204.2 rebuild incremental
       └─ TASK-204.3 migração de versão de índice
  └─ ISSUE-205 Benchmarks de recall
       └─ TASK-205.1 corpus de teste derivado de REF/knudge
       └─ TASK-205.2 recall ≥ 0.95 vs Rust
       └─ TASK-205.3 latência p50/p95

EPIC-300 Fase 3: CLI
  └─ ISSUE-301 kd init
       └─ TASK-301.1 funda .knudge/
       └─ TASK-301.2 AGENTS.md managed block
       └─ TASK-301.3 SKILL.md em .agents/skill/kd
  └─ ISSUE-302 kd ask
       └─ TASK-302.1 BM25 + RRF (integra EPIC-200)
       └─ TASK-302.2 --brief, progressivo, --full-content
       └─ TASK-302.3 --rank, --tags, --suggest, --around
       └─ TASK-302.4 pipe formatado + --json envelope
  └─ ISSUE-303 kd write
       └─ TASK-303.1 dedup probabilístico
       └─ TASK-303.2 --dry-run, --batch (JSONL)
       └─ TASK-303.3 --update, --outcome, --claim
       └─ TASK-303.4 proveniência (agent/activity)
  └─ ISSUE-304 kd task
       └─ TASK-304.1 hierarquia epic/issue/task
       └─ TASK-304.2 fechamento exige evidência
       └─ TASK-304.3 --graph
  └─ ISSUE-305 kd rewind
       └─ TASK-305.1 manifest + orçamento
       └─ TASK-305.2 context_id
  └─ ISSUE-306 kd doctor
       └─ TASK-306.1 13 checks + auditoria
       └─ TASK-306.2 --fix reversível
       └─ TASK-306.3 --explain
  └─ ISSUE-307 kd forget
       └─ TASK-307.1 soft-delete → restore
       └─ TASK-307.2 purge com retenção
  └─ ISSUE-308 kd sync
       └─ TASK-308.1 commit notas/ + eventos/
       └─ TASK-308.2 integração simple-git
  └─ ISSUE-309 kd prime + --help
       └─ TASK-309.1 protocolo estático
       └─ TASK-309.2 byte-idêntico por versão
       └─ TASK-309.3 help text

EPIC-400 Fase 4: MCP + Embeddings
  └─ ISSUE-401 Servidor MCP
       └─ TASK-401.1 @modelcontextprotocol/sdk setup
       └─ TASK-401.2 gatilhos pre_write, pre_edit, session_end, status
       └─ TASK-401.3 JSON-RPC 2.0 stdio
  └─ ISSUE-402 Embeddings HTTP
       └─ TASK-402.1 provedor HTTP (llama.cpp + GGUF)
       └─ TASK-402.2 cache persistente
       └─ TASK-402.3 fila + auto-drain
       └─ TASK-402.4 modo lazy
  └─ ISSUE-403 Degradação graciosa
       └─ TASK-403.1 sem embeddings → BM25 + warnings
       └─ TASK-403.2 strict no config
  └─ ISSUE-404 kd drain
       └─ TASK-404.1 worker de fila
       └─ TASK-404.2 progresso + retry

EPIC-500 Fase 5: Qualidade e Migração
  └─ ISSUE-501 Benchmarks
       └─ TASK-501.1 setup bench/ comparando TS vs Rust
       └─ TASK-501.2 cold start, ask, write, sync
       └─ TASK-501.3 publicação de resultados
  └─ ISSUE-502 Fuzzing
       └─ TASK-502.1 TOON fuzzer
       └─ TASK-502.2 JSONL batch fuzzer
       └─ TASK-502.3 prop-based com fast-check
  └─ ISSUE-503 Guia de migração
       └─ TASK-503.1 diff de API Rust → TS
       └─ TASK-503.2 fixtures TOON
       └─ TASK-503.3 wiki de usage (00–19)
  └─ ISSUE-504 llms.txt + README
       └─ TASK-504.1 atualizar llms.txt
       └─ TASK-504.2 README TS
       └─ TASK-504.3 changelog

```

---

## 6. Convenções e Limites

- **Arquivos TS** ≤ 300 linhas
- **Sem `any` sem justificativa** (usar `unknown` + narrowing)
- **Erros como valores**: `KnudgeError` class com `code`, `kind`, `message`
- **Determinismo**: sem `Math.random()` em lógica de domínio (usar port `Rng`)
- **Testes**: 100% coverage em core; integração em CLI
- **Async**: ports são `async` ou sync conforme necessário; documentar

---

## 7. Riscos Conhecidos

| Risco | Probabilidade | Mitigação |
|---|---|---|
| TS não garante byte-idêntico sem rigor | Médio | golden tests no CI contra corpus Rust |
| Performance degrada >2× | Médio | benchmarks na Fase 5; profile cedo |
| Concorrência de file lock | Baixo | uso de `proper-lockfile` ou FS atómico |
| Ecosystem churn (NPM packages) | Médio | pin deps, lockfile, Dependabot |
| Migrar usuários Rust → TS | Baixo | manter compatibilidade de API e TOON |

---

## 8. Critérios de Sucesso Globais

1. **Compatibilidade semântica**: comportamento idêntico ao knudge Rust para os comandos
   documentados.
2. **Compatibilidade de dados**: TOON byte-idêntico; notas Rust → TS sem migração.
3. **Qualidade**: lint, 100% coverage core, CI verde, fuzzing.
4. **Performance**: cold start < 500ms; ask p95 < 50ms em corpus de 10k notas.
5. **Adoção**: publicado em NPM (`@knudge/cli`, `@knudge/mcp`); install com `npx`.

---

## 9. Referências

- `exploration/knudge/ktu/knudge.md` — resumo de pesquisa
- `REF/knudge/plan/03_decisoes-fechadas.md` — decisões Dxx
- `REF/knudge/wiki/specs/TOON.md` — contrato de bytes
- `REF/knudge/wiki/specs/ARCHITECTURE.md` — arquitetura
- `REF/knudge/AGENTS.md` — regras de contribuição
- `REF/knudge/wiki/usage/` — guias por comando
- `@modelcontextprotocol/sdk` — MCP TypeScript SDK
- `commander` — CLI framework
- `zod` — runtime validation
- `fast-check` — property-based testing
