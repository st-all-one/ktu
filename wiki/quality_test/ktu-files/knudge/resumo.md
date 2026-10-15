# knudge — resumo para investigador

> **knudge** é uma CLI Rust (binário `kd`, v0.5.4) que implementa **memória durável por projeto
> para agentes de IA**, usando notas em Markdown como fonte da verdade e um índice derivado
> reconstruível. Sem servidor, sem banco de dados, sem daemon. O repositório está em
> `REF/knudge/`.

---

## 1. Problema de investigação

O knudge endereça o problema da **memória de projeto em fluxos de trabalho com agentes de IA**:
quando uma sessão termina, o contexto evapora; o conhecimento fica espalhado em conversas,
duplicado, contraditório e obsoleto. A literatura de RAG e memória de LLM trata memória como
recuperação sobre documentos externos; o knudge trata memória como **conhecimento versionado,
auditável e acoplado ao código**, vivendo no repositório.

**Questões de investigação que o projeto levanta:**

- Como manter um corpus de conhecimento **mergeável** (git) e **deduplicável** em escrita?
- Como derivar confiança e obsolescência **sem armazená-las** (sempre derivadas do corpus)?
- Como ligar conhecimento a código de forma que **drift** seja detetável?
- Como manter **determinismo** (mesma entrada → mesma saída) num sistema com tempo, RNG e FS?
- Como expor memória a agentes sem inchar o contexto do modelo?

---

## 2. Arquitetura

Três crates, separação estrita entre domínio puro e adaptadores:

| Crate | Papel |
|---|---|
| `knudge-core` | Modelo, schema, TOON, retrieval, ciclo de vida, **portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`). Nunca acede a SO diretamente. |
| `knudge-cli` | Binário `kd`: clap, envelope `--json`, logging, montagem de adaptadores. |
| `knudge-mcp` | Servidor MCP (JSON-RPC 2.0 sobre stdio): 4 gatilhos (`knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`). |

**Princípio central:** o domínio depende de **portas**, não de sistema operativo. Testes do core
usam fakes (`FixedClock`, `SeqRng`, `MemFs`, `FakeGit`, `RecordingLogger`, `FakeEmbedder`).

---

## 3. Modelo de dados

- **Nota** = frontmatter TOON + corpo Markdown, em `notas/<tipo>/<id>.md`.
- **Identidade derivada do conteúdo:** `id = <prefixo>_<base36(8)>` onde o hash é
  `SHA-256(type + U+001F + normalize(statement))` truncado a 4 bytes. Reclassificar `type` **não**
  reescreve o `id` (decisão D01/D02/D95).
- **31 chaves canônicas** em ordem fixa; opcionais **omitidos**, nunca `null`.
- **Enums fechados:** `NoteType` (10), `Scope` (3), `Classification` (3), `Status` (6),
  `EdgeKind` (12).
- **Extensões aditivas (schema v2):** `claims` (SPO), `provenance` (PROV-lite), ontologia leve
  (`same_as`/`broader`/`narrower`/`related`).
- **TOON:** formato próprio, raw UTF-8, ordem canônica, hashes e ids como **contrato de bytes**.

---

## 4. Persistência e integridade

- **Verdade:** `notas/` (versionado). **Auditoria:** `eventos/events.jsonl`. **Derivado:**
  `.idx/` (índice, âncoras, drift, sugestões, checkpoint).
- **Escrita atómica:** tmp + rename no mesmo dir; `fsync` em batch.
- **Ordem de commit:** nota primeiro, evento depois (crash deixa nota sem evento — recuperável).
- **Lock advisory** por arquivo-alvo (`O_CREAT|O_EXCL`, stale 30 s, reclaim por rename).
- **Rebuild double-buffer:** escreve `.idx.new/` e troca por rename — leitor vê antigo ou novo,
  nunca parcial.
- **Purga/detach/sweep:** remoção dispara `purge_derived` + detach de arestas; sweep remove só
  `*.tmp`/`*.stale` > 30 s.

---

## 5. Busca e ranking

Cascata determinística:

```
filtros → BM25 → âncoras → vetorial → PPR → fusão RRF → penalidade de contradição
        → confiança/idade/drift → snippet/why
```

- **BM25:** `k1=1.5`, `b=0.75`; IDF Robertson-Sparck Jones por campo; pesos `statement 3.0` /
  `tags 2.0` / `body 1.0`; `type_weight` por espécie; boost por confirmação `×(1+0.1·conf)`.
- **Tokenização:** ASCII → fold de diacríticos (NFD) → stopwords PT+EN → stemming PT conservador.
- **RRF:** `Σ peso_c/(k+rank+1)`, `k=60`; pesos `lexical 1.0`, `anchor 2.0`, `semantic 30.0`,
  `ppr 0.0`; desempate `(score desc, id asc)`.
- **Âncoras:** canal por path/glob; só entra quando intersecta o working set.
- **`why` fechado:** `file_match > anchor_match > tracker_match > stars > semantic > recent >
  universal`.
- **Revelação progressiva:** 1º hit corpo completo, 2–5 truncado a `preview_chars=280`, resto
  `id|statement|score|why`.

---

## 6. Escrita e dedup

- **Duas fases:** recall lexical de candidatos → decisão (`<0.75` cria, `0.75–0.92` merge,
  `≥0.92` rejeita).
- **Dedup escalável (D204):** corpus pequeno → peneira exata de postings; >256 notas densas →
  **MinHash (64 perm.) + LSH (16 bandas × 4 linhas)**; candidatos ainda passam pelo Dice exato.
- **Merge:** união de tags/âncoras/outcomes, `revision++`, corpo não sobrescrito.
- **Update/supersede:** `Patch` versionado; chave nova → novo id + `replaces`/`superseded_by`
  (bidirecional).
- **Lote:** `write --batch -` (JSONL) e `task new --batch`, best-effort com `warnings[]`.

---

## 7. Grafo e inferência

- **Arestas explícitas** = chaves de frontmatter de 1º nível; extração textual é **sugestão** em
  `.idx/suggestions.jsonl`, nunca aresta.
- **Integridade:** referências quebradas, órfãos, bidirecionalidade `replaces ↔ superseded_by`;
  SCC (Kosaraju) sobre `replaces`; membros de ciclo **não demovem**.
- **PageRank/PPR:** iteração de potência, damping `0.85`, ≤32 iter, tol L1 `1e-8`.
- **Comunidades:** Louvain determinístico (≤8 níveis, ≤64 passos) sobre arestas + âncoras
  compartilhadas.
- **TMS/defeasible:** `retracted` (premissas `forgotten`/`superseded`), `defeated_dependents`
  (transitivo via `depends_on` reverso), `defeated_by_replacement`. Sem apagar — motivo `Defeated`
  no `prune`.

---

## 8. Ciclo de vida e confiança

- **Shelf-life derivado da `classification`:** `foundational` nunca expira; `tactical` (365 dias) /
  `observational` (30 dias).
- **FSRS-like (D190):** cada sucesso estende o prazo em `retention.growth_percent=50%` e reseta o
  relógio; `retention(t)=limiar^(t/ttl)`, `limiar=0.5`.
- **Confiança Beta-Bernoulli + Wilson:** outcomes = ensaios (`success=1`, `partial=0.5`,
  `failure/abandoned`=1 falha); posterior `Beta(1+s,1+f)`; **limite inferior 95%** (Wilson,
  `Z=1.96`) é a confiança conservadora.
- **Decay/drift de âncoras:** fração válida; demove após grace se `< threshold`; `drift = 1−fração`;
  `drift_factor = 1−0.5·drift ∈ (0.5,1]`.
- **Drift de termos:** JS (base 2, smoothing 1e-9) entre metade antiga/nova por tópico; propõe
  revisão se `JS ≥ 0.5` e `≥ 4` notas.
- **Demolição:** só propõe (`Expired`/`AnchorDecay`/`Contradicted`/`Defeated`/`Drifted`);
  membros de ciclo protegidos; aplicação via `kd forget`.

---

## 9. Tarefas

- **Tarefa = view derivada de notas** (mesmo formato/store). `scope` = nível (`epic`/`issue`/
  `task`); `type` = espécie (`task` default; `--kind` = `task error question risk decision`).
- **Hierarquia fechada:** `epic ⊃ { issue ⊃ task | task }` — épico é a raiz, issue opcional, pai
  único, profundidade ≤ 4.
- **Derivado (nunca armazenado):** papel, modo, progresso, `epic_of`, impacto, contexto, fluxo.
- **Status:** `active/in_progress/blocked/closed`; fechamento com evidência via `task close`.
- **Ops:** `new/list/show/update/close/graph/plan/flow`; `--ready`/`--blocked`/`--explain`/
  `--sort impact`; `task plan --prompt|--submit` (atómico: valida tudo antes de gravar).

---

## 10. Embeddings (opcional)

- **Provedor** `http` (default; OpenAI-compatible local, endpoint `http://127.0.0.1:8889/
  v1/embeddings`) / `lightweight` (hash, CI) / `none` (BM25).
- **Modelo default:** `granite-embedding-97m-multilingual-r2` (384d).
- **Índice** `.idx/embeddings.jsonl` com `meta`; mudança de modelo invalida e re-embeda.
- **Cache** por `(body_hash, model)` com LRU; versionável em `.knudge/emb_cache.jsonl`.
- **Estado** `indexed|pending|stale` derivado; falha marca `pending` (nunca descarta).
- **Modo** `lazy` (default; drena 1 lote best-effort após cada verbo) / `manual`.
- **Worker:** `scripts/knudge-idle.sh` + `kd drain service`; `systemd --user`/`launchd`/cron;
  GGUF de revisão pinada + SHA-256.

---

## 11. Superfície da CLI

14 verbos: `init|prime|rewind|ask|write|task|map|maintenance|doctor|drain|config|forget|sync|self`
(+ `help`; `kd` = `help`).

**Contratos:**

- **stdout = dados; stderr = logs.** `--json` = `{success, command, data?, error?, warnings?}`.
- **Listas (D210):** repetição ≡ vírgula (`--tag a,b` = `--tag a --tag b`); sem forma por espaço.
- **Conjuntos fechados (D212):** valor inválido → lista + "você quis dizer" (Levenshtein).
- **Escopo obrigatório (D144):** `map`/`ask --rank`/`maintenance learn|compact|prune`/`task list`
  exigem filtro ou `--universe`, senão exit 2.
- **Exit codes:** `0` ok · `2` invalid usage · `3` not found · `4` conflict · `5` io ·
  `6` timeout · `7` bad config · `8` bad schema/note · `70` internal (`101` panic).
- **EPIPE → exit 0** (D73).

---

## 12. Estado atual

- **Versão:** 0.5.4.
- **Testes:** 883 testes + `proptest` nas funções puras + **goldens** para o contrato de bytes.
- **Regras duras:** sem `unwrap`/`expect`/`panic`/`unsafe` em `src/`; arquivos ≤ 300 linhas;
  iteração determinística (`BTreeMap`/`IndexMap`).
- **Toolchain:** Rust 1.97+ (edição 2024).
- **Documentação:** `wiki/usage/` (guias), `wiki/specs/` (técnica), `wiki/specs/adrs/`
  (D01–D212), `plan/` (decisões e implementação).

---

## 13. Contribuições científicas

1. **Identidade derivada de conteúdo** — ids estáveis por afirmação, mergeável por git, sem
   coordenação central.
2. **Índice derivado e descartável** — a verdade vive em texto legível; o índice é sempre
   reconstruível.
3. **Dedup escalável** — MinHash + LSH com peneira exata, mantendo determinismo.
4. **Confiança derivada** — Beta-Bernoulli + Wilson, sem armazenar confiança.
5. **Drift de âncoras e termos** — deteção de obsolescência por mudança de código e vocabulário.
6. **TMS/defeasible** — derrota por substituição e retração sem apagar.
7. **Determinismo total** — portas para tempo/RNG/FS, fakes em testes, iteração canónica.

---

## 14. Limites e ameaças à validade

- **Embeddings precisam de servidor local** — sem ele, `ask` degrada para BM25.
- **`forgotten`/`superseded` ficam fora do `ask` por padrão** — risco de esquecimento
  permanente se não forem incluídos explicitamente.
- **`learn`/`compact`/`prune` são read-only** — propõem, nunca aplicam; o humano/agente deve
  aceitar.
- **Tarefas rootam no épico** — hierarquia rígida pode não modelar todos os fluxos de trabalho.
- **Corpus pequeno** — abaixo de ~50 notas, um README ou ADR pode ser mais simples.
- **Sem ACL nem servidor** — memória local e versionada, sem controlo de acesso.
- **Dependência de git** — sem git, perde versionamento e merge.

---

## 15. Pontos de investigação futuros

- **Avaliação empírica** da dedup (precisão/recall do MinHash + LSH vs. Dice exato).
- **Impacto do drift** na qualidade de recuperação ao longo do tempo.
- **Comparação com RAG clássico** em memória de projeto (custo, precisão, auditabilidade).
- **Extensão da ontologia leve** para inferência mais rica (regras, restrições).
- **Integração com mais clientes MCP** (além de Claude, Cursor, Codex, Pi).
- **Estudo de usabilidade** — a superfície atual é otimizada para agentes; humanos precisam de
  tradução (ver recomendação de `kd assist`).

---

## 16. Como reproduzir

```bash
# Instalar
curl --proto '=https' --tlsv1.2 --show-error --fail \
  https://raw.githubusercontent.com/st-all-one/knudge/main/install.sh | bash

# Verificar
kd self version
kd --help

# Testar (do source)
make check     # fmt + clippy + test + gate de 300 linhas
make test      # cargo test --workspace
make build     # cargo build --workspace
```

**Documentação de referência:**

- `README.md` — instalação + quickstart.
- `SKILL.md` — guia para agentes.
- `AGENTS.md` — contribuir no código.
- `wiki/usage/` — guias de uso (um por comando).
- `wiki/specs/` — especificações técnicas.
- `wiki/specs/adrs/` — decisões D01–D212.
- `plan/` — decisões e implementação.
- `llms.txt` — índice canónico para LLMs.

---

## 17. Conclusão

O knudge é uma implementação rigorosa de **memória durável por projeto** para agentes de IA,
com forte ênfase em **determinismo**, **auditabilidade** e **mergeabilidade**. A arquitetura
baseada em portas e o modelo de dados derivado do conteúdo são contribuições relevantes para a
investigação em memória de LLM. A superfície atual é otimizada para agentes; a usabilidade
humana é o próximo desafio de design.

---

*Resumo gerado em 2025-10-15 a partir de `REF/knudge/` (v0.5.4).*
