---
question_id: Q3
same_session: true
question_issue: "Escreva o resumo do /REF/knudge, agora adaptado para um pesquisador, o resumo deve ficar em /exploration/knudge, em um único arquivo .md" 
expected_answer_ideia: "Deve ser capaz de listar categoricamente os algoritmos de otimização, modelo toon v3 de dados e usabilidade geral via kd write, kd task e kd ask, com a orientação clara de primeiro usar o ask, para evitar escrita desnecessária"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: goose 
---

# knudge — Síntese técnica para pesquisador

**Objeto:** `/REF/knudge` (no disco: `/home/one/p/ktu/REF/knudge`) · **versão** `0.5.2` · **commit** `2ef623c`
**Linguagem:** Rust (edition 2024, `rustc 1.97+`) · **Licença:** MIT OR Apache-2.0 · **Distribuição:** binários `kd` e `knudge-mcp` + crate publicável `knudge-core`.

> Documento de leitura única. Sintetiza README, `llms.txt`, `plan/`, `wiki/specs/` (contrato de bytes, matemática, busca, escrita, ciclo de vida, grafo, determinismo), `bench/` e o código-fonte. Toda fórmula e constante abaixo foi extraída de `wiki/specs/matematica.md` e confrontada com os módulos citados.

---

## 1. Abstract

O **knudge** é um sistema de **memória durável por projeto** para agentes de LLM, implementado como CLI offline (`kd`) + servidor MCP (`knudge-mcp`), sem servidor, banco, daemon ou nuvem. A tese é uma **inversão de prioridades**: em vez de otimizar armazenamento e depois "encaixar" o LLM, otimiza-se o **contexto injetado** e trata-se armazenamento/índice como **detalhe derivado e descartável**.

O design entrega três propriedades que interessam a um pesquisador:

1. **Fonte da verdade legível e versionável** — uma afirmacão autocontida por arquivo Markdown com frontmatter *TOON* (Token-Oriented Object Notation), endereçado por conteúdo.
2. **Índice estritamente derivado** — busca híbrida (léxica + âncoras + vetorial + grafo) reconstruível *do zero* a partir das notas; apagar `.idx/` nunca perde conhecimento.
3. **Determinismo byte-a-byte** — sem RNG global, sem `HashMap`, ordem canônica em toda agregação; a mesma entrada produz a mesma saída, travada por *golden tests*, *proptest*, *miri* e *fuzz*.

---

## 2. Problema e tese

**Problema.** A memória de um projeto de código não vive na cabeça do dev nem no prompt: a sessão termina e o contexto evapora; o conhecimento registrado se espalha em duplicatas e contradições; o que era verdade há seis meses continua sendo respondido como se fosse hoje.

**Tese central do repo (`plan/00_panorama.md` §1):**

> **"O usuário é o LLM. Não há humano lendo ou editando. Legibilidade humana é irrelevante; tokenização é tudo."**
> **"O custo é contexto, não disco."**
> **"O índice é derivado."**

Três consequências que explicam praticamente todas as decisões:

- **Otimizar contexto, não storage:** a nota é *uma afirmação curta*; o corpo é só o que a afirmação não diz. Sem título/heading/preâmbulo.
- **Enums fechados:** tipos abertos degradam o retrieval (o LLM inventa categorias).
- **Formato de bytes determinístico:** o sistema é *byte-sensitive*; TOON reduz 40–62 % dos tokens vs JSON em estruturas de objetos.

> ⚠️ **Observação de pesquisa:** o README, a `wiki/usage/` e o `init` **são escritos para humanos**, enquanto a arquitetura é desenhada para o LLM. Essa tensão é documentada e auditada no próprio repo (`plan/proposals/usabilidade_humana.md`) e é onde surgem divergências de contrato — ver §11.

---

## 3. Arquitetura

### 3.1 Crates e camadas (portas e adaptadores, D65)

```
knudge-mcp ──┐
             ├── knudge-core (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

| Crate | Papel | Conteúdo |
|---|---|---|
| `knudge-core` | Modelo, schema, retrieval, ciclo de vida, **portas**; fachada `Knudge` (D214, `publish = true`) | Lógica pura; `adapters` (std) isolado |
| `knudge-cli` | Binário `kd` | `clap`, `tracing`, I/O de terminal |
| `knudge-mcp` | Servidor MCP reativo (gatilhos + JSON-RPC sobre stdio) | Nada de domínio |

**Princípio:** o domínio **não conhece** terminal, `argv`, relógio global, RNG global nem filesystem. Todo acesso externo atravessa uma **porta** (`ports`: `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`, `Embedder`), com *fakes* (`FixedClock`, `SeqRng`, `MemFs`, `FaultyFs`, `FakeGit`, …) para teste reprodutível byte-a-byte e confinamento de `unsafe`/I/O impuro. `#![forbid(unsafe_code)]`, sem `unwrap`/`expect` em produção, aritmética `checked_*`/`saturating_*`, indexação `[]` negada (D92, R01).

### 3.2 Layout de dados (`.knudge/`)

| Camada | Papel | Estabilidade |
|---|---|---|
| `notas/<tipo>/<id>.md` | **Fonte da verdade** — frontmatter TOON + corpo Markdown | Fixa |
| `eventos/events.jsonl` | Log append-only (auditoria; rotação `events-NNNN.jsonl`) | Recomendada |
| `.idx/` | Índice derivado (retrieval, checkpoint, anchors, drift, embeddings, clusters) | Oscila |
| `config.toml` | Config efetiva do projeto (precedência sobre global) | Formato fixo |
| `cache/`, `emb_cache.jsonl`, `usage.jsonl`, `drift.jsonl` | Derivados descartáveis | Descartável |

**Regra de ouro:** `notas/` + `eventos/` reconstroem `.idx/` do zero. O índice **nunca** é fonte da verdade.

---

## 4. Modelo de dados e identidade endereçada por conteúdo

Nota = **frontmatter TOON** + corpo Markdown. "Sem nota parcial": ou é válida, ou não existe; opcionais são **omitidos** (nunca `null`).

### 4.1 Hash e identidade (D01, D06, D95)

| Elemento | Fórmula |
|---|---|
| `normalize(s)` | NFC + trim + colapso de whitespace |
| `body_hash` | `hex8(SHA-256(normalize(statement) ‖ LF ‖ normalize(body)))` |
| `id` | `<prefixo>_<base36(8)>(SHA-256(type ‖ U+001F ‖ normalize(statement)))` |
| hash curto | `SHA-256` truncado aos **4 primeiros bytes** (`u32` big-endian) |

**Consequências (relevantes para replicação):**
- **Idempotência sob retry:** a mesma afirmação gera o mesmo `id` em qualquer máquina → reescrever não duplica.
- **Prefixo histórico:** reclassificar o `type` **não** reescreve o `id`; a linhagem fica via `superseded_by`.
- `body_hash` cobre statement + corpo → edição invalida vetores/índice.
- **NFC no `id`**, mas *fold de acentos* (D172) e *stemming* (D206) afetam **apenas o índice derivado** — `notas/` nunca muda.

### 4.2 Contrato de bytes (TOON, D74/D75)

Subconjunto documentado e **canônico na emissão, tolerante na leitura** (Postel). Regras travadas por golden e proptest:
- Raw UTF-8, sem BOM; nada de `null` (opcional ausente = omitido); **ordem canônica** de 31 chaves é contrato (D04/D13).
- Escalares tipados (`Bool`/`Int`/`Float`/`Str`); inteiros **nunca** saem com `.0` (`1.0 → 1`, D09). `emit(parse(x)) == x`.
- Aninhamento máx. 32 níveis (proteção contra input hostil); chave duplicada é erro.
- `schema_version` atual `2` (D207), lido *on-read com defaults*, sem aliases; notas v1 seguem válidas e **byte-idênticas** no rebuild.

### 4.3 Enums fechados

| Enum | Valores |
|---|---|
| `NoteType` (10) | `fact`, `decision`, `question`, `task`, `def`, `error`, `snippet`, `link`, `meta`, `risk` (`epic` é **derivado** de `scope=epic`, D149) |
| `Scope` | `epic`, `issue`, `task` |
| `Classification` | `foundational`, `tactical`, `observational` |
| `Status` | `active`, `in_progress`, `blocked`, `closed`, `superseded`, `forgotten` |
| `EdgeKind` (12) | `references`, `depends_on`, `contradicts`, `supports`, `extends`, `replaces`, `rejects`, `results_in`, `same_as`, `broader`, `narrower`, `related` |

**Extensões semânticas (D207):** ontologia leve SKOS-lite (`same_as`/`broader`/`narrower`/`related`, com inversos e simetria), `claims` (triplas SPO `{subject, relation, object}`) para contradição **precisa**, e `provenance` PROV-lite `{entity, activity, agent}`. Todas **aditivas** (bump 1→2).

---

## 5. Núcleo algorítmico

> Tudo é `f64` determinístico, sem RNG; comparações usam `total_cmp` (ordem total IEEE-754); `NaN → 0`; valores são **derivados**, nada é persistido (D87).

### 5.1 BM25 com IDF por campo (D35–D38, D173)

Score de `d` para os termos `q`, somado por campo `f ∈ {statement, body, tags}`:

```
score(d,q) = type_weight(type_d) · (1 + 0.1·confirmação_d)
           · Σ_f w_f · Σ_{t∈q} IDF_f(t) · tf_f(t,d)(k1+1) / (tf_f(t,d) + k1·norm_f(d))
```

- `k1 = 1.5`, `b = 0.75`; `norm_f(d) = 1 − b + b·len_f(d)/avg_len_f`.
- Pesos de campo `w_f`: `statement = 3.0`, `tags = 2.0`, `body = 1.0`.
- **IDF de Robertson–Sparck Jones por campo:** `IDF_f(t) = ln(1 + (N − df_f(t) + 0.5)/(df_f(t) + 0.5))`.
- **Peso por tipo** (D37): `decision 1.20`, `error 1.15`, `fact 1.10`, `def/risk 1.05`, `task/question 1.00`, `snippet 0.95`, `link/meta 0.90`, `epic 0.70`.
- **Corte de alta frequência** (D173): descarta termo com `df/N ≥ max_term_ratio` (default `0.9`) **apenas se** `N ≥ 64`. Justificativa seminal: o RRF usa *ranks*; um termo quase universal produziria um rank-1 espúrio.
- `df(t)` usa o **maior** `df` entre campos (conservador).

### 5.2 Tokenização (D36/D122/D172/D206)

Cadeia do canal lexical: ASCII `\w` → **fold de diacríticos** (NFD + descarte de marcas; `café ≡ cafe`, via `unicode-normalization`) → stopwords PT+EN e fragmentos de 1 char → **stemming PT conservador** (radical mín. 4 bytes, plural antes do derivacional). O índice persiste o **radical**; o snippet usa **termos crus**. `INDEX_FORMAT = retrieval-v4` invalida índices antigos.

### 5.3 Fusão RRF e pesos (D81/D123/D179)

Para rank 0-based `r_c(x)` no canal `c`:

```
RRF(x) = Σ_c peso_c / (k + r_c(x) + 1),   k = 60
```

| Canal | Peso default | Racional |
|---|---|---|
| lexical | `1.0` | baseline |
| âncoras | `2.0` | corrige empate "match exato de âncora" × "rank-1 lexical ruidoso" |
| vetorial | `30.0` | comprime ranks em corpus pequeno (Pareto-domina o neutro em PT-BR) |
| PPR | `0.0` | **desligado** (só compensa com muitas arestas de autoridade) |

`rrf_k = 60` foi medido **inerte** no corpus rotulado; o lever real é `anchor_weight` (D179). **Desempate:** `(score desc, id asc)`. Canal ausente **não quebra** (degradação graciosa, R33; `strict` promove warning a erro, D94).

### 5.4 Confiança derivada: Beta-Bernoulli + limite de Wilson (D87/D189/D203)

`outcomes` viram ensaios de Bernoulli (`success = 1`, `partial = 0.5`, `failure/abandoned = 1` falha). Com prior `Beta(1,1)`, posterior `Beta(1+s, 1+f)`; usa-se o **limite inferior** do intervalo de Wilson a 95 % (`z = 1.96`), conservador: **1 sucesso ≈ 0,21** vs **20 sucessos ≈ 0,84**. `O(1)`, sem dependência.

```
score = clamp01( (base + evidência + feedback + tarefa + recência) · drift_factor(drift) )
base      = similaridade · age_factor(idade),  age_factor(a) = 1/(1 + a/90)   (meia-vida 90 d)
evidência = lower_bound(s, f)          feedback = 0.2 · max(feedback_explícito, 0)
recência  = 0.05 · age_factor(idade) · (1 − similaridade)     (D175; só desempata)
drift_factor(d) = 1 − 0.5·d            ∈ [0.5, 1]             (D203)
penalidade de contradição = max(confidence − 0.1, 0)          (D177, lado perdedor)
```

Monotonicidade verificada por proptest (crescente em similaridade/sucessos/feedback; decrescente em falhas/drift/idade).

### 5.5 Retenção FSRS-like (D190)

Shelf-life derivado da `classification`; `foundational` nunca expira. Curva exponencial:

```
R(t) = limiar^(t/ttl) = exp(−t/S),   S = ttl / ln(1/limiar)     (limiar default 0.5)
ttl_efetivo = base · (1 + growth% · reviews/100)                (growth% = 50)
```

Cada sucesso estende o prazo e **reseta a origem** (`max(created_at, último ensaio, último uso se renew_on_use)`). Renovação por uso (D154) usa `usage.jsonl` derivado.

### 5.6 Drift de âncoras e de termos

**Âncoras (D43/D86/D203):** `drift = 1 − V/(V+B)` (V válidas, B quebradas); `drift_factor = 1 − 0.5·drift`. Persistido em `.idx/drift.jsonl`; ausente ⇒ `drift = 0`.

**Termos (D208):** por tópico (âncora), divide por mediana de `created_at` em janela antiga/nova e compara distribuições de termos por **KL** (base 2, suavização `1e-9`) e **Jensen–Shannon**:
```
KL(P‖Q) = Σ_t P(t)·log2(P(t)/Q(t))     JS(P,Q) = ½KL(P‖M) + ½KL(Q‖M), M=(P+Q)/2 ∈ [0,1]
```
Propõe revisão quando `JS ≥ 0.5` **e** o tópico tem ≥ 4 notas.

### 5.7 Grafo: PageRank/PPR, Louvain, ontologia, TMS

- **PageRank/PPR (D192):** iteração de potência sobre arestas de autoridade, `d = 0.85`, `r₀ = 1/n`, massa *dangling* redistribuída por `p(v)` (soma preservada = 1), ≤32 iterações, tol. L1 `1e-8`, ordem canônica (invariante a relabeling — proptest). PPR semeado pelo *working set* alimenta o canal `ppr` (default 0,0).
- **Louvain determinístico (D193):** ganho `ΔQ_{i→c} ∝ w_{i,c} − k_i·Σ_tot(c)/(2m)`; *local moving* empatando **mantém** a comunidade corrente (`EPSILON = 1e-9`); agregação com auto-laço ×2; ≤8 níveis, ≤64 passos. Função pura do grafo.
- **Ontologia leve:** `equivalence_classes` (clausura de `same_as`), `broader_ancestors`/`narrower_descendants`, detecção de ciclo hierárquico, `claim_conflicts` (mesma `(subject, relation)`, objetos divergentes).
- **TMS/defeasible (D208):** `retracted`, `defeated_dependents` (transitivos), `defeated_by_replacement` — o lado substituído é **derrotado, não removido**.

### 5.8 Dedup escalável: MinHash/LSH (D204)

Usado quando o corpus é grande e denso (`> 256` notas) — a peneira exata degenera em O(N²).

```
base = fnv1a(t);  h_i(t) = splitmix64(base ⊕ (i·φ)),  φ = 0x9E3779B97F4A7C15
sig_i(S) = min_{t∈S} h_i(t)        (SIGNATURE_LEN = 64, i ∈ 0..64)
banding: BANDS = 16 × ROWS = 4     →  P(colisão) = 1 − (1 − s⁴)^16
```

Propriedade de MinHash `P(sig_i(S₁)=sig_i(S₂)) = J(S₁,S₂)`. Para `s = 0.92` (limiar de merge), `P ≈ 1`. Os candidatos ainda passam pelo **Dice exato** e pelo limiar → propostas são **idênticas ou superconjunto** (recall ≥, nunca perdem par real). Ganho medido: `propose_merges` denso N=1000 **−97 %**; `compact`/`doctor` **−94 %**.

### 5.9 Fluxo: PERT/CPM (D205)

Sobre o DAG `depends_on`, `dur(id) = lead_time` (fechada: `close − create`): `L(id) = dur(id) + max_{t∈deps} L(t)`; caminho crítico = `argmax L`, com desempate determinístico (maior `total_ms`, caminho mais longo, menor lexicográfico).

---

## 6. Protocolo de escrita e deduplicação (D26/D80)

```
Draft → validação de forma → fase 1: recall lexical de candidatos
      → decisão calibrada por score:
            score < 0.75        → cria nota nova
            0.75 ≤ score < 0.92 → MERGE na existente (união de campos; revision++; claims/prov idem)
            score ≥ 0.92        → REJEITA (duplicata)
      → evento (op. commit) → purge/flush derivados
```

- **Dedup on-write** (notas); **on-read** para `events.jsonl`/containers.
- **Lexical no write; semântico é eventual** (reconciliação) — nunca bloqueia.
- **Merge** funde `tags`/`anchors`/`outcomes` (união), `revision++`, sem sobrescrever o corpo. `confidence` **nunca** é armazenada (D142).
- **Supersede caminhável:** `replaces` + ponteiro reverso `superseded_by` (D98); bidirecionalidade validada por integridade.
- **Data contract soft por tipo (D191):** slots mínimos de corpo (ex.: `decision`→Alternativas/Por quê/Consequência) → `warnings[]`; `invalid_input` só sob `strict`.
- **Lote:** `--batch -` (JSONL) com teto `batch_max` (100); item inválido vira warning.

---

## 7. Determinismo, concorrência e crash-safety

**Determinismo é requisito** (D65/D92): `HashMap`/`HashSet`/`LinkedList` **proibidos**; tempo/RNG/ambiente/FS por porta; tie-breaks explícitos e documentados; algoritmos com ordem canônica.

- **Atomicidade:** escrita `tmp + rename` no mesmo diretório (D20), `fsync` em batch (D22); ordem **nota antes do evento** (D21).
- **Lock advisory** por arquivo-alvo, ordem externo(container)→interno(nota) para evitar deadlock ABBA (D25), liberação RAII, stale 30 s.
- **Rebuild double-buffer:** `.idx.new/` + rename atômico (D27) — leitor vê o índice antigo **ou** o novo.
- **Multi-dev (D153):** verdade = arquivo-por-nota (merge natural no git); índice reconstruível; eventos/cache vetorial por `merge=union`; conflito de nota é **pulado e reportado** pelo `doctor`, nunca auto-mergeado.
- **Verificação:** `make miri` (UB), `make fuzz` (TOON, JSONL), proptest nas funções puras, `FaultyFs` injeta falhas de escrita para provar atomicidade/rebuild. `DIVERGENCES.md` cataloga cada borda com o teste que a trava.

---

## 8. Avaliação (benchmarks)

Bancada **isolada do workspace e sem dependências externas** (`bench/`, zero crates além do `knudge-core`), em dois níveis: **micromb** (componentes puros) e **e2e** (ações do binário `kd` real sobre corpus sintético determinístico). `criterion` é **não-objetivo**: a bancada é *observação*, não gate (E13-T09/R43).

**Qualidade de retrieval:** Recall@k / MRR / nDCG@k sobre corpus PT-BR sintético e rotulado (tópicos × consultas), baseline versionado (`bench/qualidade.md`).

**Ambiente:** Ryzen 5 5500U; btrfs; `rustc 1.98.1`, release `lto="fat"`, `codegen-units=1`, `overflow-checks=true`. Corpus `N=200 → 235` notas e `N=1000 → 1167` notas.

**Gargalos identificados e corrigidos:**

| Achado | Magnitude | Correção | Ganho |
|---|---|---|---|
| `doctor`/`compact` O(N²) (`propose_merges` roda BM25 para cada nota) | 167 ms → **2,46 s** (4,97× notas → 14,7× tempo) | MinHash/LSH (D204) | **−94 %** |
| auto-drain ocioso O(N) em **todo** comando não-`maintenance` | `prime`/`self version` 40–48 ms | pular drain quando não há o que embedar | −92 %/`prime` |
| `Index`+`Graph` reconstruídos a cada invocação (não usam `.idx/`) | `Index::build` ~8,3 µs/nota | leitura paralela (`thread::scope`) | `ask` −40 % |
| `rewind` lê o corpo de **todas** as notas | 362 ms (N=1167) | corpos sob demanda + `impact` pré-computado | **−58 %** |
| piso fixo de invocação | ~4 ms (clap) – 13 ms (Session) | subscriber de log sob demanda | ~5–9 ms |

**Honestidade metodológica:** máquina de desenvolvimento (não isolada), corpus sintético **denso** (vocabulário de 24 palavras) e sem servidor de embeddings; o relatório explicita que isso **ordena custos e acha gargalos**, não crava latências absolutas. Duas otimizações foram **rejeitadas por medição** (dependências O7 e formato binário do `.idx/`, porque o leitor ainda precisa ler notas para o grafo).

---

## 9. Interfaces

**CLI (`kd`) — ~10 verbos, "poucos verbos, muitos modos" (Docker-like):** `init`, `prime` (protocolo estático byte-idêntico), `rewind`, `ask`, `write`, `task`, `map`, `drain`, `doctor`, `maintenance`, `config`, `forget`, `sync`, `self`.

**Contratos de máquina:** `stdout = dados`, `stderr = logs`; `--json` é o envelope; pipe `id|statement|score|why` (D39); `--params '<json>'` (D147); listas por repetição **ou** vírgula (D210); conjuntos fechados listam opções + *did-you-mean* (D212); EPIPE → exit 0 (D73).

**Exit codes** (`ErrorKind` → `exit_code`): `NotFound 3`, `InvalidInput 2`, `Conflict 4`, `Io 5`, `Timeout 6`, `Config 7`, `Schema 8`, `UnsafeBlocked 9`, `Internal 70`; `0` sucesso, `101` panic.

**MCP (`knudge-mcp`)** — JSON-RPC 2.0 sobre stdio, 4 gatilhos: `knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`. Hints são **ponteiros** (`id + statement + score`), não corpos.

---

## 10. Posicionamento

| Sistema | Diferença |
|---|---|
| **RAG clássico** | knudge é memória de **projeto** em Markdown versionado, não indexador de documentos grandes/PDFs |
| **GraphRAG** | mantém grafo explícito + Louvain/PPR, mas a fonte é texto legível — o grafo é projeção derivada |
| **Memória de agente (chat)** | persistência local por projeto, sem servidor/nuvem; multi-dev via git |
| **Wikis/ADRs** | uma afirmação por nota + dedup calibrado + ciclo de vida; mergeável por git |

**Propriedade incomum:** determinismo byte-a-byte combinado com busca híbrida (BM25 + RRF + vetorial opcional + Grafo), permitindo goldens e reprodutibilidade científica.

---

## 11. Limitações e questões de pesquisa em aberto

**Reconhecidas pelo próprio repo:**
- Corpus sintético **denso** superestima candidatos no dedup e subestima o ganho da peneira de postings — os números e2e são para *ordenar*, não absolutos.
- `.idx/` é reconstruído a cada invocação (frescor por `mtime` foi medido e **não** habilitado: parse 13,8 ms > rebuild 8,5 ms).
- Auto-drain roda após quase todo comando e pode embedar (rede) itens desnecessários; consumidores que esperam EOF (agentes) **esperam** o drain.
- Sem servidor de embeddings, `ask` degrada para BM25 com `warnings`.

**Tensão de contrato (auditada em `plan/proposals/usabilidade_humana.md`):** o CLI é desenhado para o LLM, mas parte da superfície é humana. Achados abertos incluem: `init` ensina uma sintaxe de `write` que falha (posicional ≠ `--summary`); `doctor --fix` remove âncoras quebradas e só reporta no `--json`; `rejected`/`merged` saem com exit 0 idêntico a `created`; `${id}` não é re-validado contra o conteúdo após edição manual; mensagens vazam identificadores internos (`(D93)`); não há verbo de "ler/editar nota". Cada um tem arquivo:linha e teste de regressão proposto.

**Questões abertas de pesquisa:** calibração ótima de peso de canal em corpora reais (não sintéticos); robustez do fold de acentos + stemming PT no recall multilíngue; trade-off entre reconstrução do índice vs cache vetorial versionado (D148/D153); e a validação de `id` derivado contra conteúdo (toca o contrato de bytes).

---

## 12. Reprodutibilidade

```sh
# instalação
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash

# ciclo
kd init && kd ask "…" --brief && kd write --summary "…" --type decision && kd sync -m "…"

# benchmarks (mediana/latência por operação; qualidade = Recall@k/MRR/nDCG@k)
make bench          # micro + e2e + qualidade
make bench-quality  # só qualidade de retrieval
make bench-sweep    # varredura rrf_k × pesos de fusão (D179)

# verificação
make miri ; make fuzz
```

**Extensibilidade:** `knudge-core` é crate publicável; `Knudge::builder().knowledge_dir(".a/b").open()` reproduz a montagem da CLI/MCP para embutir o núcleo em outro projeto Rust (`wiki/integration/`).

---

## 13. Apêndice — constantes (referência rápida)

| Constante | Valor |
|---|---|
| BM25 `k1` / `b` / pesos de campo | `1.5` / `0.75` / `3.0`,`2.0`,`1.0` |
| `CONFIRMATION_STEP` | `0.1` |
| `max_term_ratio` / `MIN_CUTOFF_CORPUS` | `0.9` / `64` |
| RRF `k` / pesos (lex, anchor, semantic, ppr) | `60` / `1.0`,`2.0`,`30.0`,`0.0` |
| Wilson `Z_95` | `1.96` |
| `AGE_HALF_LIFE_DAYS` / `AGE_WEIGHT` | `90` / `0.05` |
| `FEEDBACK_WEIGHT` / `CONTRADICTION_PENALTY` | `0.2` / `0.1` |
| `DRIFT_FLOOR` | `0.5` |
| `REVIEW_THRESHOLD` / `GROWTH_PERCENT` | `0.5` / `50` |
| KL `SMOOTHING` / `DRIFT_THRESHOLD` / `MIN_DRIFT_NOTES` | `1e-9` / `0.5` / `4` |
| PageRank `DAMPING` / iters / tol | `0.85` / `32` / `1e-8` |
| Louvain `MAX_LEVELS` / `MAX_PASSES` / `EPSILON` | `8` / `64` / `1e-9` |
| MinHash `SIGNATURE_LEN` / `BANDS` / `ROWS` / `MIN_LSH_CORPUS` | `64` / `16` / `4` / `256` |
| Dedup thresholds (create/merge/reject) | `<0.75` / `0.75–0.92` / `≥0.92` |
| `SCHEMA_VERSION` / `INDEX_FORMAT` | `2` / `retrieval-v4` |

---

*Síntese produzida por leitura direta do código e das specs em `knudge v0.5.2` (commit `2ef623c`). Fontes primárias: `wiki/specs/{matematica,busca,TOON,modelo-de-dados,escrita-e-dedup,ciclo-de-vida,grafo,determinismo,erros}.md`, `plan/00_panorama.md`, `plan/implementation/16_cli_surface.md`, `bench/RELATORIO.md`.*
