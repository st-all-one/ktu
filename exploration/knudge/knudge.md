# knudge — resumo técnico para investigação

> **Fonte:** `REF/knudge` (workspace Rust, versão **0.5.4**, toolchain Rust 1.97+/edição 2024).
> Documento de orientação para quem estuda o sistema: *que problema resolve, com que
> invariantes, com que algoritmos, e onde estão as decisões e as provas.*
> Estrutura de referência no repo: `wiki/specs/` (técnico), `wiki/specs/adrs/` (decisões `Dxx`),
> `plan/03_decisoes-fechadas.md` (contrato canónico), `bench/` (avaliação), `crates/` (código).

---

## 1. Problema e enquadramento

**knudge** dá **memória durável *por projeto* a agentes de IA** onde o código vive: notas Markdown
versionadas ao lado do repositório, consultáveis **antes** de agir e atualizadas **depois** de
aprender. Os sintomas que ataca (`wiki/usage/01_filosofia.md`):

- **Contaminação de conhecimento** — duplicatas e informação contraditória acumulam; o agente
  "aprende" a coisa errada.
- **Falhas de processo** — decisões moram em conversas que ninguém relê; trabalho recomeça do zero.
- **Depreciação** — o que era verdade há seis meses continua a ser respondido como atual.
- **Vida útil do projeto** — o repositório dura anos; o conhecimento que o explica dura uma sessão.

**Requisitos de desenho (não-funcionais) declarados:** sem servidor, sem base de dados, sem daemon
obrigatório; um binário `kd`; **LLM opcional em tudo** (busca/escrita/`prime`/auditoria funcionam
sem modelo); tudo em texto simples versionável por git; núcleo puro testável. É uma **CLI**
(pacote de binário) e um **servidor MCP** (JSON-RPC 2.0 sobre stdio) sobre uma biblioteca
publicável (`knudge-core`, `publish = true`, D214).

**Posicionamento face a RAG:** o sistema **não injeta conteúdo** — injeta **ponteiros**
(`id + statement + score`). O agente decide o que puxar (`prime`/`rewind`/hints MCP). É uma
postura de *memory-augmented agent* com retrieval explícito, não de pipeline RAG
(`plan/brainstorm/6.md`, `wiki/specs/mcp.md`).

---

## 2. Princípios de arquitetura

### 2.1 Duas camadas: verdade × derivado (invariante central)

| Camada | Onde | Estatuto | Versionável |
|---|---|---|---|
| **Verdade** | `notas/<tipo>/<id>.md` + `eventos/events.jsonl` | fonte da verdade; Markdown + frontmatter TOON | sim (git) |
| **Derivado** | `.idx/` (índice, âncoras, drift, uso, sugestões, checkpoint), `cache/`, `contexts/` | **descartável e reconstruível** | não |

Apagar `.idx/` **nunca** perde conhecimento; o índice é reconstruído de `notas/`. Há uma exceção
opt-in: o cache vetorial é versionável (`.knudge/emb_cache.jsonl`, D148), porque um vetor é função
pura de `(modelo, body_hash)`.

### 2.2 Núcleo puro + portas + adaptadores (D65)

O domínio (`knudge-core`) **não** conhece terminal, `argv`, relógio global, RNG global nem FS.
Todo acesso ao mundo passa por **portas** (`knudge_core::ports`), com fakes para teste:

| Porta | Real | Fake |
|---|---|---|
| `Clock` | relógio UTC (ms) | `FixedClock` |
| `Rng` | jitter | `SeqRng` |
| `Env` | variáveis/args | `FakeEnv` |
| `Fs` | I/O atómico | `MemFs`, `FaultyFs` (injeta crash) |
| `Git` | worktree/status sem shell | `FakeGit` |
| `HookRunner` | hooks externos (timeout + kill de grupo) | `NoopHookRunner` |
| `Logger` | log estruturado (stderr) | `RecordingLogger` |
| `Embedder` | HTTP/`lightweight` | `FakeEmbedder` |

Layout de crates: `knudge-core` (puro) ← `knudge-cli` (binário `kd`) e `knudge-mcp` (servidor).
Escopos temáticos do core: `error, time, logging, ports, adapters, config, schema, toon, jsonl,
store, git, graph, retrieval, write, handoff, maintenance, task, health, lifecycle, embeddings`.

### 2.3 Disciplina de implementação

- `#![forbid(unsafe_code)]` nos crates puros; arquivos de produção **≤ 300 linhas** (gate no
  `make check`); proibidos `unwrap`/`expect`/`panic` em `src/`.
- `HashMap`/`HashSet`/`LinkedList` **proibidos** no core (via `clippy.toml`): só `BTreeMap`/
  `IndexMap` para ordem canónica.
- Políticas de engenharia **R01–R43** (`plan/implementation/14_revisao_tecnica.md`): memória,
  backpressure, logs com redação de segredos, erros (`thiserror` no core / `anyhow` na CLI),
  runtime mínimo (sem `tokio full`).

---

## 3. Modelo de dados

### 3.1 A nota e a sua identidade endereçada por conteúdo

Uma nota = frontmatter **TOON** (ordem canónica) + corpo Markdown. Sem nota parcial; opcionais
**omitidos**, nunca `null` (D05).

- `normalize(s)` = NFC + trim + colapso de whitespace.
- **Id** = `<prefixo>_<base36(8)>` = `hash(type + U+001F + normalize(statement))` (D01/D95).
  **Reclassificar `type` não reescreve o id** (o prefixo é histórico).
- `body_hash` = `hex8(SHA-256(normalize(statement) + LF + normalize(body)))` — cobre statement+corpo.
- Hash curto = SHA-256 truncado aos 4 primeiros bytes (`u32` big-endian).

Consequência para merge distribuído: a mesma afirmação gera o mesmo `id` em qualquer máquina →
notas **mergeáveis por git**; conflito real (mesma chave, corpos divergentes) é **reportado**, nunca
auto-mergeado (D153).

### 3.2 Schema e enums fechados

- **31 chaves canónicas** (ordem é contrato): `id, type, statement, created_at, body_hash,
  schema_version, tags, source, superseded_by, references, depends_on, contradicts, supports,
  extends, replaces, rejects, results_in, same_as, broader, narrower, related, revision, outcomes,
  classification, anchors, status, scope, checks, evidence, claims, provenance`.
- `REQUIRED_KEYS` = `{id, statement, created_at, body_hash, schema_version}` (5); `type` omitido
  quando `scope=epic`.
- **Enums fechados:** `NoteType` (10: `fact decision question task def error snippet link meta
  risk`), `Scope` (`epic issue task`), `Classification` (`foundational tactical observational`),
  `Status` (`active in_progress blocked closed superseded forgotten`), `EdgeKind` (12). Chave
  desconhecida → rejeita no write, warning no read; tipo desconhecido → rejeita a nota.
- **Extensões aditivas (D207, `SCHEMA_VERSION=2`):** `claims` (SPO `{subject, relation, object}`),
  `provenance` (PROV-lite `{entity, activity, agent}`) e ontologia leve
  (`same_as/broader/narrower/related`).
- **Data contract *soft*** (D191): slots mínimos de corpo por espécie (`decision`→Alternativas/Por
  quê/Consequência; `error`→Causa/Correção; `risk`→Probabilidade/Impacto; …) — aviso em `write`,
  erro só sob `strict`.

### 3.3 TOON — contrato de bytes (`wiki/specs/TOON.md`)

Subconjunto documentado de TOON com **canónico na emissão, tolerante na leitura** (Postel). Raw
UTF-8 sem BOM; comentário `#` fora de aspas; aninhamento ≤ 32; inteiros nunca saem `.0`; escapes
`\n\r\t\0\u{}`; lista/mapa vazio omitido; documento vazio → 0 bytes; `LF` final. **A ordem é
contrato** (D04/D13) — travada por *golden* + `proptest` de round-trip byte-exato.

---

## 4. Pipeline de retrieval (`wiki/specs/busca.md`, fórmulas em `matematica.md`)

Cascata determinística:

```
filtros → BM25 → âncoras → vetorial → PPR → fusão RRF → penalidade de contradição
        → confiança/idade/drift → snippet/why
```

Canal ausente **degrada com `warnings`** (nunca quebra — R33).

### 4.1 BM25 (por campo)

```
score(d,q) = type_weight(type_d) · (1 + 0.1·confirmação_d)
           · Σ_f  w_f · Σ_{t∈q} IDF_f(t) · tf·(k1+1) / (tf + k1·norm_f(d))
```

- `k1 = 1.5`, `b = 0.75`; pesos de campo `statement 3.0 / tags 2.0 / body 1.0`.
- IDF de Robertson-Sparck Jones **por campo** (`ln_1p`).
- `type_weight`: `decision 1.20, error 1.15, fact 1.10, def/risk 1.05, task/question 1.00,
  snippet 0.95, link/meta 0.90, epic 0.70`.
- Boost por confirmação derivada: `×(1 + 0.1·conf)` (D38/D108).
- **Corte de alta frequência** (D173): termo descartado se `df/N ≥ 0.9` **e** `N ≥ 64` — o RRF usa
  ranks, e um termo quase universal produziria rank-1 espúrio.

### 4.2 Tokenização

ASCII → **fold de diacríticos** (NFD; `café ≡ cafe`; termo inteiro — D172) → stopwords PT+EN +
fragmentos de 1 char → **stemming PT conservador** (radical ≥ 4 bytes — D206). `normalize`/`id`/
`body_hash` seguem **NFC** (não o fold).

### 4.3 Reciprocal Rank Fusion (D81/D123/D179)

```
RRF(x) = Σ_c  peso_c / (k + r_c(x) + 1),   k = 60
```

Pesos default: `lexical 1.0`, `anchor 2.0`, `semantic 30.0`, `ppr 0.0`. O `semantic = 30`
*Pareto-domina* o neutro no corpus PT-BR medido; `anchor = 2.0` corrige empates. Desempate
determinístico `(score desc, id asc)`.

### 4.4 Outros canais e modos

- **Âncoras:** canal por path/glob (`GlobPattern`); `ask --anchor` busca **sem query textual**;
  casa nas duas direções; `doctor` lista âncoras quebradas.
- **PageRank/PPR** (D192): iteração de potência, `damping 0.85`, ≤ 32 iter, tol L1 `1e-8`; canal
  `ppr` default desligado.
- **Temporal** (D155): `ask --as-of <TS>` reconstrói o conjunto ativo em `T` a partir do log de
  eventos (`forget`/`restore`, `link replaces`) e corre o pipeline sobre esse subconjunto.
- **Views:** `ready`/`blocked` do `depends_on` transitivo; ciclo ⇒ `blocked`.
- **Saída:** pipe `id|statement|score|why`; **revelação progressiva** (D161): 1º hit com corpo
  completo, 2–5 truncado a `preview_chars=280`, resto `id|statement|score|why`; `--brief` desliga.
- **`why` fechado:** `file_match > anchor_match > tracker_match > stars > semantic > recent >
  universal`; `--json` traz `channels` (parcelas RRF/boost).

---

## 5. Protocolo de escrita e dedup (`wiki/specs/escrita-e-dedup.md`)

**Idempotência por conteúdo:** reescrever a mesma afirmação não duplica (mesmo `id`).

**Dedup em duas fases** (D26/D80):

```
fase 1: recall lexical de candidatos
decisão:  score < 0.75        → cria
          0.75 ≤ score < 0.92 → merge
          score ≥ 0.92        → rejeita
```

- **Lexical no write**, semântico *eventual* (nunca bloqueia — D80).
- **Dedup escalável** (D204): corpus pequeno/esparso → peneira exata de *postings*
  (byte-idêntica); > 256 notas densas → **MinHash (64 permutações) + LSH (16 bandas × 4 linhas)**,
  com verificação Dice exata dos candidatos. Propõe idêntico ou superconjunto (recall ≥).
- **Merge:** união de tags/âncoras/outcomes, `revision++`, sem sobrescrever corpo; `confidence`
  **nunca** armazenada.
- **Update/supersede:** `Patch` versionado; chave nova → novo `id` + `replaces`/`superseded_by`
  (bidirecional).
- **Ciclo de vida soft:** `forget`/`restore` mudam só `status` (nada é apagado); transições validadas.

---

## 6. Grafo e inferência (`wiki/specs/grafo.md`)

- **Arestas explícitas** = chaves de frontmatter de 1º nível (lista de ids); 12 tipos. A extração
  textual é **sugestão** em `.idx/suggestions.jsonl`, nunca aresta (D49/D50).
- **Integridade/ciclos:** referências quebradas, órfãos, bidirecionalidade; SCC (Kosaraju) sobre
  `replaces`; membros de ciclo **não demovem** (D45).
- **PageRank/PPR** (D192): arestas de autoridade
  (`references/supports/extends/replaces`); *dangling* redistribuído.
- **Comunidades** (D193): **Louvain determinístico** (≤ 8 níveis, ≤ 64 passos; empate mantém a
  comunidade) sobre arestas + âncoras partilhadas; `map --communities`.
- **Ontologia** (D207): classes de equivalência (`same_as`), clausura `broader`/`narrower`, deteção
  de ciclos de hierarquia, `claim_conflicts` (mesmo `(subject, relation)` com objetos divergentes).
- **TMS *defeasible*** (D208): `retracted` (premissas `forgotten`/`superseded`),
  `defeated_dependents` (transitivo via `depends_on` reverso), `defeated_by_replacement`. Sem
  apagar — motivo `Defeated` no `prune`.

---

## 7. Tarefas (`wiki/specs/tarefas.md`)

- **Tarefa = view derivada de notas** (mesmo formato/store). `scope` = nível (`epic/issue/task`);
  `type` = espécie (`task` default; `--kind` = `task error question risk decision`).
- Hierarquia **fechada:** `epic ⊃ { issue ⊃ task | task }`; épico é a raiz (sem `type`), issue
  opcional; pai por rank estritamente menor; profundidade ≤ 4.
- **Duas camadas:** marcador durável no corpo `<!-- knudge:parent <id> blocks <n> -->` + aresta
  `results_in` derivada.
- **Derivados (nunca armazenados):** papel (`error`→Bug, `question`→Spike, `risk`→Risk,
  `decision`→Decision; senão `depth 0`→Epic, `1+filhos`→Feature, `1`→Story, `≥2`→SubTask), modo
  (`incremental`→Magentic; `cadeia`→Sequential; senão Concurrent), progresso (folhas fechadas),
  `epic_of`, impacto (itens abertos dependentes transitivos).
- **Fecho exige evidência:** `kd task close --outcome <success|partial|failure|abandoned>`.
- **`task flow`** (D205): cycle/lead/throughput + **caminho crítico PERT/CPM**.
- **Escopo obrigatório** (D144): `task list`/`map`/`maintenance learn|compact|prune`/`ask --rank`
  exigem filtro ou `--universe`, senão exit 2.

---

## 8. Ciclo de vida, confiança e retenção (`wiki/specs/ciclo-de-vida.md`)

- **Shelf-life derivado da `classification`** (D135): `foundational` nunca expira; `tactical` (365
  dias) e `observational` (30).
- **Curva FSRS-like** (D190): cada `outcome` de sucesso estende o prazo em
  `retention.growth_percent` (default **50 %**) e reseta o relógio
  (`origin = max(created, último ensaio, último uso se renew_on_use)`);
  `retention(t) = limiar^(t/ttl)`, `limiar = 0.5`.
- **Renovação por uso** (D154): uso (citação) é derivado `.idx/usage.jsonl`, nunca verdade; **só
  estende**.
- **Confiança Beta-Bernoulli + Wilson** (D189): `outcomes` = ensaios (`success=1`, `partial=0.5`,
  `failure/abandoned`=1 falha); posterior `Beta(1+s, 1+f)`; a média alimenta `stars`; o **limite
  inferior 95 %** (Wilson, `Z=1.96`) é a confiança conservadora.
  `confidence_score = base + lower_bound + feedback(0.2) + tarefa + recência(≤0.05)`, `clamp01`,
  `· drift_factor`.
- **Decay/drift de âncoras** (D43/D203): fração válida → `drift = 1 − fração`;
  `drift_factor = 1 − 0.5·drift ∈ (0.5, 1]`; persistido em `.idx/drift.jsonl`.
- **Drift de termos** (D208): divergência **JS (base 2, smoothing 1e-9)** entre metade antiga/nova
  por tópico; propõe revisão se `JS ≥ 0.5` e `≥ 4` notas.
- **Clusters:** fase 1 estrutural (eixo anchor/type/classification/scope); fase 2 semântica
  **complete-link** *off-path*.
- **Demolição:** **só propõe** (`Expired`/`AnchorDecay`/`Contradicted`/`Defeated`/`Drifted`); ciclos
  protegidos; aplicação via `kd forget`. **Nada é removido antes da janela de retenção.**

---

## 9. Embeddings (opcional) (`wiki/specs/embeddings.md`)

- **Provedor plugável** (D79/D101/D202): `http` (default; servidor local **OpenAI-compatible**,
  endpoint `http://127.0.0.1:8889/v1/embeddings`), `lightweight` (hash determinístico, CI/offline),
  `none` (cai para BM25). Cliente **HTTP/1.1 bloqueante sobre `std::net`** (sem `tokio`/`reqwest`);
  **sem inferência in-process**.
- Modelo default `ibm-granite/granite-embedding-97m-multilingual-r2` (384d); `revision` pinada —
  mudar modelo **invalida** o índice.
- Índice `.idx/embeddings.jsonl` com `meta` (provider/model/revision/dimensões/similaridade). Cache
  por `(body_hash, model)` com LRU; versionável em `.knudge/emb_cache.jsonl` (D148, sem *eviction*).
- **Estado derivado** `indexed|pending|stale`; falha marca `pending` (nunca descarta). Modo `lazy`
  (default; drena 1 lote best-effort após cada verbo, D131) ou `manual`.
- **Sugestões semânticas** (D158): classifica pares em `duplicate`/`contradiction`/`link` —
  *advisory*, nunca aresta, persistido em `.idx/suggestions.jsonl`.
- **Worker:** `scripts/knudge-idle.sh` (fonte da verdade) + `kd drain service` (wrapper fino,
  D184/D186); agenda via `systemd --user`/`launchd`/cron; GGUF de revisão pinada + SHA-256 (D183).

---

## 10. MCP e handoff

- **`knudge-mcp`**: servidor MCP local e reativo, **JSON-RPC 2.0 sobre stdio**, *framing* de uma
  linha JSON por mensagem (sem `Content-Length`). `PROTOCOL_VERSION = "2025-06-18"` com lista de
  versões suportadas. 4 tools: `knudge_pre_write` (quase-duplicados na escrita),
  `knudge_pre_edit` (working set antes de editar ficheiro), `knudge_session_end` (`learn`),
  `knudge_status`. `HintEngine` é **puro**; hints são **ponteiros** (`id + statement + score`),
  com *cap* e modo observação.
- **`prime`** = protocolo estático **byte-idêntico por versão** ("help da IA"), compacto por
  default (D166). **`rewind`** = estado dinâmico: modos `Manifest`/`Scope`/`Files`/`Auto`, com
  `next:` (tarefas `ready` por impacto) e `fresh:` (stale/expiring/pending), orçamento
  `ceil(len/4)` tokens (`DEFAULT_BUDGET=4000`); `--resume <context_id>` devolve bytes idênticos
  (`.idx/contexts/`).

---

## 11. Saúde, manutenção e governação

- **`kd doctor`** (13 checks + auditoria): `healthy` só com **zero achados**; `--fix` corrige o
  reversível (body_hash, âncoras quebradas, locks stale, duplicatas, layout); `--explain` detalha
  (`esperado`/`encontrado`/`ação`).
- **`maintenance learn|compact|prune`** — **read-only**, só **propõem**: `learn`
  (`create_note/merge/supersede/link`, inclui tarefa fechada sem nota), `compact`
  (`concat/keep_latest/merge_outcomes`), `prune` (`forget|id|motivo` por shelf-life/decay).
  Aplicação sempre explícita via `write`/`write --link`/`forget`.
- **Portão de evidência** (D156): `learn`/`compact --verify` anexam `gate=passed|failed`;
  `proposals.enforce=true` faz o pré-registo bloquear com `conflict` (exit 4).
- **Redação tipada de segredos** no log (D159): `«REDACTED:tipo»`; o knudge não persiste corpo
  redigido.
- **Regras governadas** (D157): `kd config promote` promove notas elegíveis a regras num bloco
  `<!-- knudge:rules -->` do `AGENTS.md`, com teto rígido (`rules.max_promoted`); desligado por
  default; nunca auto-edita.

---

## 12. Contrato de máquina, determinismo e testes

### 12.1 Contrato de CLI

- **stdout = dados; stderr = logs.** `--json` = `{success, command, data?, error?, warnings?}` —
  nenhum log vaza para stdout.
- **Exit codes:** `0` ok · `2` invalid_input · `3` not_found · `4` conflict · `5` io · `6` timeout
  · `7` config · `8` schema · `70` internal · `101` reservado a panic.
- Busca vazia → `(no_results]` (exit 0); **EPIPE → exit 0** (D73).
- 14 verbos de topo: `init prime rewind ask write task map maintenance doctor drain config forget
  sync self` (+ `help`; `kd` sozinho = `kd help`).
- **Listas:** repetição ≡ vírgula (D210); texto livre nunca dividido; `--params '<json>'` para
  arrays. **Conjuntos fechados** (D212): valor inválido → lista + sugestão Levenshtein.

### 12.2 Determinismo como requisito

A mesma entrada produz a mesma saída **byte a byte**. Ordem canónica obrigatória; tempo/RNG/Env/FS
só por portas; *tie-breaks* explícitos em todo o lado; PageRank/Louvain/MinHash determinísticos;
pontos flutuantes com `total_cmp`, sem RNG. Views estáticas (`compute_views`) byte-idênticas; a
dinâmica (`compute_views_at`) usa `now_ms` só onde o tempo importa.

### 12.3 Estratégia de verificação

- **Fakes** no core (nunca toca o SO); `FaultyFs` injeta crash.
- **`proptest`** nas funções puras; **goldens** para o contrato de bytes; **stress** para
  concorrência/crash (leitor nunca vê índice parcial).
- **`DIVERGENCES.md`** (#1–#115): catálogo de **bordas** onde a implementação poderia divergir de
  si mesma (Unicode/NFC, contagem de escalares, hash, ordem, atomicidade, ordem nota/evento, lock,
  rotação de eventos, determinismo de iteração, EPIPE, framing MCP, …) — **cada linha com
  mitigação e o teste que a trava**. É o substituto do *harness* diferencial que o projeto não tem.
- ~883 testes + proptest; gate `make check` (fmt + `clippy -D warnings` + test + ≤ 300 linhas).

### 12.4 Persistência e crash-safety

Escrita de nota `tmp + rename` no mesmo diretório (D20), `fsync` em lote (D22); **ordem nota antes
do evento** (D21) — crash deixa nota sem evento (recuperável). Lock *advisory* por ficheiro-alvo
(`O_CREAT|O_EXCL`, stale 30 s, *reclaim* por rename, ordem para evitar deadlock ABBA — D23–D25).
Rebuild **double-buffer** `.idx.new/` + rename (D27) — leitor vê antigo ou novo, nunca parcial.

---

## 13. Catálogo de decisões e evidência

- **Decisões `D01–D209+`** (uma por ficheiro em `wiki/specs/adrs/`; contrato canónico em
  `plan/03_decisoes-fechadas.md`; atlas em `llms.txt`). Grupos: identidade/IDs, schema/bytes,
  versionamento, escrita/crash, concorrência, git, retrieval, ciclo de vida, grafo, tarefas,
  config, arquitetura, contrato, TOON, testes, e a vaga 0.5.0 (D172–D212: fold de diacríticos,
  Beta-Bernoulli, FSRS, PageRank/PPR, Louvain, MinHash/LSH, stemming PT, claims/ontologia, TMS/drift
  e a superfície v3).
- **Avaliação** (`bench/`): bancada determinística (micro + e2e) com baselines congelados
  (`ULTIMO-v0.5.0.*`); **qualidade** de retrieval (Recall@k / MRR / nDCG@k, E16/T01) em
  `qualidade.md`. O relatório documenta os gargalos quadráticos atacados (doctor/compact O(N²) →
  MinHash/LSH, −94 %; varredura O(N) do auto-drain; reconstrução de índice por comando).
- **Bordas numéricas:** `DIVERGENCES.md` #101/#103/#105/#107/#109/#111.

---

## 14. Ângulos de investigação (leitura crítica)

Pontos que merecem estudo, não só uso:

1. **Dedup em dois estágios com limiares calibrados (0.75/0.92)** — como se comporta sob corpora
   PT-BR ruidosos? MinHash/LSH preserva *recall* ≥ da peneira exata (proposta testada), mas a
   fronteira exato↔aproximado (256 notas) pode ser tema de estudo.
2. **Pesos de fusão (`semantic = 30`)** — Pareto-domina o neutro no corpus medido (D179); quão
   sensível é o ranking a estes pesos fora do corpus de calibração?
3. **Determinismo como contrato** — o custo de impor ordem canónica global vs ganhos em
   reprodutibilidade/auditabilidade; impacto em performance.
4. **Confiança derivada (Beta + Wilson) + FSRS + drift** — combinação pouco comum num sistema de
   memória de projeto; comparar com *memory decay* baseado em frequência pura.
5. **MCP só com ponteiros vs RAG com conteúdo** — hipótese de "contexto preservado, agente decide";
   mede-se em qualidade de resposta e custo de tokens.
6. **Duplo-alvo humano × máquina** — o pipeline é LLM-first; não há deteção de TTY, cor ou pager
   (ver `plan/proposals/usabilidade_humana.md`). Estudo de UX para operação humana é território
   aberto.
7. **TMS *defeasible* + ontologia leve em Markdown versionado** — até onde a inferência sobre
   notas (claims/retração) é suficiente antes de precisar de uma representação mais expressiva.

---

## 15. Mapa de leitura rápida no repo

| Para… | Ver |
|---|---|
| Visão e princípios | `README.md`, `wiki/usage/01_filosofia.md`, `plan/00_panorama.md` |
| Camadas e módulos | `wiki/specs/ARCHITECTURE.md` |
| Contrato de bytes | `wiki/specs/TOON.md` |
| Fórmulas (BM25/RRF/Beta/FSRS/PPR/Louvain/LSH/PERT) | `wiki/specs/matematica.md` |
| Bordas + testes que as travam | `wiki/specs/DIVERGENCES.md` |
| Retrieval | `wiki/specs/busca.md` |
| Escrita/dedup | `wiki/specs/escrita-e-dedup.md` |
| Grafo/TMS/ontologia | `wiki/specs/grafo.md` |
| Tarefas/fluxo | `wiki/specs/tarefas.md` |
| Ciclo de vida/confiança | `wiki/specs/ciclo-de-vida.md` |
| Embeddings | `wiki/specs/embeddings.md` |
| Determinismo/testes | `wiki/specs/determinismo.md` |
| Decisões (`Dxx`) | `wiki/specs/adrs/`, `plan/03_decisoes-fechadas.md` |
| Avaliação | `bench/RELATORIO.md`, `bench/qualidade.md` |
| Implementação por épico | `plan/implementation/` (`00`–`24`, `16_cli_surface.md`) |
