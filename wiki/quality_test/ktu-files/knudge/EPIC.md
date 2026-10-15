# EPIC: Reimplementar knudge em TypeScript

> **Objetivo:** reimplementar o knudge (memória durável por projeto para agentes de IA) em
> TypeScript, preservando os contratos semânticos e o determinismo, mas trocando a toolchain
> Rust por Node.js/TypeScript.
>
> **Fonte de verdade:** `exploration/knudge/resumo.md` (v0.5.4).
> **Data:** 2025-10-15.

---

## 1. Contexto e motivação

O knudge é uma CLI Rust que implementa memória durável por projeto usando notas Markdown,
índice derivado, dedup por MinHash+LSH, ranking BM25+RRF, e um grafo de dependências com
derrota TMS. A reimplementação em TypeScript visa:

- **Acessibilidade** — TypeScript é a linguagem mais usada no ecossistema de agentes de IA.
- **Integração nativa** — MCP servers, CLIs e bibliotecas JS/TS são o padrão de facto.
- **Velocidade de iteração** — sem compilação, com hot-reload e tipagem gradual.
- **Distribuição** — `npm install` em vez de compilar binário.

---

## 2. Vantagens e desvantagens

### 2.1 Vantagens

| Aspecto | Detalhe |
|---|---|
| **Ecossistema** | `commander`, `zod`, `fast-glob`, `minisearch`, `natural` (BM25), `graphlib` (SCC/PageRank) |
| **MCP nativo** | `@modelcontextprotocol/sdk` é TypeScript — integração direta |
| **Tipagem** | TypeScript captura invariantes em compile-time (enums, union types) |
| **Testes** | `vitest` com propriedades (fast-check) e snapshot testing |
| **Distribuição** | `npm`/`npx` — zero compilação, binário via `pkg` ou `bun build` |
| **Debugging** | Source maps, REPL, inspect em runtime |
| **Onboarding** | Curva de contribuição menor que Rust |

### 2.2 Desvantagens

| Aspecto | Detalhe |
|---|---|
| **Performance** | ~2–5× mais lento que Rust em hot paths (BM25, MinHash, PageRank) |
| **Memória** | GC pauses, overhead de objetos JS |
| **Determinismo** | `Map`/`Set` preservam inserção, mas `Object.keys()` não garante ordem — precisa de `Map` ou arrays ordenados |
| **Concorrência** | Single-threaded — sem paralelismo real (exceto worker_threads) |
| **Binário** | `pkg`/`bun` geram binários grandes (~50–100 MB) |
| **Segurança de tipos** | `any` é tentador — precisa de `strict: true` e lint rigoroso |
| **FS atómico** | `fs.rename` é atómico em POSIX, mas `fsync` não é direto em Node |
| **Lock advisory** | `fs.open(O_CREAT|O_EXCL)` funciona, mas stale lock detection é mais frágil |

### 2.3 Veredito

**Recomendado** se o público-alvo é o ecossistema JS/TS de agentes de IA e a prioridade é
velocidade de desenvolvimento e integração. **Não recomendado** se a prioridade é performance
em corpus grandes (>10k notas) ou binário pequeno/estático.

---

## 3. Plano mestre

### Fase 0 — Fundação (2 semanas)

- [ ] Setup do monorepo (pnpm workspaces, TypeScript strict, ESLint, Prettier)
- [ ] Estrutura de packages: `@knudge/core`, `@knudge/cli`, `@knudge/mcp`
- [ ] Portas (interfaces) para `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`
- [ ] Fakes para testes: `FixedClock`, `SeqRng`, `MemFs`, `FakeGit`, `RecordingLogger`
- [ ] Utilitários: `sha256`, `base36`, `normalize` (NFD + fold), `levenshtein`

### Fase 1 — Modelo de dados (2 semanas)

- [ ] `Note` type com frontmatter TOON
- [ ] Parser/serializador TOON (raw UTF-8, ordem canônica, 31 chaves)
- [ ] Enums: `NoteType` (10), `Scope` (3), `Classification` (3), `Status` (6), `EdgeKind` (12)
- [ ] Identidade derivada: `id = <prefixo>_<base36(8)>` com `SHA-256(type + U+001F + normalize(statement))`
- [ ] Schema v2: `claims` (SPO), `provenance` (PROV-lite), ontologia leve
- [ ] Validação com `zod` (runtime) + tipos TS (compile-time)

### Fase 2 — Persistência (2 semanas)

- [ ] Escrita atómica: tmp + rename + `fsync`
- [ ] Lock advisory por arquivo-alvo (`O_CREAT|O_EXCL`, stale 30 s)
- [ ] Rebuild double-buffer: `.idx.new/` → rename
- [ ] Event log: `eventos/events.jsonl` (append-only)
- [ ] Purge/detach/sweep: remoção de notas e limpeza de derivados

### Fase 3 — Busca e ranking (3 semanas)

- [ ] Tokenização: ASCII → NFD → stopwords PT+EN → stemming PT conservador
- [ ] BM25: `k1=1.5`, `b=0.75`, IDF Robertson-Sparck Jones, pesos por campo
- [ ] MinHash (64 perm.) + LSH (16 bandas × 4 linhas) para dedup
- [ ] Dice exato como peneira final
- [ ] RRF: `Σ peso_c/(k+rank+1)`, `k=60`, pesos `lexical 1.0`, `anchor 2.0`, `semantic 30.0`, `ppr 0.0`
- [ ] PageRank/PPR: iteração de potência, damping `0.85`, ≤32 iter, tol L1 `1e-8`
- [ ] Louvain determinístico (≤8 níveis, ≤64 passos)
- [ ] Confiança Beta-Bernoulli + Wilson (limite inferior 95%, `Z=1.96`)
- [ ] `why` fechado: `file_match > anchor_match > tracker_match > stars > semantic > recent > universal`

### Fase 4 — Escrita e dedup (2 semanas)

- [ ] Duas fases: recall lexical → decisão (`<0.75` cria, `0.75–0.92` merge, `≥0.92` rejeita)
- [ ] Merge: união de tags/âncoras/outcomes, `revision++`, corpo não sobrescrito
- [ ] Update/supersede: `Patch` versionado, `replaces`/`superseded_by` bidirecional
- [ ] Lote: `write --batch -` (JSONL), best-effort com `warnings[]`

### Fase 5 — Grafo e inferência (2 semanas)

- [ ] Arestas explícitas = chaves de frontmatter de 1º nível
- [ ] Extração textual como sugestão (`.idx/suggestions.jsonl`), nunca aresta
- [ ] Integridade: referências quebradas, órfãos, bidirecionalidade
- [ ] SCC (Kosaraju) sobre `replaces`; membros de ciclo não demovem
- [ ] TMS/defeasible: `retracted`, `defeated_dependents`, `defeated_by_replacement`

### Fase 6 — Ciclo de vida (2 semanas)

- [ ] Shelf-life derivado da `classification` (foundational/tactical/observational)
- [ ] FSRS-like: `retention.growth_percent=50%`, `retention(t)=limiar^(t/ttl)`, `limiar=0.5`
- [ ] Decay/drift de âncoras: fração válida, demove após grace se `< threshold`
- [ ] Drift de termos: JS (base 2, smoothing 1e-9), propõe revisão se `JS ≥ 0.5` e `≥ 4` notas
- [ ] Demolição: só propõe (`expired`/`anchor_decay`/`contradicted`/`defeated`/`drifted`)

### Fase 7 — Tarefas (2 semanas)

- [ ] Tarefa = view derivada de notas (mesmo formato/store)
- [ ] Hierarquia fechada: `epic ⊃ { issue ⊃ task | task }`, pai único, profundidade ≤ 4
- [ ] Derivado: papel, modo, progresso, `epic_of`, impacto, contexto, fluxo
- [ ] Status: `active/in_progress/blocked/closed`
- [ ] Ops: `new/list/show/update/close/graph/plan/flow`

### Fase 8 — Embeddings (2 semanas)

- [ ] Provedor `http` (OpenAI-compatible local) / `lightweight` (hash) / `none` (BM25)
- [ ] Índice `.idx/embeddings.jsonl` com `meta`
- [ ] Cache por `(body_hash, model)` com LRU
- [ ] Estado `indexed|pending|stale` derivado
- [ ] Modo `lazy` (default) / `manual`
- [ ] Worker: `scripts/knudge-idle.sh` + `kd drain service`

### Fase 9 — CLI (2 semanas)

- [ ] 14 verbos: `init|prime|rewind|ask|write|task|map|maintenance|doctor|drain|config|forget|sync|self`
- [ ] Envelope `--json`: `{success, command, data?, error?, warnings?}`
- [ ] stdout = dados; stderr = logs
- [ ] Listas: repetição ≡ vírgula (`--tag a,b`)
- [ ] Conjuntos fechados: valor inválido → lista + "você quis dizer" (Levenshtein)
- [ ] Exit codes: `0` ok · `2` invalid usage · `3` not found · `4` conflict · `5` io · `6` timeout · `7` bad config · `8` bad schema/note · `70` internal
- [ ] EPIPE → exit 0

### Fase 10 — MCP (1 semana)

- [ ] Servidor MCP (JSON-RPC 2.0 sobre stdio) com `@modelcontextprotocol/sdk`
- [ ] 4 gatilhos: `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`

### Fase 11 — Testes e qualidade (2 semanas)

- [ ] 883+ testes unitários (vitest)
- [ ] `fast-check` nas funções puras (propriedades)
- [ ] Goldens para o contrato de bytes (TOON, ids, hashes)
- [ ] Cobertura ≥ 90%
- [ ] Benchmarks: BM25, MinHash, PageRank vs. Rust

### Fase 12 — Documentação e release (1 semana)

- [ ] `README.md` — instalação + quickstart
- [ ] `SKILL.md` — guia para agentes
- [ ] `AGENTS.md` — contribuir no código
- [ ] `wiki/usage/` — guias de uso
- [ ] `wiki/specs/` — especificações técnicas
- [ ] `wiki/specs/adrs/` — decisões D01–D212
- [ ] `llms.txt` — índice canónico para LLMs
- [ ] Publicação no npm

---

## 4. Issues encadeadas sequencialmente

### Épico 0: Fundação

#### Issue 0.1 — Setup do monorepo
**Descrição:** Criar estrutura pnpm workspaces com TypeScript strict, ESLint, Prettier, e scripts
de build/test/lint.

**Critérios de aceitação:**
- [ ] `pnpm install` funciona
- [ ] `pnpm build` compila todos os packages
- [ ] `pnpm test` roda testes
- [ ] `pnpm lint` não tem erros
- [ ] TypeScript `strict: true` em todos os packages

**Dependências:** nenhuma

---

#### Issue 0.2 — Definir portas (interfaces)
**Descrição:** Criar interfaces TypeScript para `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`,
`Logger` em `@knudge/core`.

**Critérios de aceitação:**
- [ ] Interfaces documentadas com JSDoc
- [ ] Métodos assíncronos onde necessário
- [ ] Tipos de retorno explícitos
- [ ] Sem dependências de Node.js (apenas tipos)

**Dependências:** 0.1

---

#### Issue 0.3 — Implementar fakes para testes
**Descrição:** Criar `FixedClock`, `SeqRng`, `MemFs`, `FakeGit`, `RecordingLogger`, `FakeEmbedder`
em `@knudge/core/testing`.

**Critérios de aceitação:**
- [ ] `FixedClock` retorna tempo fixo
- [ ] `SeqRng` retorna valores em sequência
- [ ] `MemFs` simula filesystem em memória
- [ ] `FakeGit` simula operações git
- [ ] `RecordingLogger` captura logs para asserções
- [ ] `FakeEmbedder` retorna vetores determinísticos

**Dependências:** 0.2

---

#### Issue 0.4 — Utilitários criptográficos e de string
**Descrição:** Implementar `sha256`, `base36`, `normalize` (NFD + fold), `levenshtein`.

**Critérios de aceitação:**
- [ ] `sha256` usa `crypto` do Node.js
- [ ] `base36` converte bigint para string base36
- [ ] `normalize` aplica NFD e remove diacríticos
- [ ] `levenshtein` implementa distância de edição
- [ ] Testes com vetores conhecidos

**Dependências:** 0.1

---

### Épico 1: Modelo de dados

#### Issue 1.1 — Tipo `Note` e enums
**Descrição:** Definir tipos TypeScript para `Note`, `NoteType`, `Scope`, `Classification`,
`Status`, `EdgeKind`.

**Critérios de aceitação:**
- [ ] `Note` é um tipo com frontmatter e corpo
- [ ] Enums como `const` objects (não `enum` nativo)
- [ ] Union types para valores válidos
- [ ] JSDoc com descrição de cada campo

**Dependências:** 0.4

---

#### Issue 1.2 — Parser/serializador TOON
**Descrição:** Implementar parser e serializador TOON (raw UTF-8, ordem canônica, 31 chaves).

**Critérios de aceitação:**
- [ ] Parser aceita TOON válido e rejeita inválido
- [ ] Serializador produz TOON canónico
- [ ] Ordem das chaves é determinística
- [ ] Testes com goldens (contrato de bytes)

**Dependências:** 1.1

---

#### Issue 1.3 — Identidade derivada
**Descrição:** Implementar `id = <prefixo>_<base36(8)>` com `SHA-256(type + U+001F + normalize(statement))`.

**Critérios de aceitação:**
- [ ] `computeId(type, statement)` retorna id estável
- [ ] Reclassificar `type` não reescreve o `id`
- [ ] Testes com exemplos conhecidos

**Dependências:** 1.2

---

#### Issue 1.4 — Schema v2 (claims, provenance, ontologia)
**Descrição:** Adicionar suporte a `claims` (SPO), `provenance` (PROV-lite), e ontologia leve
(`same_as`/`broader`/`narrower`/`related`).

**Critérios de aceitação:**
- [ ] `claims` é array de `{subject, predicate, object}`
- [ ] `provenance` segue PROV-lite
- [ ] Ontologia leve é opcional
- [ ] Validação com `zod`

**Dependências:** 1.3

---

### Épico 2: Persistência

#### Issue 2.1 — Escrita atómica
**Descrição:** Implementar escrita atómica: tmp + rename + `fsync`.

**Critérios de aceitação:**
- [ ] `writeAtomic(path, content)` cria tmp, escreve, fsync, rename
- [ ] Falha no meio não deixa arquivo corrompido
- [ ] Testes com simulação de crash

**Dependências:** 1.4

---

#### Issue 2.2 — Lock advisory
**Descrição:** Implementar lock advisory por arquivo-alvo (`O_CREAT|O_EXCL`, stale 30 s).

**Critérios de aceitação:**
- [ ] `acquireLock(path)` cria `.lock` com PID e timestamp
- [ ] Lock stale (>30 s) é reclaimable
- [ ] `releaseLock(path)` remove `.lock`
- [ ] Testes com locks concorrentes

**Dependências:** 2.1

---

#### Issue 2.3 — Rebuild double-buffer
**Descrição:** Implementar rebuild double-buffer: `.idx.new/` → rename.

**Critérios de aceitação:**
- [ ] Rebuild escreve em `.idx.new/`
- [ ] Swap é atômico (rename)
- [ ] Leitor vê antigo ou novo, nunca parcial
- [ ] Testes com leitura durante rebuild

**Dependências:** 2.1

---

#### Issue 2.4 — Event log
**Descrição:** Implementar event log append-only em `eventos/events.jsonl`.

**Critérios de aceitação:**
- [ ] `appendEvent(event)` adiciona linha JSON
- [ ] Ordem de commit: nota primeiro, evento depois
- [ ] Crash deixa nota sem evento (recuperável)
- [ ] Testes com replay de eventos

**Dependências:** 2.1

---

#### Issue 2.5 — Purge/detach/sweep
**Descrição:** Implementar remoção de notas com purge de derivados e detach de arestas.

**Critérios de aceitação:**
- [ ] `purgeNote(id)` remove nota e derivados
- [ ] `detachEdges(id)` remove arestas referenciando a nota
- [ ] `sweep()` remove `*.tmp`/`*.stale` > 30 s
- [ ] Testes com grafo de dependências

**Dependências:** 2.3, 2.4

---

### Épico 3: Busca e ranking

#### Issue 3.1 — Tokenização
**Descrição:** Implementar tokenização: ASCII → NFD → stopwords PT+EN → stemming PT conservador.

**Critérios de aceitação:**
- [ ] Tokenizador normaliza para lowercase
- [ ] Remove stopwords PT e EN
- [ ] Aplica stemming PT conservador
- [ ] Testes com exemplos conhecidos

**Dependências:** 1.4

---

#### Issue 3.2 — BM25
**Descrição:** Implementar BM25 com `k1=1.5`, `b=0.75`, IDF Robertson-Sparck Jones.

**Critérios de aceitação:**
- [ ] Índice invertido por campo
- [ ] Pesos: `statement 3.0`, `tags 2.0`, `body 1.0`
- [ ] Boost por confirmação `×(1+0.1·conf)`
- [ ] Testes com corpus pequeno

**Dependências:** 3.1

---

#### Issue 3.3 — MinHash + LSH
**Descrição:** Implementar MinHash (64 perm.) + LSH (16 bandas × 4 linhas) para dedup.

**Critérios de aceitação:**
- [ ] MinHash com 64 permutações
- [ ] LSH com 16 bandas × 4 linhas
- [ ] Candidatos passam por Dice exato
- [ ] Testes com pares similares/dissimilares

**Dependências:** 3.1

---

#### Issue 3.4 — RRF
**Descrição:** Implementar Reciprocal Rank Fusion com pesos `lexical 1.0`, `anchor 2.0`,
`semantic 30.0`, `ppr 0.0`.

**Critérios de aceitação:**
- [ ] `rrf(ranks, weights, k=60)` retorna score fusionado
- [ ] Desempate `(score desc, id asc)`
- [ ] Testes com rankings parciais

**Dependências:** 3.2, 3.3

---

#### Issue 3.5 — PageRank/PPR
**Descrição:** Implementar PageRank/PPR com damping `0.85`, ≤32 iter, tol L1 `1e-8`.

**Critérios de aceitação:**
- [ ] Iteração de potência
- [ ] Convergência por tolerância L1
- [ ] PPR com seed nodes
- [ ] Testes com grafos pequenos

**Dependências:** 1.4

---

#### Issue 3.6 — Louvain
**Descrição:** Implementar Louvain determinístico (≤8 níveis, ≤64 passos).

**Critérios de aceitação:**
- [ ] Otimização de modularidade
- [ ] Determinístico (mesma entrada → mesma saída)
- [ ] Testes com grafos conhecidos

**Dependências:** 1.4

---

#### Issue 3.7 — Confiança Beta-Bernoulli + Wilson
**Descrição:** Implementar confiança com posterior `Beta(1+s,1+f)` e limite inferior 95% (Wilson,
`Z=1.96`).

**Critérios de aceitação:**
- [ ] `confidence(successes, failures)` retorna limite inferior
- [ ] Outcomes: `success=1`, `partial=0.5`, `failure/abandoned=1 falha`
- [ ] Testes com exemplos conhecidos

**Dependências:** 1.4

---

#### Issue 3.8 — `why` e revelação progressiva
**Descrição:** Implementar `why` fechado e revelação progressiva de snippets.

**Critérios de aceitação:**
- [ ] `why` segue ordem: `file_match > anchor_match > tracker_match > stars > semantic > recent > universal`
- [ ] 1º hit: corpo completo
- [ ] 2–5: truncado a `preview_chars=280`
- [ ] Resto: `id|statement|score|why`

**Dependências:** 3.4, 3.7

---

### Épico 4: Escrita e dedup

#### Issue 4.1 — Duas fases de escrita
**Descrição:** Implementar recall lexical de candidatos → decisão (`<0.75` cria, `0.75–0.92`
merge, `≥0.92` rejeita).

**Critérios de aceitação:**
- [ ] Recall usa BM25 + MinHash
- [ ] Decisão por limiar de similaridade
- [ ] Testes com notas similares/dissimilares

**Dependências:** 3.2, 3.3

---

#### Issue 4.2 — Merge
**Descrição:** Implementar merge: união de tags/âncoras/outcomes, `revision++`, corpo não
sobrescrito.

**Critérios de aceitação:**
- [ ] Merge preserva corpo original
- [ ] União de tags e âncoras
- [ ] `revision` incrementa
- [ ] Testes com notas sobrepostas

**Dependências:** 4.1

---

#### Issue 4.3 — Update/supersede
**Descrição:** Implementar `Patch` versionado com `replaces`/`superseded_by` bidirecional.

**Critérios de aceitação:**
- [ ] Chave nova → novo id
- [ ] `replaces` e `superseded_by` são bidirecionais
- [ ] Testes com cadeias de substituição

**Dependências:** 4.2

---

#### Issue 4.4 — Lote
**Descrição:** Implementar `write --batch -` (JSONL) e `task new --batch`, best-effort com
`warnings[]`.

**Critérios de aceitação:**
- [ ] Processa JSONL linha a linha
- [ ] Falha em uma linha não para o lote
- [ ] `warnings[]` reporta erros parciais
- [ ] Testes com lote misto (válido/inválido)

**Dependências:** 4.1

---

### Épico 5: Grafo e inferência

#### Issue 5.1 — Arestas explícitas
**Descrição:** Implementar extração de arestas de chaves de frontmatter de 1º nível.

**Critérios de aceitação:**
- [ ] Arestas são extraídas de `depends_on`, `replaces`, etc.
- [ ] Extração textual é sugestão, nunca aresta
- [ ] Sugestões em `.idx/suggestions.jsonl`
- [ ] Testes com notas com/sem arestas

**Dependências:** 1.4

---

#### Issue 5.2 — Integridade do grafo
**Descrição:** Implementar verificação de integridade: referências quebradas, órfãos,
bidirecionalidade.

**Critérios de aceitação:**
- [ ] Detecta referências quebradas
- [ ] Detecta órfãos
- [ ] Verifica bidirecionalidade `replaces ↔ superseded_by`
- [ ] Testes com grafos inválidos

**Dependências:** 5.1

---

#### Issue 5.3 — SCC (Kosaraju)
**Descrição:** Implementar SCC (Kosaraju) sobre `replaces`; membros de ciclo não demovem.

**Critérios de aceitação:**
- [ ] Encontra componentes fortemente conectados
- [ ] Membros de ciclo são protegidos
- [ ] Testes com grafos cíclicos

**Dependências:** 5.2

---

#### Issue 5.4 — TMS/defeasible
**Descrição:** Implementar TMS/defeasible: `retracted`, `defeated_dependents`,
`defeated_by_replacement`.

**Critérios de aceitação:**
- [ ] `retracted` marca premissas `forgotten`/`superseded`
- [ ] `defeated_dependents` é transitivo via `depends_on` reverso
- [ ] `defeated_by_replacement` marca derrotados
- [ ] Testes com cenários de derrota

**Dependências:** 5.3

---

### Épico 6: Ciclo de vida

#### Issue 6.1 — Shelf-life
**Descrição:** Implementar shelf-life derivado da `classification` (foundational/tactical/
observational).

**Critérios de aceitação:**
- [ ] `foundational` nunca expira
- [ ] `tactical` expira em 365 dias
- [ ] `observational` expira em 30 dias
- [ ] Testes com notas de cada classificação

**Dependências:** 1.4

---

#### Issue 6.2 — FSRS-like
**Descrição:** Implementar FSRS-like: `retention.growth_percent=50%`, `retention(t)=limiar^(t/ttl)`,
`limiar=0.5`.

**Critérios de aceitação:**
- [ ] Cada sucesso estende prazo em 50%
- [ ] Relógio reseta após sucesso
- [ ] `retention(t)` calcula retenção
- [ ] Testes com simulação de uso

**Dependências:** 6.1

---

#### Issue 6.3 — Decay/drift de âncoras
**Descrição:** Implementar decay/drift de âncoras: fração válida, demove após grace se `< threshold`.

**Critérios de aceitação:**
- [ ] Calcula fração de âncoras válidas
- [ ] Demove após grace se `< threshold`
- [ ] `drift = 1−fração`
- [ ] `drift_factor = 1−0.5·drift ∈ (0.5,1]`
- [ ] Testes com âncoras válidas/inválidas

**Dependências:** 6.1

---

#### Issue 6.4 — Drift de termos
**Descrição:** Implementar drift de termos: JS (base 2, smoothing 1e-9), propõe revisão se
`JS ≥ 0.5` e `≥ 4` notas.

**Critérios de aceitação:**
- [ ] Calcula JS divergence entre metade antiga/nova
- [ ] Proõe revisão se `JS ≥ 0.5` e `≥ 4` notas
- [ ] Testes com vocabulário estável/mudado

**Dependências:** 6.3

---

#### Issue 6.5 — Demolição
**Descrição:** Implementar demolição: só propõe (`expired`/`anchor_decay`/`contradicted`/
`defeated`/`drifted`).

**Critérios de aceitação:**
- [ ] Propõe demolição com motivo
- [ ] Membros de ciclo protegidos
- [ ] Aplicação via `kd forget`
- [ ] Testes com notas expiradas

**Dependências:** 6.2, 6.3, 6.4

---

### Épico 7: Tarefas

#### Issue 7.1 — Tarefa como view derivada
**Descrição:** Implementar tarefa como view derivada de notas (mesmo formato/store).

**Critérios de aceitação:**
- [ ] Tarefa é derivada de notas
- [ ] Mesmo formato/store
- [ ] `scope` = nível (`epic`/`issue`/`task`)
- [ ] `type` = espécie (`task` default; `--kind` = `task error question risk decision`)

**Dependências:** 1.4

---

#### Issue 7.2 — Hierarquia fechada
**Descrição:** Implementar hierarquia fechada: `epic ⊃ { issue ⊃ task | task }`, pai único,
profundidade ≤ 4.

**Critérios de aceitação:**
- [ ] Épico é a raiz
- [ ] Issue é opcional
- [ ] Pai único
- [ ] Profundidade ≤ 4
- [ ] Testes com hierarquias válidas/inválidas

**Dependências:** 7.1

---

#### Issue 7.3 — Derivados de tarefa
**Descrição:** Implementar derivados: papel, modo, progresso, `epic_of`, impacto, contexto, fluxo.

**Critérios de aceitação:**
- [ ] Derivados são calculados, nunca armazenados
- [ ] `epic_of` encontra épico raiz
- [ ] Impacto calcula dependências
- [ ] Testes com hierarquias complexas

**Dependências:** 7.2

---

#### Issue 7.4 — Status e fechamento
**Descrição:** Implementar status `active/in_progress/blocked/closed` e fechamento com evidência.

**Critérios de aceitação:**
- [ ] Status é derivado
- [ ] `task close` exige evidência
- [ ] Testes com transições de status

**Dependências:** 7.3

---

#### Issue 7.5 — Ops de tarefa
**Descrição:** Implementar `new/list/show/update/close/graph/plan/flow`.

**Critérios de aceitação:**
- [ ] Todos os verbos funcionam
- [ ] `--ready`/`--blocked`/`--explain`/`--sort impact`
- [ ] `task plan --prompt|--submit` é atômico
- [ ] Testes com cada verbo

**Dependências:** 7.4

---

### Épico 8: Embeddings

#### Issue 8.1 — Provedor HTTP
**Descrição:** Implementar provedor `http` (OpenAI-compatible local, endpoint
`http://127.0.0.1:8889/v1/embeddings`).

**Critérios de aceitação:**
- [ ] Cliente HTTP para OpenAI-compatible
- [ ] Modelo default: `granite-embedding-97m-multilingual-r2` (384d)
- [ ] Testes com servidor mock

**Dependências:** 0.4

---

#### Issue 8.2 — Provedor lightweight
**Descrição:** Implementar provedor `lightweight` (hash, CI) e `none` (BM25).

**Critérios de aceitação:**
- [ ] `lightweight` usa hash determinístico
- [ ] `none` desativa embeddings
- [ ] Testes com cada provedor

**Dependências:** 8.1

---

#### Issue 8.3 — Índice de embeddings
**Descrição:** Implementar índice `.idx/embeddings.jsonl` com `meta`.

**Critérios de aceitação:**
- [ ] Índice é append-only
- [ ] `meta` registra modelo e dimensão
- [ ] Mudança de modelo invalida e re-embeda
- [ ] Testes com mudança de modelo

**Dependências:** 8.1

---

#### Issue 8.4 — Cache de embeddings
**Descrição:** Implementar cache por `(body_hash, model)` com LRU.

**Critérios de aceitação:**
- [ ] Cache é versionável em `.knudge/emb_cache.jsonl`
- [ ] LRU evita crescimento ilimitado
- [ ] Testes com cache hit/miss

**Dependências:** 8.3

---

#### Issue 8.5 — Estado e modo
**Descrição:** Implementar estado `indexed|pending|stale` derivado e modo `lazy`/`manual`.

**Critérios de aceitação:**
- [ ] Estado é derivado, nunca armazenado
- [ ] Falha marca `pending` (nunca descarta)
- [ ] Modo `lazy` drena 1 lote best-effort após cada verbo
- [ ] Modo `manual` exige `kd drain`
- [ ] Testes com cada modo

**Dependências:** 8.4

---

#### Issue 8.6 — Worker
**Descrição:** Implementar worker: `scripts/knudge-idle.sh` + `kd drain service`.

**Critérios de aceitação:**
- [ ] Script de idle detecta inatividade
- [ ] `kd drain service` roda em background
- [ ] GGUF de revisão pinada + SHA-256
- [ ] Testes com worker

**Dependências:** 8.5

---

### Épico 9: CLI

#### Issue 9.1 — Verbos base
**Descrição:** Implementar 14 verbos: `init|prime|rewind|ask|write|task|map|maintenance|doctor|
drain|config|forget|sync|self`.

**Critérios de aceitação:**
- [ ] Todos os verbos funcionam
- [ ] `kd` = `help`
- [ ] Testes com cada verbo

**Dependências:** 2.5, 4.4, 5.4, 6.5, 7.5, 8.6

---

#### Issue 9.2 — Envelope `--json`
**Descrição:** Implementar envelope `--json`: `{success, command, data?, error?, warnings?}`.

**Critérios de aceitação:**
- [ ] stdout = dados; stderr = logs
- [ ] `--json` retorna envelope
- [ ] Testes com cada verbo

**Dependências:** 9.1

---

#### Issue 9.3 — Listas e conjuntos fechados
**Descrição:** Implementar listas (repetição ≡ vírgula) e conjuntos fechados (valor inválido →
lista + "você quis dizer").

**Critérios de aceitação:**
- [ ] `--tag a,b` = `--tag a --tag b`
- [ ] Valor inválido → lista + sugestão (Levenshtein)
- [ ] Testes com valores válidos/inválidos

**Dependências:** 9.2

---

#### Issue 9.4 — Exit codes
**Descrição:** Implementar exit codes: `0` ok · `2` invalid usage · `3` not found · `4` conflict
· `5` io · `6` timeout · `7` bad config · `8` bad schema/note · `70` internal.

**Critérios de aceitação:**
- [ ] Cada erro mapeia para exit code correto
- [ ] EPIPE → exit 0
- [ ] Testes com cada exit code

**Dependências:** 9.3

---

### Épico 10: MCP

#### Issue 10.1 — Servidor MCP
**Descrição:** Implementar servidor MCP (JSON-RPC 2.0 sobre stdio) com
`@modelcontextprotocol/sdk`.

**Critérios de aceitação:**
- [ ] Servidor responde a JSON-RPC 2.0
- [ ] 4 gatilhos: `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`
- [ ] Testes com cliente MCP

**Dependências:** 9.4

---

### Épico 11: Testes e qualidade

#### Issue 11.1 — Testes unitários
**Descrição:** Implementar 883+ testes unitários (vitest).

**Critérios de aceitação:**
- [ ] Cobertura ≥ 90%
- [ ] Testes para cada módulo
- [ ] Testes com fakes (portas)

**Dependências:** 10.1

---

#### Issue 11.2 — Testes de propriedade
**Descrição:** Implementar `fast-check` nas funções puras.

**Critérios de aceitação:**
- [ ] Propriedades para `sha256`, `base36`, `normalize`, `levenshtein`
- [ ] Propriedades para BM25, MinHash, RRF, PageRank
- [ ] Testes com geradores aleatórios

**Dependências:** 11.1

---

#### Issue 11.3 — Goldens
**Descrição:** Implementar goldens para o contrato de bytes (TOON, ids, hashes).

**Critérios de aceitação:**
- [ ] Goldens para TOON
- [ ] Goldens para ids
- [ ] Goldens para hashes
- [ ] Testes com goldens

**Dependências:** 11.2

---

#### Issue 11.4 — Benchmarks
**Descrição:** Implementar benchmarks: BM25, MinHash, PageRank vs. Rust.

**Critérios de aceitação:**
- [ ] Benchmarks para BM25
- [ ] Benchmarks para MinHash
- [ ] Benchmarks para PageRank
- [ ] Comparação com Rust

**Dependências:** 11.3

---

### Épico 12: Documentação e release

#### Issue 12.1 — Documentação base
**Descrição:** Implementar `README.md`, `SKILL.md`, `AGENTS.md`.

**Critérios de aceitação:**
- [ ] `README.md` — instalação + quickstart
- [ ] `SKILL.md` — guia para agentes
- [ ] `AGENTS.md` — contribuir no código

**Dependências:** 11.4

---

#### Issue 12.2 — Wiki
**Descrição:** Implementar `wiki/usage/`, `wiki/specs/`, `wiki/specs/adrs/`.

**Critérios de aceitação:**
- [ ] `wiki/usage/` — guias de uso (um por comando)
- [ ] `wiki/specs/` — especificações técnicas
- [ ] `wiki/specs/adrs/` — decisões D01–D212

**Dependências:** 12.1

---

#### Issue 12.3 — `llms.txt`
**Descrição:** Implementar `llms.txt` — índice canónico para LLMs.

**Critérios de aceitação:**
- [ ] Índice canónico
- [ ] Links para documentação
- [ ] Testes com LLM

**Dependências:** 12.2

---

#### Issue 12.4 — Publicação no npm
**Descrição:** Publicar packages no npm.

**Critérios de aceitação:**
- [ ] `@knudge/core` publicado
- [ ] `@knudge/cli` publicado
- [ ] `@knudge/mcp` publicado
- [ ] `npx @knudge/cli` funciona

**Dependências:** 12.3

---

## 5. Dependências entre épicos

```
Épico 0 (Fundação)
  └─> Épico 1 (Modelo de dados)
       └─> Épico 2 (Persistência)
            └─> Épico 3 (Busca e ranking)
                 ├─> Épico 4 (Escrita e dedup)
                 ├─> Épico 5 (Grafo e inferência)
                 ├─> Épico 6 (Ciclo de vida)
                 ├─> Épico 7 (Tarefas)
                 └─> Épico 8 (Embeddings)
                      └─> Épico 9 (CLI)
                           └─> Épico 10 (MCP)
                                └─> Épico 11 (Testes e qualidade)
                                     └─> Épico 12 (Documentação e release)
```

---

## 6. Estimativa de esforço

| Épico | Semanas | Issues |
|---|---|---|
| 0 — Fundação | 2 | 4 |
| 1 — Modelo de dados | 2 | 4 |
| 2 — Persistência | 2 | 5 |
| 3 — Busca e ranking | 3 | 8 |
| 4 — Escrita e dedup | 2 | 4 |
| 5 — Grafo e inferência | 2 | 4 |
| 6 — Ciclo de vida | 2 | 5 |
| 7 — Tarefas | 2 | 5 |
| 8 — Embeddings | 2 | 6 |
| 9 — CLI | 2 | 4 |
| 10 — MCP | 1 | 1 |
| 11 — Testes e qualidade | 2 | 4 |
| 12 — Documentação e release | 1 | 4 |
| **Total** | **25** | **58** |

---

## 7. Riscos e mitigações

| Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|
| Performance inferior ao Rust | Alta | Médio | Benchmarks precoces; otimizar hot paths |
| Determinismo quebrado | Médio | Alto | Testes de propriedade; goldens |
| GC pauses em corpus grandes | Médio | Médio | Usar `Map`/`Set`; evitar objetos temporários |
| Dependências npm instáveis | Médio | Baixo | Pinatar versões; vendor críticos |
| Curva de aprendizado de TypeScript strict | Baixo | Médio | Documentação; exemplos |
| Mudanças de API do MCP | Médio | Médio | Abstração; testes de integração |

---

## 8. Conclusão

A reimplementação do knudge em TypeScript é viável e traz vantagens significativas de
acessibilidade e integração com o ecossistema JS/TS de agentes de IA. O plano mestre proposto
cobre 12 épicos, 58 issues, e estima 25 semanas de trabalho. Os principais riscos são
performance e determinismo, que devem ser mitigados com benchmarks precoces e testes de
propriedade.

---

*Documento gerado em 2025-10-15 a partir de `exploration/knudge/resumo.md` (v0.5.4).*
