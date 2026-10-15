# knudge — síntese técnica para investigação

> **Artefacto analisado:** `REF/knudge` (repositório de referência, *read-only*), versão de
> workspace **0.5.4** (edição Rust 2024, MSRV 1.97), licença `MIT OR Apache-2.0`.
> **Data da análise:** snapshot lido diretamente do código e da documentação do repo (não de
> fontes externas). Números de benchmark e contagens foram verificados por inspeção de ficheiros.
> **Escopo:** este documento resume o *design*, os *contratos* e as *decisões* relevantes para
> quem investiga memória durável para agentes, recuperação de informação (IR) determinística e
> engenharia de sistemas com garantias de auditabilidade.

---

## 1. Problema e tese

O knudge endereça **memória durável por projeto para agentes LLM**. O problema de investigação
subjacente: como dar a um agente memória persistente, auditável e reprodutível **sem** introduzir
infraestrutura com estado (servidor, base de dados, daemon) que degrade a previsibilidade?

**Tese de design (três invariantes):**

1. **A nota é a verdade; o índice é derivado.** O conhecimento vive em ficheiros Markdown
   (`notas/<tipo>/<id>.md`, versionados em git). Todo o índice (BM25, âncoras, grafo, embeddings)
   é **descartável e reconstruível** a partir das notas; apagar o derivado nunca perde
   conhecimento. Isto separa *fonte de verdade* de *estrutura de acesso* — consequência direta
   para integridade e recuperação de desastre.
2. **Determinismo como contrato.** Sem RNG; ordem canónica (`BTreeMap`/`IndexMap`); desempates
   explícitos; toda a aleatoriedade "aparente" (MinHash, PageRank) é *hash* determinístico. A
   mesma consulta sobre o mesmo corpus produz sempre o mesmo resultado, byte a byte.
3. **Contrato de máquina versus experiência humana.** `stdout = dados`, `stderr = logs`; envelope
   `--json` estável; códigos de saída mapeados a `ErrorKind`. A superfície é *LLM-first* por
   desenho.

**Ciclo operacional:** `ask → write → task → sync` (procurar antes de gravar, registar
conhecimento, planear/executar trabalho, versionar).

---

## 2. Arquitetura

Separação estrita em **núcleo puro + portas + adaptadores** (hexagonal). O domínio **não conhece**
terminal, `argv`, relógio global, RNG ou sistema de ficheiros; todo o acesso ao mundo externo
atravessa uma **porta** (`knudge_core::ports`).

```
knudge-mcp ──┐
             ├── knudge-core (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

| Crate | Papel | Observação |
|---|---|---|
| `knudge-core` | Modelo, schema, retrieval, ciclo de vida, grafo, **portas** | Biblioteca pública (`publish = true`, D214); `adapters` (std) isolado |
| `knudge-cli` | Binário **`kd`** | Só `bin`; `clap`, `tracing`, I/O de terminal |
| `knudge-mcp` | Servidor **MCP** (JSON-RPC 2.0 stdio) | Motor de gatilhos + transporte; nada de domínio |

**Portas e respetivos fakes** (permitem testar o domínio byte a byte, sem I/O real):

| Porta | Fornece | Fake |
|---|---|---|
| `Clock` | instante UTC | `FixedClock` |
| `Rng` | aleatoriedade (só jitter) | `SeqRng` |
| `Env` | variáveis/argumentos | `FakeEnv` |
| `Fs` | escrita atómica, append, rename, `fsync`, mtime | `MemFs`, `FaultyFs` |
| `Git` | worktree/status (sem shell) | `FakeGit` |
| `HookRunner` | hooks externos (timeout + kill de grupo) | `NoopHookRunner` |
| `Logger` | log estruturado (stderr) | `RecordingLogger` |

**Nota metodológica:** `FaultyFs` permite *crash-injection* — testar a invariante "nota antes do
evento" e o rebuild *double-buffer* sob falha. Este é o padrão arquitetural mais reutilizável do
projeto para quem investiga *crash consistency*.

**Fachada de incorporação (D214):** `knudge_core::Knudge::builder()` permite usar o núcleo como
biblioteca noutro projeto, com o diretório de conhecimento configurável (layout relativo). Isto
abre a porta a reutilização fora da CLI.

---

## 3. Modelo de dados e identidade

**Nota = frontmatter TOON (formato próprio) + corpo Markdown.** Sem nota parcial (D05): opcionais
são **omitidos**, nunca `null`.

**Identidade endereçada por conteúdo** — este é o mecanismo central e merece atenção de
investigação:

| Elemento | Fórmula | Implicação |
|---|---|---|
| `normalize(s)` | NFC + trim + colapso de whitespace | estabilidade Unicode |
| `body_hash` | `hex8(normalize(statement) + LF + normalize(body))` | cobre *statement* + corpo |
| `id` | `<prefixo>_<base36(8)>(type + U+001F + normalize(statement))` | **derivado**, nunca inventado |
| hash curto | SHA-256 truncado aos 4 primeiros bytes (`u32` BE) | D95 |

Propriedades: **idempotente sob retry** (mesmo conteúdo → mesmo `id`); **reclassificar o `type`
não reescreve o `id`** (a linhagem fica explícita via `superseded_by`); edição de corpo muda o
`body_hash` e invalida vetores/índice. O `id` usa **NFC**; o *fold* de acentos e o *stemming* são
exclusivos do índice derivado — `notas/` nunca é reescrito por eles.

**Chaves canónicas:** 31, em ordem fixa. `REQUIRED_KEYS` = 5 (`id`, `statement`, `created_at`,
`body_hash`, `schema_version`). **Enums fechados** (`NoteType` 10 valores, `Scope` 3,
`Classification` 3, `Status` 6, `EdgeKind` 12) — tipos abertos degradam o retrieval porque o LLM
inventa categorias.

**Extensões semânticas (aditivas, `SCHEMA_VERSION=2`, D207):**
- **Ontologia leve (SKOS-lite):** `same_as`/`broader`/`narrower`/`related` (`broader`↔`narrower`
  inversos; `same_as`/`related` simétricos).
- **`claims`:** triplos SPO `{subject, relation, object}` (≤120 escalares), permitindo deteção de
  **contradição precisa** (`claim_conflicts`).
- **`provenance`:** mapa PROV-lite `{entity, activity, agent}`.
- **Data contract soft (D191):** *slots* mínimos de corpo por espécie (ex.: `decision`→
  Alternativas/Por quê/Consequência), verificados por cabeçalho/rótulo com *fold* de diacríticos.
  É **aviso**, não erro (erro só sob `strict`).

---

## 4. Recuperação de informação (núcleo quantitativo)

A busca é uma **cascata determinística**: filtros → BM25 → âncoras → vetorial → PPR → **fusão RRF**
→ penalidade de contradição → confiança/idade/drift → snippet.

### 4.1 BM25 (D35–D38, D173)

`score(d,q) = type_weight · (1 + 0.1·confirmação) · Σ_f w_f · Σ_t IDF_f(t) · tf·(k1+1)/(tf+k1·norm_f)`

- `k1=1.5`, `b=0.75`; **IDF por campo** (Robertson-Sparck Jones com `ln_1p`); pesos de campo
  `statement 3.0 / tags 2.0 / body 1.0`.
- **Peso por tipo** (`decision 1.20` … `epic 0.70`).
- **Corte de alta frequência** (D173): descarta termo se `df(t)/N ≥ 0.9`, **só** com `N ≥ 64`.
  Justificação: o RRF usa *ranks*; um termo quase universal produziria rank-1 espúrio apesar do
  IDF pequeno. Abaixo de 64 notas o corte é *no-op*.

### 4.2 Fusão RRF (D81/D123/D179)

`RRF(x) = Σ_c peso_c / (k + rank_c(x) + 1)`, com `k=60`; pesos `lexical 1.0`, `anchor 2.0`,
`semantic 30.0`, `ppr 0.0`. Desempate `(score desc, id asc)`. Canal ausente não quebra (degrada
com `warnings`). O peso `semantic=30` **Pareto-domina** o neutro num corpus PT-BR rotulado —
relevante para quem estuda calibração de fusão de rank.

### 4.3 Confiança: Beta-Bernoulli + Wilson (D87/D189/D203)

Cada `outcome` é um ensaio (`success=1`, `partial=0.5`, `failure/abandoned`=1 falha). Com prior
`Beta(1,1)`, posterior `Beta(1+s, 1+f)`; a **média** alimenta `stars`, mas a confiança
conservadora é o **limite inferior de Wilson a 95%** (`z=1.96`), `clamp`ado a `(0,1]`. Uma nota
com **1** sucesso obtém ≈0,21 e **não empata** com uma com **20** sucessos (≈0,84). Composição:

`score = clamp01((base + evidência + feedback + tarefa + recência) · drift_factor)`

com meia-vida de idade de 90 dias, `AGE_WEIGHT=0.05` (só desempate), `drift_factor = 1 − 0.5·drift
∈ (0.5, 1]`. **Monotonicidade verificada por proptest** (crescente em similaridade/sucessos/
feedback; decrescente em falhas/drift/idade).

### 4.4 Retenção FSRS-like (D190)

O shelf-life deixa de ser prazo fixo: `R(t) = limiar^(t/ttl) = exp(−t/S)`, `S = ttl/ln(1/limiar)`,
`limiar=0.5`. Cada sucesso **estende** o prazo em `growth=50%` da base e reseta a origem
(`max(created, último ensaio, último uso se renew_on_use)`). `ttl ≤ 0` ⇒ nunca decai
(`foundational`).

### 4.5 Dedup escalável (D26/D80/D204)

Duas fases: *recall* lexical de candidatos → decisão (score `<0.75` cria, `0.75–0.92` **merge**,
`≥0.92` **rejeita**). Em corpus pequeno/esparso, peneira exata de *postings* (byte-idêntica). Em
corpus denso grande (`>256` notas), **MinHash (64 perm.) + LSH (16 bandas × 4 linhas)** como
*blocking*, com candidatos ainda validados pelo **Dice exato** — garante propostas **idênticas ou
superconjunto** (recall ≥). Detalhe relevante: a peneira por *postings* só pula documentos sem
*overlap*; num corpus sintético denso quase todo o par partilha termos, pelo que o ganho só
aparece com vocabulário real.

---

## 5. Grafo e inferência

- **Arestas explícitas** = chaves de frontmatter (a nota é a verdade). A extração textual é
  **sugestão** em `.idx/suggestions.jsonl`, **nunca** aresta (D49/D50).
- **Integridade/ciclos:** SCC (Kosaraju iterativo); membros de ciclo **não demovem** (D45).
- **PageRank/PPR (D192):** iteração de potência, `damping=0.85`, `≤32` iter, `tol L1=1e-8`; massa
  *dangling* redistribuída; as arestas de autoridade (`references`/`supports`/`extends`/
  `replaces`) definem um subgrafo dirigido. Canal `ppr` default desligado (`0.0`).
- **Comunidades (D193):** **Louvain determinístico** (≤8 níveis, ≤64 passos, *tie* mantém a
  comunidade) sobre arestas + âncoras partilhadas.
- **Ontologia (D207):** classes de equivalência, clausura `broader`/`narrower`, deteção de ciclo de
  hierarquia, `claim_conflicts`.
- **TMS/defeasible (D208):** `retracted` (premissas `forgotten`/`superseded`), `defeated_dependents`
  (transitivo via `depends_on` reverso), `defeated_by_replacement` — **sem apagar**, com motivo
  (`Defeated`) no `prune`.
- **Drift de termos (D208):** divergência Jensen-Shannon (base 2, *smoothing* `1e-9`) entre metade
  antiga/nova por tópico; propõe revisão se `JS ≥ 0.5` e `≥4` notas.

---

## 6. Tarefas como vista derivada

As tarefas **não** são um tipo separado de armazenamento: são uma **vista derivada** de notas.
`scope` = nível (`epic`/`issue`/`task`); `type` = espécie. Hierarquia fechada
`epic ⊃ { issue ⊃ task | task }` (épico = raiz sem `type`; issue opcional; profundidade ≤ 4).
Marcador durável `<!-- knudge:parent <id> blocks <n> -->` no corpo + aresta `results_in` derivada.
Tudo o que parece "estado" (papel Bug/Spike/Risk/Decision, modo, progresso, impacto, fluxo) é
**derivado**, nunca armazenado. Fechar exige **evidência** (`--outcome`); `close_task` corre
validators e infere o `outcome`. **Caminho crítico** via PERT/CPM (D205) sobre o DAG `depends_on`.

---

## 7. Persistência, concorrência e recuperação

| Peça | Regra |
|---|---|
| Verdade | `notas/<tipo>/<id>.md` (versionado, layout material por tipo — D150) |
| Auditoria | `eventos/events.jsonl` (+ `events-NNNN.jsonl`, `merge=union`) |
| Derivado | `.idx/` (índice, âncoras, drift, uso, sugestões, checkpoint); cache vetorial opcional |
| Escrita atómica | tmp + rename no mesmo diretório (D20); `fsync` em batch (D22) |
| **Ordem de commit** | **nota primeiro, evento depois** (D21); crash deixa nota sem evento (recuperável) |
| Lock | advisory por ficheiro (`O_CREAT\|O_EXCL`, stale 30 s, reclaim por rename; ordem
externo=container/interno=nota; RAII — D23–D25) |
| Rebuild | **double-buffer** `.idx.new/` + rename atómico (D27): o leitor vê antigo **ou** novo, nunca parcial |
| Remoção | canónico primeiro, depois purga do derivado (D84/D160) |

**Ponto de interesse para investigação de sistema:** a ordem "nota antes do evento" transforma um
crash num estado *recuperável* (nota sem evento) em vez de um estado *inconsistente* (evento sem
nota). Combinada com o rebuild *double-buffer* e o lock *advisory*, dá garantias verificáveis sem
transações distribuídas.

---

## 8. Embeddings (canal opcional)

- **Provedor** `http` (default; servidor OpenAI-compatible local, endpoint `127.0.0.1:8889`,
  D202) / `lightweight` (hash, CI) / `none` (BM25). Cliente HTTP/1.1 **bloqueante sobre
  `std::net`** — **sem** `tokio`/`reqwest`; **sem** inferência in-process (D101).
- Modelo default `granite-embedding-97m-multilingual-r2` (384d).
- **Índice** `.idx/embeddings.jsonl` com `meta` (provider/model/revision/dimensões); mudança de
  modelo **invalida** e re-embeda. **Cache** por `(body_hash, model)` com LRU; versionável.
- **Estado** `indexed|pending|stale` **derivado**; falha marca `pending` (nunca descarta). **Modo**
  `lazy` (default; drena 1 lote *best-effort* após cada verbo) / `manual`.
- **Worker:** `kd drain service --install` instala `systemd --user`/`launchd` + servidor
  `llama.cpp` + GGUF (revisão pinada + SHA-256, D183).

---

## 9. Contrato de interface e erros

- **Superfície: 14 verbos** (`D209`): `init prime rewind ask write task map maintenance doctor
  drain config forget sync self` (+ `help`; `kd` sozinho = `help`). `prime` é o protocolo
  **estático byte-idêntico**; `rewind` é o estado dinâmico. O binário `knudge-mcp` expõe 4 gatilhos
  por MCP (`knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`).
- **Envelope `--json`:** `{success, command, data?, error{code,message,retryable}, warnings?}`.
- **Exit codes ↔ `ErrorKind`:** `InvalidInput=2`, `NotFound=3`, `Conflict=4`, `Io=5`, `Timeout=6`,
  `Config=7`, `Schema=8`, `UnsafeBlocked=9`, `Internal=70`; `0` ok; `101` reservado a panic.
  EPIPE → exit 0 (D73).
- **Conjuntos fechados (D212):** valor inválido → lista completa + "você quis dizer" (Levenshtein).
  **Listas (D210):** repetição ≡ vírgula (`--tag a,b` = `--tag a --tag b`).
- **Escopo obrigatório (D144):** `map`/`ask --rank`/`maintenance learn|compact|prune`/`task list`
  exigem filtro ou `--universe`, senão exit 2 (previne varrimentos O(N) acidentais).

---

## 10. Evidência empírica (benchmark)

A bancada (`bench/`) mede latência ponta-a-ponta e micro-componentes. Corpus sintético
determinístico; máquina de desenvolvimento (Ryzen 5 5500U, btrfs) — números servem para **ordenar
custos**, não para cravar valores de produção.

**Gargalos identificados (baseline pré-reforma E15) e o seu resultado:**

1. **Dedup pairwise O(N²)** em `doctor`/`compact`: 167 ms → **2,46 s** ao passar de 235 → 1167
   notas. Atacado com MinHash/LSH (D204) → **−94%**.
2. **Auto-drain O(N) escondido em todo comando não-`maintenance`**: cortava `prime`/`self version`
   de ~40–48 ms para ~13 ms quando desligado. Atacado (E15-T03) → **−92/−94%**.
3. **Reconstrução de `Index`+`Graph` a cada invocação** (leitura de todas as notas) em vez de usar
   `.idx/`. Leitura paralela (`std::thread::scope`) → `ask` **−40%**, `task list --universe` **−51%**.
4. **`rewind` lê o corpo de todas as notas**: 362 ms em N=1167; atacado (E15-T09) → **−58%**.
5. **Piso fixo de invocação** ~4 ms (só `clap`) a ~13 ms (clap + `logging::init` + `Session::open`).

**Decisões de engenharia notáveis por medição:** dependências propostas na Onda 7 (`memchr`,
`rayon`, `mimalloc`) e o formato binário do `.idx/` foram **rejeitadas com dados** — `mimalloc`
regride em carga I/O-bound; o binário economiza <20% porque as notas ainda são lidas para o grafo.
O carregamento do `.idx/` por `mtime` ficou implementado mas **não habilitado** (decodificar o
JSONL custa 13,8 ms vs. 8,5 ms de rebuild — ligar regrediria `ask`). Este é um caso de estudo
sobre *medir antes de otimizar*.

---

## 11. Qualidade, testes e disciplina de engenharia

- **~883 testes** + `proptest` em funções puras + **goldens** para o contrato de bytes.
- **Camadas:** golden (bytes de `prime`/`--json`/erros/EPIPE), property (TOON, RRF, decay, ids),
  stress (lock sem *lost update*), crash-injection (`FaultyFs`), bordas (`DIVERGENCES.md`,
  ~115+ bordas catalogadas, cada uma com o teste que a trava), aceite (`17_matriz_aceitacao.md`).
- **Regras duras:** `#![forbid(unsafe_code)]`; sem `unwrap`/`expect`/`panic` em `src/`; ficheiros
  de produção **≤ 300 linhas**; `clippy -D warnings` (pedantic); `HashMap`/`HashSet` proibidos
  (determinismo).
- **Escala do código medida:** ~49.9k linhas em 396 ficheiros `.rs` de produção; **201 ficheiros
  de ADR** (`wiki/specs/adrs/`), numeração D01–D215. A densidade de ADRs por linha de código é, em
  si, um dato de investigação sobre *design recordado*.
- **Supply chain / CI:** `deny`/`audit`/`machete`/`typos`, `miri` no core puro, *fuzz* de TOON/JSONL.

---

## 12. Apreciação crítica

**Pontos fortes (relevantes para investigação):**

- **Auditabilidade como propriedade de primeira classe.** Notas legíveis por humanos + eventos
  append-only + derivado reconstruível ⇒ qualquer estado é explicável e reproduzível. Modelo
  raro e diretamente explorável para *provenance tracking* em agentes.
- **Determinismo rigoroso.** Sem RNG, ordem canónica, desempates explícitos ⇒ resultados
  reproduzíveis — pré-requisito para avaliação científica de IR.
- **Decisões quantitativas justificadas e calibradas** (*k=60* medido "inerte"; `semantic=30`
  Pareto-dominante; corte de alta frequência com limiar vs. tamanho de corpus).
- **Disciplina de engenharia verificável** (portas + fakes, crash-injection, goldens, ≤300 linhas)
  ⇒ o sistema é, ele próprio, um *testbed* para *crash consistency* e determinismo.
- **Rejeitar dependências por medição** ⇒ higiene de *performance* acima da moda de stack.

**Limitações e tensões (para agenda de investigação):**

- **Custo de reconstrução vs. cache do índice.** O desenho "derivado descartável" implica
  re-scan por invocação; a mitigação (paralelismo, *blocking*) tem teto. A questão aberta é o
  *trade-off* ótimo entre frescura, latência e complexidade de invalidação.
- **Recalibração de constantes empírica e local.** `k=60`, pesos RRF e limiares de dedup foram
  calibrados num corpus sintético/PT-BR; a generalização cross-corpus e cross-língua está em
  aberto (o próprio documento de matemática admite-o).
- **Embeddings opcionais e externos.** Sem servidor local, `ask` degrada para BM25 — não há
  inferência in-process. Boa higiene de dependências, mas limita o *recall* semântico por default.
- **Usabilidade humana secundária ao contrato de máquina.** Diagnóstico documentado no próprio
  repo (`plan/proposals/usabilidade_humana.md`): saída em *pipe* mesmo em TTY, sem cor/pager/
  tabelas, sem deteção de TTY no *stdout*, sem editor. A proposta (D216–D222) **não está
  implementada** (verifiquei: zero ocorrências de `--format`/`NO_COLOR`/`Command::Status` no
  código). É uma lacuna clara de *dual-target* (máquina vs. humano).
- **Modelo de conhecimento "chato por desenho".** *Data contract* soft por espécie (D191) e enums
  fechados reduzem ambiguidade ao custo de flexibilidade de representação — tensão clássica entre
  estrutura normativa e abertura.

---

## 13. Onde olhar no repositório

| Interesse | Ficheiro(s) |
|---|---|
| Arquitetura, portas, escopos | `wiki/specs/ARCHITECTURE.md` |
| Quantitativo (fórmulas e derivações) | `wiki/specs/matematica.md` |
| Modelo de dados / contrato de bytes | `wiki/specs/modelo-de-dados.md`, `wiki/specs/TOON.md` |
| Busca, escrita/dedup, grafo, ciclo de vida | `wiki/specs/busca.md`, `escrita-e-dedup.md`, `grafo.md`, `ciclo-de-vida.md` |
| Decisões fechadas (D01–D215) | `plan/03_decisoes-fechadas.md`, `wiki/specs/adrs/` |
| Bordas e testes que as travam | `wiki/specs/DIVERGENCES.md` |
| Superfície CLI / aceite | `plan/implementation/16_cli_surface.md`, `17_matriz_aceitacao.md` |
| Evidência de performance | `bench/RELATORIO.md`, `bench/ULTIMO-v0.5.0.md`, `bench/qualidade.md` |
| Usabilidade humana (proposta) | `plan/proposals/usabilidade_humana.md` |

---

## 14. Relevância e questões abertas

**Pré-requisitos:** o que torna o knudge um objeto de estudo é a combinação de (a) *IR
determinística* num contexto de agentes, (b) um modelo de memória com *ciclo de vida* (retenção
FSRS-like, decay de âncoras, drift de termos, TMS/defeasible) e (c) um compromisso verificável com
auditabilidade e reprodutibilidade.

**Questões abertas que o artefacto levanta:**

1. **Calibração cross-corpus:** como transportar `k`/pesos/limiares para corpora reais
   (não-sintéticos, multi-língua) sem perder determinismo?
2. **Invalidação de índice:** existe um teste de validade barato (manifesto `body_hash`+`mtime`)
   que torne o *cache* do índice sempre vantajoso? O projeto demonstrou que nem sempre é.
3. **Dedup com *recall* garantido:** o *blocking* MinHash/LSH preserva *recall ≥* vs. o exato —
   qual é o custo real em vocabulário aberto e o ponto de viragem face à peneira exata?
4. **TMS/defeasible em escala:** o modelo "nunca apagar, só retratar" mantém-se viável quando a
   base de conhecimento cresce e as cadeias de derrota se alargam?
5. **Dual-target:** como conciliar experiência humana e contrato de máquina sem duplicar
   superfícies nem perturbar golden tests (a proposta D216–D222 é exatamente este problema em
   aberto)?
6. **Proveniência PROV-lite:** `claims`+`provenance` habilitam raciocínio verificável sobre
   afirmações; que consultas/garantias se podem construir sobre `claim_conflicts` e TMS?

**Verificação desta síntese.** Todas as afirmações estruturais (versão, nº de ADRs, nº de verbos,
ausência de D216–D222, conjunto de enums, fórmulas e constantes) foram lidas diretamente do código
e da documentação do repositório na sessão de análise. Os números de desempenho são **citados da
bancada do próprio projeto** e devem ser lidos como ordens de grandeza, não como valores absolutos.
