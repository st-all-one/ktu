# knudge — síntese orientada a investigação

> **Objeto:** `REF/knudge` (snapshot **v0.5.4**, workspace Rust, licença MIT OR Apache-2.0).
> **O que é:** infraestrutura de **memória durável por projeto para agentes LLM**, materializada
> como CLI (`kd`, 14 verbos), servidor MCP (`knudge-mcp`), sem servidor, sem base de dados e sem
> daemon. Duas afirmações organizam todo o desenho: **(i) a nota é a verdade; (ii) o índice é
> derivado e descartável**.
> **Como ler esta síntese:** distingo sempre **[doc]** (o que o repositório afirma),
> **[código: caminho:linha]** (o que verifiquei a ler o fonte) e **[medido]** (o que corri nesta
> sessão). O binário instalado nesta máquina é **0.6.1**, o snapshot é **0.5.4** — onde a execução
> importa, digo qual usei.

---

## 1. Ficha técnica

| Item | Valor [doc] |
|---|---|
| Versão do snapshot | `0.5.4` (`Cargo.toml`), changelog com secção "não publicado" |
| Edição / MSRV | Rust **2024** / **1.97+** (`Cargo.toml`); CI usa `stable` + `nightly` (Miri) |
| Crates | `knudge-core` (domínio puro + portas + adaptadores `std`), `knudge-cli` (binário `kd`), `knudge-mcp` (JSON-RPC 2.0 sobre stdio) |
| Dependências de produção | `thiserror`, `indexmap`, `unicode-normalization`, `sha2`, `clap`, `serde(_json)`, `tracing(_subscriber)` |
| Dependências **proibidas** | `tokio`, `reqwest`, `hyper`, `axum` (`deny.toml`); `criterion` é "não-objectivo" (R43) |
| Tamanho | 396 ficheiros `.rs` em `src/`, ~**56,9k linhas** de Rust; wiki ~**13,3k linhas** em 60+ `.md` |
| Corpus de decisões | **200 ADRs** (`wiki/specs/adrs/D01…D212`) + `plan/03_decisoes-fechadas.md` + `DIVERGENCES.md` |
| Binários distribuídos | `install.sh` (release pré-compilado, verificação SHA-256, substituição não-destrutiva), `make install` do fonte |

**Invariante de engenharia que condiciona a leitura do código** [doc/`clippy.toml`]: `unsafe_code`
negado (`forbid` no core), `unwrap`/`expect`/`panic`/`indexing_slicing`/`as`-casts negados, ficheiros
de produção ≤ **300 linhas** (`scripts/check_file_length.sh`), iteração sempre canónica
(`BTreeMap`/`IndexMap`; `HashMap`/`LinkedList` proibidos), `RUSTFLAGS=-D warnings` na CI.

---

## 2. Modelo de dados e contrato de bytes

Uma **nota** = frontmatter **TOON** (formato próprio, ordem canónica) + corpo Markdown, gravada em
`.knudge/notas/<tipo>/<id>.md` (layout derivado do prefixo, D150).

- **31 chaves canónicas**, ordem fixa; `REQUIRED_KEYS` = 5 (`id`, `statement`, `created_at`,
  `body_hash`, `schema_version`). Opcionais **omitidos**, nunca `null` (D05).
- **Identidade endereçada por conteúdo**: `id = <tipo>_<base36(8)>` =
  `hash(type + U+001F + normalize(statement))`; **reclassificar o `type` não reescreve o `id`**
  (D01/D02/D95). `body_hash = hex8(normalize(statement) + LF + normalize(body))`; hash curto =
  SHA-256 truncado aos 4 primeiros bytes.
- **Enums fechados**: `NoteType` (10; `epic` derivado de `scope=epic`), `Scope`, `Classification`,
  `Status`, `EdgeKind` (12 = 8 + ontologia leve). Chave desconhecida: rejeita no write, *warning*
  no read.
- **Extensões aditivas (`SCHEMA_VERSION=2`, D207)**: `claims` (SPO `{subject, relation, object}`,
  ≤120 escalares), `provenance` (PROV-lite `{entity, activity, agent}`) e ontologia
  `same_as`/`broader`/`narrower`/`related` (inversos/simétricos declarados).
- **TOON** é o contrato de bytes: UTF-8 cru sem BOM, LF final, documento vazio = 0 bytes, escapes
  `\n\r\t\0\u{}`, aninhamento ≤32, inteiros nunca com `.0`. Canónico na emissão, tolerante na
  leitura. **Ordem é contrato** (não apenas estilo).
- **Duas normalizações distintas, de propósito**: identidade/hashes usam **NFC** (consistência de
  bytes); tokenização de busca usa **NFD + fold de diacríticos** (`café ≡ cafe`, D172).

**Verificação empírica da formalização**: há *fuzz targets* para `toon::parse` e `jsonl::decode`
(`fuzz/fuzz_targets/`), *proptests* nas funções puras e **8 goldens byte-exatos** em
`crates/knudge-cli/tests/golden/` (incluindo `prime.txt`, `json_error_*.json`). A suíte `miri_smoke`
roda sob Miri — declarada como sanidade de *bounds*/UTF-8, já que não há `unsafe`.

---

## 3. Arquitetura e persistência

```
knudge-mcp ─┐
            ├── knudge-core  (domínio puro + portas + adapters std)
knudge-cli ─┘
```

- **Portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`, `Embedder`) isolam o SO;
  a CI/testes usam **8 fakes** (`FixedClock`, `SeqRng`, `FakeEnv`, `MemFs`, `FakeGit`,
  `NoopHookRunner`, `RecordingLogger`, `FakeEmbedder`). É o que permite determinismo byte-a-byte e
  proptest sem tocar o disco.
- **Três camadas de persistência**: *verdade* (`notas/`, versionada), *auditoria*
  (`eventos/events.jsonl`, com rotação e `merge=union`), *derivado* (`.idx/`: índice invertido,
  âncoras, drift, uso, sugestões, checkpoint, embeddings).
- **Escrita atómica** (tmp + rename no mesmo diretório; `fsync` em lote) e **ordem de commit
  nota-antes-do-evento**, de modo que um *crash* deixa nota sem evento (recuperável), nunca o
  inverso. *Locks* advisory por ficheiro-alvo, ordem de aquisição documentada (anti-ABBA), RAII.
- **Reconstrução *double-buffer*** do índice (escreve `.idx.new/` e troca por rename): o leitor vê
  antigo ou novo, nunca parcial.
- **Multiuso/multidev** (D153): a verdade é ficheiro-por-nota (merge textual natural); conflito de
  nota é **reportado** pelo `doctor`, nunca auto-fundido.

---

## 4. Recuperação: o núcleo quantitativo

`wiki/specs/matematica.md` documenta as derivações — ponto raro e valioso para replicação.
Convenções: tudo `f64`, **nunca há RNG**, comparações com `total_cmp` (ordem total IEEE-754),
`clamp01` explícito, `NaN → 0`, ordem de iteração canónica.

### 4.1 BM25 com pesos por campo (D35–D38, D173)

```
score(d,q) = type_weight · (1 + 0.1·confirmação_d) · Σ_f w_f · Σ_{t∈q} IDF_f(t) · tf·(k1+1)/(tf + k1·norm_f(d))
norm_f(d) = 1 − b + b·len_f(d)/avg_len_f ;  k1 = 1.5 ;  b = 0.75
IDF_f(t) = ln(1 + (N − df_f(t) + 0.5)/(df_f(t) + 0.5))
```

- Pesos de campo: `statement 3.0`, `tags 2.0`, `body 1.0` (o `statement` domina).
- `type_weight`: `decision 1.20`, `error 1.15`, `fact 1.10`, `def/risk 1.05`, `task/question 1.00`,
  `snippet 0.95`, `link/meta 0.90`, `epic 0.70`.
- **Corte de alta frequência**: descarta termos com `df/N ≥ 0.9` **apenas se `N ≥ 64`** — com
  justificação explícita (o RRF usa *ranks*; um termo quase universal geraria rank-1 espúrio).
- Pré-processamento: fold de diacríticos (NFD) → stopwords PT+EN → *stemming* PT conservador
  (radical ≥ 4 bytes, D206), com `INDEX_FORMAT = "retrieval-v4"`. Mudar o stemmer invalida só o
  derivado: **os bytes de `notas/` não mudam**.

### 4.2 Fusão RRF e canais (D81/D123/D179/D192)

```
RRF(x) = Σ_c peso_c / (k + r_c(x) + 1) ;  k = 60
pesos: lexical 1.0 · anchor 2.0 · semantic 30.0 · ppr 0.0 (desligado por defeito)
desempate: (score desc, id asc)
```

Cascata completa: filtros → BM25 → âncoras → vetorial → PPR → RRF → penalidade de contradição →
confiança/idade/drift → *snippet*/*why*. Canal ausente degrada com `warnings` (nunca falha dura);
o modo `strict` promove *warning* a erro.

### 4.3 Dedup em duas fases, com índice sensível à localidade (D26/D80/D204)

Recall lexical de candidatos → decisão por limiares: `score < 0.75` **cria**, `0.75–0.92`
**funde**, `≥ 0.92` **rejeita**. Para escalar: corpus pequeno/esparso → peneira exata de postings
(byte-idêntica); `> 256` notas densas → **MinHash (64 permutações) + LSH (16 bandas × 4 linhas)**,
com os candidatos ainda validados pelo Dice exato. A invariante declarada é *recall* não decrescente
(as propostas do LSH são idênticas ou superconjunto das exatas).

### 4.4 Grafo, inferência e agregação

- **Arestas** são chaves de frontmatter (explícitas). Semântica textual é apenas *sugestão*
  (`.idx/suggestions.jsonl`), nunca aresta — separação deliberada entre observação e crença.
- **PPR/PageRank**: iteração de potência, *damping* 0.85, ≤32 iterações, tolerância L1 `1e-8`,
  arestas de autoridade `references/supports/extends/replaces`.
- **Comunidades**: Louvain determinístico (≤8 níveis, empate mantém a comunidade corrente) sobre
  arestas + âncoras partilhadas.
- **Ontologia**: classes de equivalência (`same_as`), clausura `broader`/`narrower`, deteção de
  `claim_conflicts` (mesmo `(subject, relation)`, objetos divergentes).
- **TMS/defeasible**: propaga `retracted`, `defeated_dependents` (via `depends_on` reverso) e
  `defeated_by_replacement` — **sem apagar**; a demolição apenas *propõe* (`prune`), aplica-se com
  `forget`.

### 4.5 Ciclo de vida, confiança e decaimento

- **Shelf-life** derivada da `classification`: `foundational` nunca expira, `tactical` 365 d,
  `observational` 30 d.
- **FSRS-like** (D190): cada sucesso estende o prazo em +50 % e reinicia o relógio;
  `retention(t) = limiar^(t/ttl)`, `limiar = 0.5`.
- **Confiança Beta-Bernoulli + limite inferior de Wilson** (D189): `outcomes` = ensaios
  (sucesso 1, parcial 0.5, falha 1), posterior `Beta(1+s, 1+f)`; *score* = base + limite inferior a
  95 % + feedback (0.2) + tarefa + recência (≤0.05), `clamp01`, multiplicado por `drift_factor`.
- **Drift de âncoras**: fração válida; `drift_factor = 1 − 0.5·drift ∈ (0.5, 1]`.
- **Drift de termos**: divergência de Jensen-Shannon entre metade antiga/nova do tópico; propõe
  revisão se `JS ≥ 0.5` com ≥4 notas.

### 4.6 Embeddings (opcionais, canal vetorial)

Provedor `http` por defeito (endpoint local OpenAI-compatible `http://127.0.0.1:8889/v1/embeddings`,
D202), alternativas `lightweight` (hash, para CI) e `none` (degrada para BM25). Modelo por defeito
`granite-embedding-97m-multilingual-r2` (384 d). Cliente HTTP/1.1 bloqueante escrito sobre
`std::net` (`!run_in_process`, D101). Índice em `.idx/embeddings.jsonl` com metadados de
provider/modelo/dimensões; mudança de modelo invalida e reindexa. Modo `lazy` faz *auto-drain*
ocioso por lote após cada verbo.

---

## 5. Interface: dois consumidores com o mesmo motor

| Superfície | Papel |
|---|---|
| `kd` (14 verbos: `init prime rewind ask write task map maintenance doctor drain config forget sync self`) | `stdout` = dados; `stderr` = logs. Ciclo declarado: **`ask → write → task → sync`** |
| `kd prime` | protocolo estático, **byte-idêntico por versão** (golden) — é o "help da IA" |
| `kd rewind` | estado dinâmico/handoff com **orçamento de tokens** `ceil(len/4)`, `DEFAULT_BUDGET=4000`; `context_id` (`ctx_<base36>`) permite `--resume` byte-idêntico |
| `knudge-mcp` | 4 gatilhos (`knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`) via JSON-RPC sobre stdio. **Os *hints* são ponteiros** (`id + statement + score`) — o corpo nunca entra no contexto do modelo |

Contrato de máquina: envelope `--json` = `{success, command, data?, error{code,message,retryable}, warnings?}`
e **exit codes** por `ErrorKind` (2 `invalid_input`, 3 `not_found`, 4 `conflict`, 5 `io`, 6 `timeout`,
7 `config`, 8 `schema`, 9 `unsafe_blocked`, 70 `internal`); `EPIPE` → exit 0. Escrita em lote por
JSONL (`--batch`, teto 100), `--dry-run`, auto-drain excluído em `doctor`/`drain`/`maintenance`.

---

## 6. Avaliação e evidência empírica no repositório

### 6.1 Verificação funcional

| Mecanismo | Medida nesta sessão |
|---|---|
| `make check` = `fmt --check` + `clippy -D warnings` + `test` + gate de 300 linhas | não corri (snapshot read-only; sem `target/`) |
| Testes unitários/integração | **924** `#[test]` contados por mim (`core` 698, `cli` 187, `mcp` 39); o README afirma "883 testes" |
| Proptest | 20 `proptest!` (funções puras) |
| Goldens byte-exatos | 8 ficheiros (`prime`, `prime --long`, `json` de erros, `init`, `no_results`) |
| Fuzz | 2 alvos (`toon_parse`, `jsonl_decode`) — exatamente as duas fronteiras de bytes |
| Miri | suíte mínima `tests/miri_smoke.rs` (UB/alocação/UTF-8) |
| Supply chain | `cargo deny check` + `audit` + `machete` + `typos` na CI; + *job* de *smoke* Windows |
| Estrutura | todo ficheiro `src/**` ≤ 300 linhas, verificado por script |

### 6.2 Desempenho (`bench/`, snapshots em `ULTIMO*.md`, análise em `RELATORIO.md`)

A bancada é **isolada do workspace**, sem dependências externas, e mede dois níveis: *micro*
(componentes puros, escala com `N`) e *e2e* (binário `kd` real sobre projeto temporário com corpus
sintético determinístico). Achados e correções reportados [doc]:

- **Gargalo principal identificado**: `doctor`/`compact` eram **O(N²)** (167 ms → 2,46 s de N=235 a
  N=1167) por rodar BM25 sobre o índice inteiro *para cada nota*. Corrigido com MinHash/LSH
  (**−94 %**).
- **Custo oculto**: toda a invocação pagava uma varredura O(N) no *auto-drain* ocioso (afetava até
  verbos sem relação com embeddings) — `prime`/`self version` caíram de ~40–48 ms para ~2,5–3,5 ms.
- **Sem índice persistido**: cada verbo reconstruía `Index`+`Graph` lendo todas as notas
  (`Index::build` ≈ 8 µs/nota, `Note::parse` ≈ 6,6 µs/nota) — observação relevante: o desenho
  "índice descartável" troca latência por robustez de forma consciente.
- Ganhos micro relevantes por otimização de normalização/hash (`normalize` −89 %, `body_hash` −84 %,
  `note_id` −84 %).

### 6.3 Qualidade de recuperação (Recall@k / MRR / nDCG@k)

Dataset **sintético e rotulado** (PT-BR), gerado deterministicamente, com famílias desenhadas para
isolar fenómenos: `com-acento`, `sem-acento`, `sinonimo`, `morfologia`, `working-set`. Verifiquei a
aritmética interna [código: `bench/src/quality.rs`]: `PER_TOPIC = 8` notas relevantes por tópico,
`KS = {1,3,5,10}`, relevância binária, `recall@k = |rel ∩ top-k| / |rel|`. Logo **`R@1 = 12,5 %`
= 1/8** é consistente com `MRR = nDCG@1 = 100 %` (top-1 sempre relevante) — não é contradição.

- *Baseline* de produção [doc `bench/qualidade.md`]: R@1 geral **30 %**, R@5 70 %, R@10 100 %,
  MRR/nDCG 100 %; a família `working-set` (1 relevante/consulta, canal de âncoras) tem R@1 100 %.
  O teto de nDCG@1/MRR a 100 % é o **efeito de teto** esperado de um conjunto com um relevante
  "óbvio" por consulta.
- **Ablação do stemmer** (`bench/t11_stemming.md`): a família `morfologia` passa de **0 % → 100 %**
  de nDCG@5; ganho geral **+25 %** de nDCG@5; critério de adoção declarado (limiar 20 %) → D206.
- **Sweep da fusão** (`bench/t09_fusao.md`): 8 valores de `rrf_k` × 4 de `anchor_w` × 4 de
  `semantic_w` produzem **linhas idênticas** (nDCG@5 = MRR = nDCG@1 = 100 %, R@1 = 34,4 %). A tese
  "`k=60` é inerte" fica *suportada*, mas o conjunto **não tem poder discriminativo** para
  justificar os pesos escolhidos.
- **Observação de comparabilidade**: o baseline do *sweep* reporta R@1 = **34,4 %** (≈ 11/32) e o de
  `qualidade.md` reporta **30,0 %** (18/60) para a mesma configuração nominal — denominadores
  diferentes (artefactos de tarefas distintas do épico E16). Comparações só são válidas **dentro do
  mesmo snapshot**; o denominador devia constar de cada relatório.

---

## 7. Corpus de decisões: um objeto de estudo por si

O repositório é quase um diário de investigação: **200 ADRs** com *status*, categoria, contexto
mínimo, decisão e **impacto obtido**, gerados a partir de `plan/03_decisoes-fechadas.md`; um
`DIVERGENCES.md` que regista bordas numéricas/comportamentais conscientes; `plan/implementation/`
com 12 documentos de execução (incl. matriz de aceitação por verbo); e `plan/proposals/` com
trabalho **não implementado**. Para quem estuda processo de engenharia, isto permite reconstruir
não só o *como*, mas a justificação e a sequência de revisões (ex.: D212 "revisa D158/D210").

---

## 8. Limites, riscos e questões abertas

1. **Validade externa da avaliação.** Os corpora de qualidade e de desempenho são sintéticos
   (≈70 % conhecimento, 20 % trabalho, 10 % épicos), com um provider de embeddings local; a bancada
   admite que serve para *ordenar custos* e não para números absolutos. Não há comparação com
   baselines externos (BM25 puro vs híbrido com embeddings reais, RAG de referência) nem avaliação
   *end-to-end* de utilidade para o agente (ex.: custo de contexto até à resposta correta).
2. **Poder estatístico.** `n = 12` consultas por família; nenhuma medida de variância/intervalos de
   confiança nas tabelas de qualidade; tetos saturados de nDCG@1/MRR; sweep sem discriminação
   (§6.3). É a lacuna mais evidente para quem queira publicar resultados do sistema.
3. **Latência medida em máquina de desenvolvimento** (Ryzen 5 5500U, btrfs), com variância
   admitida, e o *piso* fixo (~4–13 ms por invocação) também pesa nas ações baratas.
4. **Fragilidade de reprodução observada fora dos testes** [medido]: com o `kd` 0.6.1 instalado,
   `kd write` num diretório **sem projeto** cria `.knudge/{notas,eventos,.idx}` **sem `config.toml`
   nem `AGENTS.md`**, com `exit 0` e sem qualquer aviso, e o `doctor` responde
   `ok config config válida`; `kd ask` no mesmo contexto devolve `[no_results]` com `exit 0`
   (indistinguível de "nada encontrado"). Isto importa a quem investiga: **um corpus pode ter sido
   fundado pela metade** (config por omissão, sem `AGENTS.md`) e os scripts de medição leem "vazio"
   onde há ausência de projeto. Confirmei o mecanismo no fonte 0.5.4
   [`crates/knudge-core/src/store/mod.rs`: `Store::write`→`ensure_dirs()`; `list_ids` devolve vazio
   se `notas/` não existe].
5. **Fricções de usabilidade humana já mapeadas**: `kd help <verbo>` não mostra exemplo nem "quando
   não usar" (apesar do próprio rodapé o prometer); `kd self completions` deriva de uma lista manual
   e omite `map`/`doctor`/`drain`; a moldura do `clap` (e o erro de verbo inválido) sai em inglês
   num produto em PT-BR. O repo tem `plan/proposals/usabilidade_humana.md` (D216–D222, não
   implementada) para TTY/cor/pager/`status`/editor; deixei uma Fase 0 complementar (fundação
   explícita, `ask` sem projeto, ajuda que ensina, completions/casca em PT) em
   `exploration/usabilidade-humana-kd-fase0.md`.
6. **Contagem de testes divergente** entre README (883) e o que conta o fonte (924) — sintoma
   menor, mas do mesmo tipo de deriva que a §6.3: a documentação estática precisa de um teste de
   paridade (o repo já usa esse padrão para `prime`).
7. **Questões abertas que o desenho suscita** (não respondidas no corpus): qual o ganho real do
   canal vetorial (peso 30 é dominante face a lexical 1.0) quando o índice não é sintético; como se
   comporta o TMS *defeasible* em cadeias longas de `depends_on`; qual a taxa de conflitos de merge
   em uso multidev; e se o custo O(N) por invocação (reconstruir índice/grafo) continua aceitável à
   escala de dezenas de milhares de notas.

---

## 9. Reprodução mínima

```bash
# verificação funcional (exige toolchain 1.97+)
make check        # fmt --check + clippy -D warnings + test + gate de 300 linhas
make miri         # smoke de UB
make deny         # supply chain

# desempenho e qualidade
make bench            # micro + e2e + qualidade → bench/ULTIMO.md
make bench-quality    # Recall@k/MRR/nDCG@k → bench/qualidade.md
make bench-sweep      # rrf_k × pesos da fusão → stdout

# corpus de decisões
ls wiki/specs/adrs | wc -l      # 200 ADRs
sed -n '1,60p' wiki/specs/matematica.md
```

Nesta sessão **não** corri `make check`/`bench` (snapshot read-only, sem `target/`): os números de
§6.2/§6.3 são os artefactos versionados no repo, não medições minhas. As minhas medições foram de
*comportamento da CLI* (§8.4/§8.5), com o binário 0.6.1.

---

## 10. Glossário operacional

| Termo | Definição |
|---|---|
| **nota / nota-epic** | Unidade de verdade (Markdown + TOON); um `epic` é uma nota com `scope=epic` e sem `type` |
| **âncora** | Único elo externo da nota com o código (`--anchor PATH`); é canal de recuperação e objeto de *drift* |
| **índice derivado** | `.idx/` (BM25, âncoras, grafo, embeddings, uso, drift) — reconstruível; apagá-lo nunca perde conhecimento |
| **prime vs rewind** | `prime` = protocolo estático (byte-idêntico); `rewind` = estado dinâmico com orçamento de tokens |
| **outcomes** | Evidência de execução (`success/partial/failure/abandoned`) que alimenta confiança e BM25 |
| **TOON** | Formato de frontmatter do projeto: ordem canónica, opcionais omitidos, byte-exato |
| **RRF** | *Reciprocal Rank Fusion* — fusão por *ranks* dos canais (lexical, âncora, vetorial, PPR) |
| **D###** | Decisão de arquitetura numerada (200 no total), com status e impacto |
