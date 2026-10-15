---
question_id: Q3
same_session: true
question_issue: "Escreva o resumo do /REF/knudge, agora adaptado para um pesquisador, o resumo deve ficar em /exploration/knudge, em um único arquivo .md" 
expected_answer_ideia: "Deve ser capaz de listar categoricamente os algoritmos de otimização, modelo toon v3 de dados e usabilidade geral via kd write, kd task e kd ask, com a orientação clara de primeiro usar o ask, para evitar escrita desnecessária"
provider: opencode_go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
agent: opencode 
---

# knudge — resumo técnico para pesquisadores

**Objeto:** `knudge` (binário `kd`), v0.5.2 — sistema de memória durável por projeto, desenhado
como *retrieval substrate* para agentes de linguagem. Repositório analisado em `REF/knudge`
(~57 k LOC de Rust, edição 2024, MSRV 1.97).

**Uma frase:** um substrato de conhecimento em que **a nota é a verdade** (Markdown + frontmatter
TOON), **o índice é derivado e reconstruível**, e a recuperação é **híbrida e determinística**
(BM25 + âncoras + grafo + embeddings opcionais, fundidos por RRF).

> Nota de enquadramento: o projeto é explícito de que *"o usuário é o LLM"* — a otimização é de
> **contexto injetado**, não de armazenamento, e a legibilidade humana é secundária
> (`plan/00_panorama.md` §1). Este resumo põe a lente metodológica e reprodutível que interessa a
> quem pesquisa sistemas de recuperação e agentes.

---

## 1. Tese e posicionamento

O ponto de partida é uma inversão de prioridade: a maioria dos *knowledge systems* otimiza
**armazenamento** e depois adapta o LLM; o knudge otimiza o **contexto recuperável** e trata
persistência/índice como detalhe derivado. Três premissas guiam tudo:

1. **Consumidor = agente de linguagem.** Tokenização e determinismo são requisitos de primeira
   classe; prosa e enfeite visual são ruído.
2. **Custo = contexto, não disco.** Um índice rápido é inútil se o trecho injetado for verboso.
3. **Índice = função pura das notas.** Se `.idx/` não for 100% reconstruível a partir de
   `notas/` + `eventos/`, o design quebrou.

Consequência de engenharia: a fonte de verdade são arquivos **byte-sensíveis** (contrato TOON), e
todo o resto — BM25, grafo, vetores, cache — é material descartável.

---

## 2. Arquitetura: núcleo puro + portas + adaptadores

```
knudge-mcp ──┐
             ├── knudge-core   (núcleo puro + portas + adaptadores std)
knudge-cli ──┘
```

| Crate | Papel | Publica? |
|---|---|---|
| `knudge-core` | modelo, schema, TOON, retrieval, ciclo de vida, **portas** | biblioteca pública (D214) |
| `knudge-cli` | binário `kd`; monta adaptadores `std` e escreve a saída | só `[[bin]]` |
| `knudge-mcp` | servidor JSON-RPC 2.0 sobre stdio (tools de memória) | só `[[bin]]` |

**Princípio de inversão de dependência (D65).** O domínio nunca toca terminal, `argv`, relógio/RNG
globais nem FS. Todo acesso externo atravessa uma **porta** (`knudge_core::ports`), com fakes
determinísticos:

| Porta | Provê | Fake | Uso em pesquisa |
|---|---|---|---|
| `Clock` | `Timestamp` UTC ms | `FixedClock` | reprodutibilidade temporal |
| `Rng` | jitter | `SeqRng` | determinismo |
| `Env`, `Fs` | ambiente e I/O atômico | `FakeEnv`, `MemFs`/`FaultyFs` | **injeção de falhas** |
| `Git` | worktree/status | `FakeGit` | isolamento |
| `HookRunner` | hooks externos (sem shell) | `NoopHookRunner` | extensão |
| `Logger` | log estruturado (stderr) | `RecordingLogger` | redação de segredos |

Isso permite testar o domínio **byte a byte** e reserva o `unsafe` (hoje ausente) aos adaptadores.

---

## 3. O contrato de bytes: TOON

O frontmatter das notas usa um subconjunto documentado de **TOON** (Token-Oriented Object
Notation), escolhido por reduzir ~40–62% de tokens vs JSON em estruturas de objeto
(`wiki/specs/TOON.md`). É um contrato, não um formato de conveniência.

- **Canônico na emissão, tolerante na leitura** (Postel): uma única forma de emitir; o parser
  aceita espaços, comentários e aspas desnecessárias.
- **`null` não existe**: opcionais ausentes são **omitidos** (D05); listas/mapas vazios também.
- **Ordem é contrato**: `CANONICAL_KEYS` fixa **31 chaves**; as 12 arestas ficam entre
  `superseded_by` e `revision`.
- **Raw UTF-8, sem BOM**; aninhamento máx. 32 (defesa contra input hostil).
- **Inteiros nunca saem como `.0`** (`1.0` → `1`), e há golden para isso.

### 3.1 Identidade derivada do conteúdo (D95)

```
normalize(s) = NFC + trim + colapso de whitespace
body_hash    = hex8( SHA-256( normalize(statement) + LF + normalize(body) ) )
id           = <prefixo>_<base36(8)>( SHA-256( type + U+001F + normalize(statement) ) )
```

Duas propriedades interessantes para pesquisa:

- **Endereçamento por conteúdo estável a reclassificação.** O `id` deriva de `type` no momento da
  criação, mas o **prefixo é histórico**: mudar `type` não reescreve o `id` (borda #4).
- **Normalização explícita, não implícita.** Duas grafias Unicode equivalentes (`é` precomposto vs
  decomposto) colapsam via NFC **antes** do hash, garantindo `id`/`body_hash` idênticos (borda #1).
  Curiosamente, a recuperação lexical **dobra diacríticos** (NFD→ASCII) só no índice derivado,
  sem tocar os bytes de `notas/` (D172, borda #96).

O schema evoluiu 1→2 adicionando, de forma **aditiva**, ontologia leve (`same_as`/`broader`/
`narrower`/`related`), *claims* SPO e proveniência PROV-lite (D207). Notas v1 continuam válidas e
byte-idênticas no rebuild.

---

## 4. Recuperação: pipeline híbrido determinístico

O `recall` (`knudge-core::retrieval`) é o núcleo científico do sistema. Ordem:
**filtros → BM25 → âncoras → canais (semântico/PPR) → RRF → desempate**.

| Componente | Configuração | Referência |
|---|---|---|
| Tokenização | `[a-z0-9_]`, **fold de diacríticos** (D172) e **stemming PT conservador** (D206) | bordas #43/#96/#109 |
| BM25 | `k1=1.5`, `b=0.75`, **IDF por campo** (`statement` peso 3), peso por tipo, corte de alta frequência `df/N` | D35/D37/D173 |
| Boost por evidência | `score · (1 + 0.1·(success + 0.5·partial))` | D38 |
| Âncoras | canal por `path`/`id`, globs `?`/`*`/`**`, casamento **bidirecional** | D81/D86 |
| Fusão | **RRF** `Σ peso_c/(k+rank+1)`, `k=60`, `anchor_weight=2.0`, `semantic_weight=30` | D81/D124/D179 |
| Desempate | `(score desc, id asc)` — ordem total determinística | D81 |
| Canal PPR | PageRank/PPR personalizado, `recall.ppr_weight` (default 0) | D192 |
| Stopwords | remoção PT+EN e fragmentos de 1 char na consulta (`content_terms`) | D122 |

Pontos metodologicamente relevantes:

- **Degradação graciosa (R33):** canal opcional que falha retorna resultado parcial + `warnings[]`;
  `strict` (config de projeto, D94) promove warning a erro. Não há flag `--strict`.
- **O canal vetorial é intersectado com os filtros determinísticos** (D102, borda #26), e o rótulo
  `why` reflete o **canal de origem**, não só metadados (`semantic` vence `recent`, borda #42).
- **Contrato de observabilidade:** o `--json` expõe por hit
  `channels: {lexical, anchor, semantic, recent, stars}` cujas três primeiras **somam o score**
  RRF (D151, borda #65) — um vetor de atribuição de crédito por canal, útil para análise.
- **Busca vazia é estado, não erro:** `[no_results]` literal, exit 0 (D152).

### 4.1 Embeddings: provedor HTTP assíncrono, opcional

Os vetores **não** são o mecanismo principal; servem a dedup semântico, descoberta de links e
clusterização. O design é deliberadamente conservador (D101/R16/R43):

- Provedor `http` (default) fala com um servidor local **OpenAI-compatible** (`llama-server`/TEI/
  Ollama) — **sem inferência in-process** e sem runtime de IA no binário. `lightweight` (hash) e
  `none` (BM25) completam o espaço.
- Índice `.idx/embeddings.jsonl` com cabeçalho `meta` (provider/model/revision/dims); **trocar de
  modelo invalida** e força re-embed (D79).
- Fila **lazy** com gap tolerado: notas recém-criadas ficam *dark* até o drain, acháveis só por
  BM25/estrutura nessa janela; falha marca `pending`, **nunca descarta** (D80/D83).
- Cache por `body_hash` com eviction LRU (D83), *dirty flag*/debounce (D85).
- Default multilíngue `ibm-granite/granite-embedding-97m-multilingual-r2` (384d, Apache-2.0),
  escolhido por A/B em corpus PT-BR (D123): **nDCG@5 0.845 vs 0.771 do BM25** e **0.775** do modelo
  inglês anterior.

Custo de armazenamento: 384d·f32 ≈ 1.5 KB/nota; *brute-force* cosseno fica submilissegundo até
~10⁴ notas — o que motivou a manter ANN fora do default. O plano de **reranking + ANN** está
documentado em `plan/proposals/reranking_ann.md` (E19/T06–T08).

---

## 5. Grafo de conhecimento e raciocínio não-monotônico

As arestas são explícitas no frontmatter (a nota é a verdade, D49/D98): **12 tipos** —
navegação (`references`, `extends`, `results_in`), causalidade/suporte (`depends_on`, `supports`,
`contradicts`), versionamento (`replaces`, `rejects`, `superseded_by`) — mais ontologia leve
(D207).

Componentes algorítmicos (`knudge-core::graph`):

| Algoritmo | Papel | Determinismo |
|---|---|---|
| SCC (Kosaraju iterativo) | ciclos de `replaces`/`depends_on`; membros **protegidos de demolição** (D45) | ordem canônica |
| Equivalência + clausura transitiva | `same_as` forma classes; `broader`/`narrower` inversos; detecta ciclos de hierarquia (D207) | `BTreeMap` |
| **PageRank/PPR** | autoridade/rank; *dangling* redistribuído; invariante a relabeling (D192, borda #101) | iteração em ordem canônica |
| **Louvain** (comunidades) | partição do grafo; empates mantêm comunidade corrente (D193, borda #102) | sem RNG, nós canônicos |
| **TMS** (truth maintenance) | dependentes transitivos de premissas **retratadas** (`forgotten`/`superseded`) e derrotados por `replaces`; **sem apagar nada** (D208) | índice reverso de `depends_on` |

O TMS e o *drift* de termos (KL/JS por âncora, D208) são os elementos mais próximos de raciocínio
não-monotônico: a retratação de uma premissa **propaga** a derrota para dependentes, e mudanças de
vocabulário num tópico são detectadas como *drift* — sempre como **propostas**, nunca mutações
silenciosas (D47/D112).

---

## 6. Escrita, dedup e ciclo de vida

O `write` é **idempotente por conteúdo** e opera em duas fases (propose → decide → apply, D26):

| Faixa de score (Dice) | Ação |
|---|---|
| `< 0.75` (`create_below`) | cria nova nota |
| `[0.75, 0.92)` (`merge_below`) | **merge** no candidato (tags/âncoras unidas, corpo acrescido, `revision++`) |
| `≥ 0.92` | **rejeita** |

- O score lexical é **Dice** sobre termos; em corpus denso, *blocking* por **MinHash/LSH** reduz
  pares candidatos sem alterar o limiar (D204, borda #107) — resposta direta ao gargalo O(N²)
  medido na bancada.
- **Confiança derivada, nunca armazenada**: posterior **Beta-Bernoulli** (`sim`, evidência, drift,
  idade) com limite inferior conservador, sempre em `[0,1]` (D87/D189/D203, bordas #98/#103).
- **Shelf-life com revisão espaçada** (FSRS-like, D190): `foundational` nunca expira; `tactical`
  365d; `observational` 30d; cada `outcome` de sucesso **estende** o prazo (mas nunca encurta).
- **Demolição sempre soft** (`forget`), com motivos tipados: `expired`, `anchor_decay`,
  `contradicted`, `defeated` (TMS), `drifted` (KL/JS). Purga física só após janela de retenção e
  limpa arestas de entrada (borda #48).

Interessante para pesquisa: **evidência confirmatória derivada de tarefas** (D108) — tarefas com
`outcomes` de sucesso que compartilham âncoras com uma nota **confirmam** essa nota em tempo de
consulta, sem escrever nada. A confirmação é saturante e não depende de distância no grafo (o
vínculo é a âncora, borda #35).

---

## 7. Handoff e orçamento de contexto

`rewind` (D57) é o mecanismo de reconstrução de contexto, com três modos (manifest, escopo,
working set) e **orçamento em tokens** `ceil(chars/4)`, default 4000 (D40/D82). O `context_id` é
derivado do texto e `--resume` devolve **bytes idênticos** (D88). O ranking combina estrelas,
classificação e recência com desempate total. O corpo só entra para `foundational`/`decision` e
passa pelo mesmo orçamento (borda #92).

---

## 8. Protocolo de máquina e interface de agentes

- **Superfície determinística de 14 verbos** (`plan/implementation/16_cli_surface.md`, D209):
  `init, prime, rewind, ask, write, task, map, doctor, drain, maintenance, config, forget, sync,
  self`. O `prime` é **byte-idêntico por versão** (estático, cacheável); `rewind` é o estado
  dinâmico.
- **Contrato de I/O:** stdout = dados, stderr = logs (R20); `--json` é o envelope
  `{success, command, data?, error{code,message,retryable}, warnings?}` (D71/R31). EPIPE → exit 0
  (D73). Exit codes congelados (`not_found=3, invalid_input=2, conflict=4, io=5, timeout=6,
  config=7, schema=8, internal=70`; 101 reservado a panic).
- **MCP (`knudge-mcp`)**: JSON-RPC 2.0 sobre stdio, uma linha por mensagem, com 4 gatilhos
  (`knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`). Os *hints* são
  **ponteiros** (`id + statement + score`), nunca o corpo — o contexto do modelo não é poluído
  (D68).

---

## 9. Metodologia de verificação (o que interessa a quem pesquisa)

O projeto trata **determinismo byte a byte** como invariante central e construiu uma bateria
incomum de defesas. `make check` = fmt + clippy `-D warnings` + testes + gate de 300 linhas;
`make ci` soma `miri`, `cargo-deny`, `cargo-audit`, `cargo-machete` e `typos`.

| Camada | O que trava | Onde |
|---|---|---|
| **Golden** | bytes de `prime`/`--json`/erros/`init`/EPIPE | `crates/knudge-cli/tests/golden/` |
| **Property (proptest)** | TOON round-trip, RRF, decay/confiança, id/hash sob normalização | `src/<mod>/tests*`, `proptest-regressions/` |
| **Stress** | lock sem *lost update*, escrita concorrente, leitor × rebuild | `tests/stress.rs` |
| **Crash-injection** | nota-antes-de-evento, tmp+rename, rebuild double-buffer | `FaultyFs` |
| **Dinâmica** | UB no core (miri), fuzz TOON/JSONL, supply-chain | `.github/`, `fuzz/`, `deny.toml` |
| **Bancada** | latência (medianas) e qualidade (Recall@k/MRR/nDCG) | `bench/` |

**883 testes** + proptests + goldens. Invariantes de engenharia: `#![forbid(unsafe_code)]`,
proibição de `unwrap`/`expect`/`panic` em `src/`, arquivos de produção ≤ 300 linhas, e
**proibição de `HashMap`/`HashSet`** no core (só `BTreeMap`/`IndexMap`) para garantir ordem de
iteração determinística.

### 9.1 `DIVERGENCES.md` como *harness* diferencial substituto

O achado metodológico mais interessante: como não há um produto de referência para *differential
testing*, o projeto mantém um **catálogo de 115 bordas** onde a implementação poderia divergir de
si mesma (Unicode, ordem de chaves, tie-breaks, atomicidade, timestamps, lock, framing MCP…),
cada uma com **risco → mitigação → o teste que a trava**. Toda mudança que toque uma linha precisa
manter o teste verde. É, na prática, um *metamórfico/diferencial* organizado por contrato.

### 9.2 Evidência empírica de performance

Da bancada (`bench/RELATORIO.md`), corpus sintético de 235 e 1167 notas:

- **Gargalo crítico era O(N²)**: `doctor`/`compact` iam de 167 ms (N=235) a ~2.5 s (N=1167) por
  causa de dedup pairwise. Resolvido com **MinHash/LSH** (D204): queda de **−94%** no estado final.
- **Varredura O(N) escondida** no auto-drain ocioso afetava todos os comandos; desligá-la cortava
  `prime`/`self version` de ~40–48 ms para ~13 ms.
- **Leitura paralela do corpus** com `std::thread::scope` (zero-dep, sem `rayon`) e remontagem na
  ordem de `list_ids` (bytes idênticos): **−65%** (21→7.4 ms); `ask` **−40%**.
- Várias otimizações propostas (`memchr`, `smallvec`, `rayon`, `mimalloc`, índice binário) foram
  **rejeitadas por medição** — disciplina empírica explícita (R43).

### 9.3 Cobertura de qualidade

Baseline PT-BR rotulado (192 notas, 60 consultas) em `bench/qualidade.md`: **nDCG@5 = 100% em
todas as famílias**, R@10 = 100%, MRR = 100%. A própria bancada reconhece que **está saturada** —
não há *headroom* para medir ganho de reranking, o que motivou a exigir "famílias difíceis" antes
de qualquer A/B (E19).

---

## 10. Limitações e problemas em aberto

1. **Bancada de qualidade saturada** (100% nDCG@5) impede medir melhorias de ranking; precisa de
   distratores/consultas longas/paráfrases distantes.
2. **Reranking e ANN bloqueados** por dependências externas: um *cross-encoder* local
   (`bge-reranker-v2-m3` etc.) e escala de corpus ≥ 10⁴ notas (E19/T06–T08). Política de reversão:
   sem ganho ≥ 20% em qualidade (ou −20% latência) com o mesmo output, a mudança é revertida
   (R43).
3. **`rewind` ainda caro** (lê o corpo de todas as notas; ~362 ms em N=1167 antes das otimizações);
   impacto/travessia de grafo dentro do comparador de ordenação.
4. **Reconstrução de `Index`+`Graph` a cada invocação** em vez de usar o `.idx/` derivado com
   invalidação barata.
5. **Dedup no write é lexical** enquanto não há vetores (dedup semântico é *eventual*); o modelo
   default é multilíngue, mas a assimetria query→passage e o viés de idioma pedem A/B no corpus
   real.
6. **Sem prova formal** dos invariantes de determinismo — são garantidos por construção (tipos
   proibidos) e travados por proptest/golden, não por *model checking*.

---

## 11. Como manter e reproduzir

- **Decisões:** `plan/03_decisoes-fechadas.md` (D01–D209+); **políticas:** R01–R44
  (`14_revisao_tecnica.md`).
- **Bordas:** `wiki/specs/DIVERGENCES.md`; **arquitetura:** `wiki/specs/ARCHITECTURE.md`;
  **contrato de bytes:** `wiki/specs/TOON.md`.
- **Reproduzir bancada:** `make bench` / `make bench-quick`; qualidade com
  `make bench-quality`.
- **Extensão como biblioteca:** fachada `knudge_core::Knudge` (`Knudge::builder().knowledge_dir(".a/b").open()?`,
  D214), com layout configurável e defaults que preservam o comportamento histórico.

---

## 12. Síntese

O `knudge` é um estudo de caso de **substrato de recuperação determinístico e auditável** para
agentes: um contrato de bytes mínimo (TOON), identidade endereçada por conteúdo, recuperação
híbrida com fusão RRF ponderada e atribuição de crédito por canal, ciclo de vida com confiança
bayesiana e revisão espaçada, raciocínio não-monotônico via TMS, e uma metodologia de verificação
que trata cada borda de determinismo como um teste. Seus méritos de pesquisa estão menos no
retrieval bruto (que reconhecidamente *satura* a bancada atual) e mais na **disciplina de
contrato, reprodutibilidade e medição empírica** — incluindo reverter otimizações que não se pagam.
Os problemas abertos (reranking, ANN, escala de avaliação) são exatamente onde o sistema cruza do
"artefato bem construído" para o "sistema adaptativo".
