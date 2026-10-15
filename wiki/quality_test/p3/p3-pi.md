---
question_id: Q3
same_session: true
question_issue: "Escreva o resumo do /REF/knudge, agora adaptado para um pesquisador, o resumo deve ficar em /exploration/knudge, em um único arquivo .md" 
expected_answer_ideia: "Deve ser capaz de listar categoricamente os algoritmos de otimização, modelo toon v3 de dados e usabilidade geral via kd write, kd task e kd ask, com a orientação clara de primeiro usar o ask, para evitar escrita desnecessária"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: pi
---

# knudge — resumo técnico para pesquisador(a)

> **Objeto:** `/REF/knudge` — workspace Rust v0.5.2; binários `kd` (CLI) e `knudge-mcp`
> (servidor MCP); biblioteca `knudge-core`.
> **Escopo:** memória de projeto durável para agentes de LLM, com notas Markdown como fonte da
> verdade e índice derivado reconstruível.
> **Método:** ficha de leitura construída a partir de `README.md`, `llms.txt`, `wiki/specs/`
> (`ARCHITECTURE.md`, `matematica.md`, `DIVERGENCES.md`), `plan/` (decisões D01–D214, propostas) e
> `bench/` (`RELATORIO.md`, `qualidade.md`).
> **Natureza:** resumo interpretativo; onde há número/fórmula, a fonte interna é citada.

---

## 1. Resumo executivo

O knudge é um sistema de **memória episódico-semântica por projeto** para agentes de LLM.
Ele trata cada fato/decisão como uma **nota Markdown atômica** (uma afirmação por nota) e mantém
um **índice derivado e descartável** (BM25 + âncoras + embeddings opcionais + grafo) que pode ser
reconstruído byte a byte a partir das notas. Não há servidor, banco nem daemon: persistência é
sistema de arquivos, concorrência é *lock* advisory por arquivo, e a integração com o agente é por
CLI (`kd`) ou por MCP (JSON-RPC 2.0 sobre stdio).

O argumento central de design é uma **inversão explícita da hipótese de RAG tradicional**:

- **RAG clássico:** um corpus externo é indexado e consultado; o LLM é consumidor.
- **knudge:** o próprio agente **escreve** o corpus durante o trabalho, e a recuperação é o que
  fecha o ciclo de aprendizado. O usuário primário é o LLM (`plan/00_panorama.md` §1), não uma
  pessoa.

O sistema é notável por três escolhas de engenharia pouco comuns:

1. **Determinismo como contrato de primeira classe** — sem RNG, sem `HashMap`, tempo/FS/git por
   portas injetáveis, ordem total em todo tie-break, testes de *property* e *golden*.
2. **Modelo quantitativo explícito** para confiança e ciclo de vida (Beta-Bernoulli + Wilson,
   retenção FSRS-like, drift de âncoras) documentado em `wiki/specs/matematica.md`.
3. **Disciplina de "nada sem aceite"** — `doctor`/`learn`/`compact`/`prune` só propõem; a
   aplicação é um ato explícito.

---

## 2. Posicionamento no espaço de pesquisa em memória de agentes

A literatura de *agent memory* tende a duas linhas: (a) sumarização/compressão de conversa
(vetores ou hierarquias de episódios) e (b) RAG sobre documentos. O knudge ocupa um terceiro
lugar: **memória curada por projeto**, com granularidade de afirmação e write-path com dedup.

| Eixo | RAG de documentos | Memorização de conversa | knudge |
|---|---|---|---|
| Unidade | chunk | episódio/turno | nota atômica (1 afirmação) |
| Fonte da verdade | corpus externo | histórico | arquivos Markdown no repo |
| Quem escreve | humano/ingest | o sistema | o agente (com dedup) |
| Índice | vetorial | vetorial/grafo | híbrido RRF (BM25+âncora+vetor+PPR) |
| Verdade vs. índice | acoplados | acoplados | **separados** (índice descartável) |
| Auditabilidade | baixa | baixa | alta (eventos + git) |

Essa separação verdade/índice é o que permite "apagar `.idx/` nunca perde conhecimento" — um
invariante testável e uma decisão de projeto (D15/D27/D84).

---

## 3. Arquitetura

### 3.1 Camadas (D65)

```
knudge-mcp ──┐
             ├── knudge-core (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

- `knudge-core` — modelo, schema, retrieval, ciclo de vida e **portas**; biblioteca pública
  (`publish = true`, D214); `#![forbid(unsafe_code)]`.
- `knudge-cli` — binário `kd`, monta adaptadores (`clap`, `tracing`, I/O).
- `knudge-mcp` — servidor MCP reativo: motor de gatilhos + transporte JSON-RPC (sem domínio).

O domínio **não conhece** SO, `argv`, relógio global, RNG global nem FS. Todo acesso externo passa
por portas (`knudge-core::ports`): `Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`.
Consequência metodológica: o núcleo é testável só com fakes (`FixedClock`, `SeqRng`, `MemFs`,
`FaultyFs`, `FakeGit`, `RecordingLogger`), e `FaultyFs` **injeta falhas de crash** para testar
atomicidade — um padrão de teste reprodutível raro em sistemas de armazenamento.

### 3.2 Escopos temáticos

`error`, `time`, `logging`, `config`, `schema`, `toon`, `jsonl`, `store`, `git`, `graph`,
`retrieval`, `write`, `handoff`, `maintenance`, `task`, `health`, `lifecycle`, `embeddings`.
Cada um tem contrato próprio em `wiki/specs/` e ADRs associados.

### 3.3 Portas de integração
- **CLI** — 14 verbos: `init|prime|rewind|ask|write|task|map|maintenance|doctor|drain|config|forget|sync|self`.
- **MCP** — 4 gatilhos (`knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`,
  `knudge_status`); hints são **ponteiros** (`id + statement + score`), não corpos, para não
  poluir o contexto do modelo.
- **Lib** — fachada `Knudge::builder()` (D214) para embutir o core em outro projeto.

---

## 4. Modelo de dados e contrato de bytes

### 4.1 Nota e identidade

Nota = **frontmatter TOON** (formato de bytes próprio) + **corpo Markdown**, em
`notas/<tipo>/<id>.md` (caminho derivado do prefixo do id, D150). Não existe nota parcial (D05).

Identidade **endereçada por conteúdo**:

```
id  = <type>_<base36(8)>  onde 8 dígitos = SHA-256(type + U+001F + normalize(statement))[0:4] (u32 BE)
normalize(s) = NFC + trim + colapso de espaços
body_hash    = hex8(normalize(statement) + LF + normalize(body))
```

Implicações de pesquisa: o id é **imutável sob reclassificação de tipo** (D01/D02/D95), e
`body_hash` cobre statement+corpo. Isso torna a escrita **idempotente sob retry** — um requisito
prático para agentes que reexecutam operações.

### 4.2 Schema

- **31 chaves canônicas** em ordem fixa (a ordem é contrato de bytes):
  `id, type, statement, created_at, body_hash, schema_version, tags, source, superseded_by,
  references, depends_on, contradicts, supports, extends, replaces, rejects, results_in, same_as,
  broader, narrower, related, revision, outcomes, classification, anchors, status, scope, checks,
  evidence, claims, provenance`.
- **Enums fechados:** `NoteType` (10), `Scope` (3), `Classification` (3), `Status` (6), `EdgeKind`
  (12 = 8 semânticas + 4 de ontologia leve). Valor desconhecido rejeita no *write* e alerta no
  *read*.
- **Extensões v2 (D207):** `claims` (triplas SPO `{subject, relation, object}`), `provenance`
  (PROV-lite `{entity, activity, agent}`) e ontologia leve (`same_as`/`broader`/`narrower`/`related`),
  com leis algébricas (inversão `broader↔narrower`, simetria de `same_as`/`related`).
- **Slots mínimos por espécie (D191, *soft*):** `decision`→Alternativas/Por quê/Consequência;
  `error`→Causa/Correção; etc. Aviso no write; erro só sob `strict` (config de projeto, D94).

### 4.3 TOON

Formato de serialização canônico na emissão e tolerante na leitura: UTF-8 cru sem BOM, comentário
`#` fora de aspas, aninhamento ≤32, inteiros sem `.0`, escapes `\n\r\t\0\u{}`, listas/mapas vazios
omitidos, `LF` final. Um contrato de bytes com **goldens** para travar regressões.

---

## 5. Persistência, consistência e recuperação

Três planos distintos (D15/D84):

| Plano | Caminho | Papel |
|---|---|---|
| Verdade | `notas/` | Markdown versionado no git |
| Auditoria | `eventos/events*.jsonl` | log append-only, `merge=union` |
| Derivado | `.idx/` | índice, âncoras, drift, uso, sugestões, checkpoint |

Mecanismos:

- **Escrita atômica:** `tmp + rename` no mesmo diretório (D20); `fsync` em batch (D22).
- **Ordem de commit:** **nota antes do evento** (D21) — crash deixa nota sem evento (recuperável),
  nunca o inverso. Hipótese de consistência explícita.
- **Concorrência:** lock advisory por arquivo-alvo (`O_CREAT|O_EXCL`), stale 30 s, *reclaim* por
  rename, ordem de aquisição documentada (D23–D25).
- **Rebuild double-buffer** (D27): escreve `.idx.new/` e troca por `rename` — o leitor enxerga o
  índice antigo ou o novo, nunca parcial.
- **Dedup de eventos on-read** por id derivado do conteúdo (idempotente sob `merge=union`, D31).

Esse conjunto é o que sustenta o invariante verificável "apagar `.idx/` nunca perde conhecimento".

---

## 6. Retrieval: cascata híbrida e fusão RRF

### 6.1 Pipeline

`filtros determinísticos → BM25 → âncoras → vetorial → PPR → fusão RRF → penalidade de contradição
→ confiança/idade/drift → snippet`.

### 6.2 BM25 (*fielded*)

Score em `wiki/specs/matematica.md` §1:

```
score(d,q) = type_weight(type_d) · (1 + 0.1·confirmação_d)
           · Σ_f w_f · Σ_{t∈q} IDF_f(t) · tf_f(t,d)·(k1+1) / (tf_f(t,d) + k1·norm_f(d))
```

Constantes: `k1=1.5`, `b=0.75`; pesos de campo `statement=3.0`, `tags=2.0`, `body=1.0`;
`IDF` de Robertson-Sparck Jones **por campo**; `type_weight` de 1.20 (decision) a 0.70 (epic);
**corte de alta frequência** quando `df/N ≥ 0.9` e `N ≥ 64` (D173) — justificado pela fusão por
*rank* (um termo quase universal produziria rank-1 espúrio). Pré-processamento: fold de
diacríticos (NFD, `café≡cafe`, D172) + stopwords PT/EN + stemming PT conservador (radical ≥4 bytes,
D206). `INDEX_FORMAT = "retrieval-v4"`.

### 6.3 Fusão RRF (Reciprocal Rank Fusion)

```
RRF(x) = Σ_c  peso_c / (k + r_c(x) + 1),   k = 60
pesos default: lexical=1.0, anchor=2.0, semantic=30.0, ppr=0.0
desempate: (score desc, id asc)
```

Os pesos **não são arbitrários**: `semantic=30` Pareto-domina o neutro no corpus de avaliação
PT-BR, e `anchor=2.0` corrige o empate entre match exato de âncora e rank-1 lexical ruidoso
(D179). Este é um ponto de calibração empírica explicitamente documentado — candidato natural a
recalibração por *learning-to-rank*.

### 6.4 Sinais e explicabilidade

O campo `why` é fechado e ordenado por precedência:
`file_match > anchor_match > tracker_match > stars > semantic > recent > universal`. O `--json`
expõe `channels: {lexical, anchor, semantic, ppr, recent, stars, body}` com as **parcelas RRF** —
ou seja, a recuperação é **auditável** por contribuição de canal. **Âncoras** ligam notas ao
código (`--anchor PATH`, globs) e viabilizam busca por arquivo sem query textual.

### 6.5 Temporalidade

`--as-of <TS>` reconstrói o conjunto ativo em um instante `T` a partir do log de eventos — uma
**consulta temporal sobre a memória**, não apenas sobre o conteúdo atual.

---

## 7. Confiança e ciclo de vida (núcleo probabilístico)

Esta é provavelmente a parte de maior interesse para pesquisa aplicada.

### 7.1 Confiança Beta-Bernoulli + limite de Wilson (D87/D189)

Cada `outcome` é um ensaio Bernoulli (`success=1`, `partial=0.5`, `failure/abandoned`=1 falha).
Posterior `Beta(1+s, 1+f)`; usa-se o **limite inferior de 95 %** (Wilson, `Z=1.96`) como confiança
conservadora. Isso é importante: com 1 sucesso o score cai a ≈0,21 e **não empata** com 20 sucessos
(≈0,84) — penaliza pouca evidência sem descartá-la. A confiança **nunca é armazenada**; é derivada
em tempo de consulta (D87).

Composição do score (matematica.md §3.4):

```
base      = similaridade · age_factor(idade)
evidência = lower_bound(s,f)
feedback  = 0.2 · max(feedback_explícito, 0)
tarefa    = task_confirmation
recência  = 0.05 · age_factor · (1 − similaridade)
score     = clamp01( (base + evidência + feedback + tarefa + recência) · drift_factor(drift) )
age_factor(a) = 1/(1 + a/90)
```

Propriedades verificadas por `proptest`: monotonicidade em similaridade/sucessos/feedback,
antitonicidade em falhas/drift/idade.

### 7.2 Retenção FSRS-like (D190)

Shelf-life deixa de ser prazo fixo e passa a seguir uma curva de retenção com estabilidade
crescente:

```
R(t) = limiar^(t/ttl) = exp(−t/S),   S = ttl / ln(1/limiar),   limiar = 0.5
ttl_efetivo = base · (1 + 0.5 · reviews / 100)
origin = max(created_at, último ensaio, último uso se renew_on_use)
```

`foundational` nunca expira; `tactical`=365d; `observational`=30d (D135). O uso **só estende**
(D154) — análogo a revisão espaçada de sistemas de memória humana.

### 7.3 Drift e demolição

- **Drift de âncoras** (D203): `drift = 1 − válidas/(válidas+quebradas)`;
  `drift_factor = 1 − 0.5·drift ∈ [0.5,1]` desconta o score inteiro.
- **Drift de termos** (D208): divergência **JS** (base 2) entre metade antiga/nova por tópico;
  propõe revisão se `≥0.5` e `≥4` notas.
- **Demolição** só **propõe** motivos `Expired | AnchorDecay | Contradicted | Defeated | Drifted`;
  membros de ciclo são protegidos; aplicação via `kd forget` — **nada é apagado** (soft delete).

---

## 8. Grafo e inferência

- **Arestas explícitas** são as únicas reais (chaves de frontmatter); extração textual vai para
  `.idx/suggestions.jsonl` e **nunca** vira aresta (D49/D50).
- **Integridade:** referências quebradas, órfãos, bidirecionalidade `replaces↔superseded_by`,
  SCC (Kosaraju) sobre `replaces`/`depends_on`; membros de ciclo **não demovem** (D45).
- **PageRank/PPR** (D192): iteração de potência, damping `0.85`, ≤32 iter, tol L1 `1e-8`, dangling
  redistribuído; canal `ppr` default desligado.
- **Comunidades** (D193): Louvain determinístico (≤8 níveis, ≤64 passos, empate estável) sobre
  arestas + âncoras compartilhadas.
- **Ontologia** (D207): classes de equivalência, clausura `broader`/`narrower`, detecção de ciclos
  de hierarquia e `claim_conflicts` (mesma `(subject, relation)` com objetos divergentes).
- **TMS/defeasible** (D208): `retracted` (premissas `forgotten`/`superseded`),
  `defeated_dependents` (transitivo via `depends_on` reverso), `defeated_by_replacement`. Sem
  apagar — apenas deriva derrota. Aproximação pragmática de *truth maintenance*.

---

## 9. Write-path e dedup anti-redundância

Dedup em duas fases (D26/D80): *recall* lexical de candidatos → decisão por score Dice:

```
< 0.75        → create
[0.75, 0.92)  → merge
≥ 0.92        → reject
```

Para escala, o *blocking* usa **MinHash (64 permutações) + LSH (16 bandas × 4 linhas)** acima de
256 notas densas; candidatos ainda passam pelo Dice exato; as propostas são idênticas ou
superconjunto (recall ≥). O `merge` une tags/âncoras/outcomes, incrementa `revision`, jamais
sobrescreve o corpo. `update` com chave nova **supersede** (novo id + `replaces`/`superseded_by`
bidirecional). O lote (`--batch` JSONL) é best-effort com `warnings[]` e `--dry-run`.

Este é o mecanismo que ataca o problema prático de **inflação de memória** em agentes que escrevem
muito — um análogo de *entity resolution* sobre texto curto.

---

## 10. Embeddings (opcional)

- **Provedor plugável** (`trait Embedder`): `http` (OpenAI-compatible local, default
  `http://127.0.0.1:8889/v1/embeddings`, D202), `lightweight` (hash, CI), `none` (BM25). Cliente
  HTTP/1.1 bloqueante sobre `std::net` — sem tokio/reqwest, sem inferência in-process.
- **Modelo default:** `granite-embedding-97m-multilingual-r2` (384d).
- **Índice** `.idx/embeddings.jsonl` com `meta` (provider/model/revision/dimensões/similaridade);
  troca de modelo invalida e re-embeda. **Cache** por `(body_hash, model)` com LRU (D148).
- **Modo `lazy`** (default): drena 1 lote best-effort após cada verbo (D131); falha marca `pending`
  e **nunca descarta** (degradação graciosa, R33).
- **Semântica:** `rank_query` (cosseno/dot, brute-force) alimenta o canal vetorial;
  `suggest` classifica pares em `duplicate`/`link`/`contradiction` (advisory).

---

## 11. Tarefas como view derivada

Tarefas **não** são entidade separada: são notas com `scope` (`epic ⊃ {issue ⊃ task | task}`,
pai por rank estritamente menor, D149). Marcador durável no corpo
(`<!-- knudge:parent <id> blocks <n> -->`) + aresta `results_in` derivada. Papel (Bug/Spike/
Risk/Decision/Epic/Feature/Story/SubTask), modo (Sequential/Concurrent/Magentic), progresso,
`epic_of` e impacto (dependentes transitivos) são **derivados**, nunca armazenados. O
fechamento exige **evidência** (validators + outcomes). `task flow` calcula cycle/lead/throughput
e caminho crítico **PERT/CPM** (D205) — uma ponte explícita entre memória e gestão de projeto.

---

## 12. Reprodutibilidade e disciplina de engenharia

- **Determinismo (D92):** sem RNG; `BTreeMap`/`IndexMap` (proíbe `HashMap`/`HashSet`); tempo/RNG/
  Env/FS só por portas; tie-breaks explícitos; PageRank/Louvain/MinHash determinísticos.
  Aritmética `f64` com `total_cmp`; `NaN→0`; `clamp` explícito.
- **Erros como contrato:** `ErrorKind → exit` (`InvalidInput=2`, `NotFound=3`, `Conflict=4`,
  `Io=5`, `Timeout=6`, `Config=7`, `Schema=8`, `UnsafeBlocked=9`, `Internal=70`); `code()` é o
  contrato de máquina; `retryable()` só `Timeout`.
- **Testes:** ~**922** blocos de teste, `proptest` nas funções puras, *goldens* para o contrato de
  bytes. Lints negam `unsafe`, `unwrap/expect/panic`, `indexing_slicing`, `arithmetic_side_effects`
  (warn); produção com arquivos ≤300 linhas.
- **DIVERGENCES:** catálogo de ~115 bordas problemáticas conhecidas, **cada uma com mitigação e
  teste que a trava** — um artefato metodológico de honestidade epistêmica raro.
- **Decisões:** ~200 ADRs (`wiki/specs/adrs/`), uma por arquivo, com contexto e impacto.

---

## 13. Avaliação empírica

### 13.1 Qualidade de recuperação (`bench/qualidade.md`)

Corpus PT-BR sintético e rotulado (192 notas, 60 consultas), métricas Recall@k / MRR / nDCG@k:

| família | n | R@1 | R@5 | R@10 | MRR | nDCG@5 |
|---|---:|---:|---:|---:|---:|---:|
| com-acento / morfologia / sem-acento / sinonimo | 12 cada | 12,5 % | 62,5 % | 100 % | 100 % | 100 % |
| working-set | 12 | **100 %** | 100 % | 100 % | 100 % | 100 % |
| **geral** | 60 | 30,0 % | 70,0 % | **100 %** | 100 % | 100 % |

Leitura: Recall@10 e MRR saturam, mas **R@1 é modesto (30 %)** — há espaço para *reranking*. Os
autores reconhecem que a bancada está **saturada** em nDCG@5, o que motiva a proposta de
reranking/ANN (`plan/proposals/reranking_ann.md`).

### 13.2 Performance (`bench/RELATORIO.md`)

Corpus sintético determinístico (N=235 e N=1167). Achados principais e correções:

- `doctor`/`compact` eram **O(N²)** (propose_merges rodava BM25 por nota); corrigido com
  MinHash/LSH → **−94 %** (D204).
- Custo escondido de varredura O(N) por auto-drain afetava até comandos sem embeddings.
- `rewind` era a ação mais cara não-quadrática; leitura paralela do corpus → **−65 %**;
  `rewind` → **−58 %**.
- Micro: `normalize`/`body_hash`/`note_id` → −84 a −89 %.
- Piso fixo de invocação ~4–13 ms (clap + logging + `Session::open`).

Limitações declaradas pelos próprios autores: máquina de desenvolvimento, corpus sintético, sem
servidor de embeddings — os números servem para **ordenar** custos, não para valores absolutos.

---

## 14. Ameaças à validade e limitações

1. **Corpus sintético e auto-rotulado.** As métricas de retrieval vêm de fixtures geradas
   deterministicamente; não há avaliação com corpus humano real nem com *ground truth* externo.
2. **Saturação da bancada.** nDCG@5 = 100 % em quase todas as famílias → poder discriminativo
   baixo; mudanças de qualidade podem não aparecer.
3. **Deduplicação heurística.** Limiares Dice 0.75/0.92 são calibrados, não aprendidos; não há
   estudo de *precision/recall* do merge/reject contra anotação humana.
4. **Embeddings brute-force.** `rank_query` é O(N·d); ANN só se justifica acima de ~10⁴ notas
   (proposta T08, ainda bloqueada por escala).
5. **Um único modelo de embedding.** Sem estudo de sensibilidade a provedor/modelo multilíngue.
6. **Peso RRF manual.** `semantic=30` e `anchor=2.0` são calibrações pontuais, não
   *learning-to-rank*.
7. **Ciclo de vida FSRS-like sem validação longitudinal.** `growth%=50`, `limiar=0.5` e
   meia-vida de 90 dias são plausíveis, mas não validados contra dados de uso real.
8. **Escopo declarado pequeno.** Não é ACL, wiki multi-usuário nem indexador de documentos — o
   que limita generalização das conclusões.
9. **Usabilidade humana** foi explicitamente descartada no design (`plan/00_panorama.md` §1), o
   que gera atritos documentados em `plan/proposals/usabilidade_humana.md`.

---

## 15. Direções de pesquisa sugeridas pelo próprio material

1. **Reranking com cross-encoder + fusão calibrada** (`reranking_ann.md`, E19-T06): segundo modelo
   local (`bge-reranker-v2-m3` etc.) para atacar o R@1 modesto.
2. **ANN / Matryoshka embeddings** (E19-T08) para escala ≥10⁴ notas, com quantização.
3. **Calibração da fusão** por *learning-to-rank* em vez de pesos manuais (D179).
4. **Revisão do ciclo de vida** com dados de uso real (FSRS vs. heurísticas) e estudo da retenção
   informativa.
5. **Auditoria empírica do dedup** (limiares 0.75/0.92, MinHash recall/precision) contra
   anotação humana.
6. **Protocolos de transferência entre projetos** (o corpus é por projeto; há fachada de
   incorporação D214, mas não há estudo de memória cross-projeto).
7. **Avaliação com corpus real** e *ground truth* independente, dado que a bancada saturou.

---

## 16. Referências internas (onde aprofundar)

| Tópico | Documento |
|---|---|
| Contrato de bytes / gramática | `wiki/specs/TOON.md`, `wiki/specs/modelo-de-dados.md` |
| Arquitetura e portas | `wiki/specs/ARCHITECTURE.md` |
| Fórmulas e constantes | `wiki/specs/matematica.md` |
| Bordas e mitigações | `wiki/specs/DIVERGENCES.md` (~115 entradas) |
| Decisões (ADRs) | `wiki/specs/adrs/` (~200), `plan/03_decisoes-fechadas.md` |
| Superfície da CLI | `plan/implementation/16_cli_surface.md` |
| Matriz de aceitação | `plan/implementation/17_matriz_aceitacao.md` |
| Embeddings / reranking / ANN | `plan/04_embeddings.md`, `plan/proposals/reranking_ann.md` |
| Benchmarks / qualidade | `bench/RELATORIO.md`, `bench/qualidade.md` |
| Propostas de evolução | `plan/proposals/` |
| Uso / skill do agente | `wiki/usage/`, `SKILL.md`, `llms.txt` |

---

## 17. Glossário mínimo

- **TOON** — formato de bytes canônico do frontmatter.
- **Âncora** — ligação nota↔arquivo de código (glob); canal determinístico de recuperação.
- **RRF** — Reciprocal Rank Fusion (funde canais por *rank*, não por score).
- **PPR** — Personalized PageRank (autoridade no grafo, semente = working set).
- **Outcome** — evidência de execução de uma tarefa (`success|partial|failure|abandoned`).
- **Supersessão** — substituição versionada de uma nota (`replaces`/`superseded_by`).
- **TMS** — *truth maintenance system*: deriva retração/derrota sem apagar.
- **`.idx/`** — índice derivado, descartável e reconstruível.

---

> **Avaliação em uma frase:** o knudge é um sistema de memória de agente com engenharia de
> reprodutibilidade incomumente rigorosa e um núcleo probabilístico explícito, mas sua base
> empírica atual é sintética e saturada — o maior valor para pesquisa está em abrir a avaliação
> para corpora reais, calibrar a fusão e validar o ciclo de vida com uso longitudinal.
