# knudge — Resumo para Investigador

> **Projeto:** REF/knudge  
> **Versão analisada:** 0.5.4  
> **Linguagem:** Rust 1.97+ (edição 2024)  
> **Licença:** MIT OR Apache-2.0  

---

## 1. Contexto e Motivação

O knudge responde a um problema crescente nos sistemas de IA baseados em LLMs: a **perda de contexto entre sessões** e a **ausência de memória durável e auditável** nos fluxos de trabalho com agentes. Ao contrário de soluções baseadas em RAG sobre documentos, o knudge posiciona-se como **memória de projeto** — um repositório de conhecimento versionado, consultável e auditável que vive ao lado do código-fonte.

A premissa central é que a **memória de um agente deve ser tratada como código**: versionada, revisável, extensível e reconstrutível.

---

## 2. Problema de Investigação

Como projetar um sistema de memória durável para agentes de IA que seja:

1. **Auditável** — conhecimento explícito e versionado, não oculto em embeddings opacos
2. **Reconstrutível** — o índice pode ser apagado e refeito sem perda de conhecimento
3. **Determinístico** — comportamento reproduzível, sem dependências externas não controladas
4. **Offline-first** — sem servidor, sem base de dados, sem daemon
5. **Integrável** — expõe-se por CLI e MCP (Model Context Protocol)

---

## 3. Arquitetura

O knudge divide-se em três crates:

| Crate | Papel |
|---|---|
| `knudge-core` | Núcleo puro: modelo, schema, retrieval, ciclo de vida, portas |
| `knudge-cli` | Binário `kd` — monta adaptadores e produz a saída |
| `knudge-mcp` | Servidor MCP — gatilhos por JSON-RPC 2.0 sobre stdio |

### 3.1 Separação de preocupações

O domínio depende exclusivamente de **portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`), nunca de chamadas diretas ao sistema operativo. Permite testes determinísticos com fakes (`FixedClock`, `SeqRng`, `MemFs`, `FakeGit`, `RecordingLogger`, `FakeEmbedder`).

### 3.2 Camadas de persistência

| Camada | Conteúdo | Natureza |
|---|---|---|
| `notas/` | Arquivos Markdown com frontmatter TOON | **Verdade** — versionada |
| `eventos/` | Log de auditoria em JSONL | Auditoria — versionada |
| `.idx/` | Índice invertido, grafo, sugestões, drift | **Derivado** — descartável e reconstruível |
| `.knudge/` | Config, cache vetorial, validators | Auxiliar — versionado parcialmente |

A decisão de **separação entre verdade e derivado** é central: o índice é uma otimização, nunca a fonte. Se o índice divergir, é reconstruído (`kd doctor --fix`).

---

## 4. Contribuições Técnicas

### 4.1 Contrato de bytes (TOON)

Formato próprio de serialização (TOML-like) com:

- Ordem canónica de chaves (contrato de bytes)
- Tolerância na leitura, rigidez na escrita
- Tipos escalares Unicode sem BOM
- Idempotência sob retry (`body_hash` SHA-256 truncado)

O TOON elimina ambiguidade de serialização e garante que o mesmo conteúdo gera sempre os mesmos bytes — crucial para versionamento e deduplicação.

### 4.2 Deduplicação escalável

Duas fases:

1. **Recall lexical** de candidatos (BM25)
2. **Decisão** com base em limiares configuráveis:
   - `< 0.75` → cria
   - `0.75–0.92` → merge (união de tags/âncoras/outcomes)
   - `≥ 0.92` → rejeita (duplicata)

Para corpus densos (>256 notas), o knudge usa **MinHash (64 permutações) + LSH (16 bandas × 4 linhas)** para bloquear candidatos, mantendo o Dice exato como verificação final. Propostas são idênticas ou superconjunto (recall ≥).

### 4.3 Busca híbrida determinística

Cascata de canais:

```
filtros → BM25 → âncoras → vetorial → PPR → fusão RRF → penalidade de contradição → confiança/idade/drift
```

| Canal | Peso RRF | Descrição |
|---|---|---|
| `lexical` | 1.0 | BM25 com IDF Robertson-Sparck Jones |
| `anchor` | 2.0 | Interseção por path/glob |
| `semantic` | 30.0 | Cosseno sobre embeddings (opcional) |
| `ppr` | 0.0 | PageRank/Personalized PageRank (default off) |

Determinismo garantido por: `BTreeMap`/`IndexMap` (proibição de `HashMap`/`HashSet`), tie-breaks explícitos, e iterações determinísticas (PageRank, Louvain).

### 4.4 Ciclo de vida e confiança

**Shelf-life** derivado da classificação:

| Classificação | TTL |
|---|---|
| `foundational` | nunca expira |
| `tactical` | 365 dias |
| `observational` | 30 dias |

**FSRS-like** (D190): cada sucesso estende o prazo em 50% e reseta o relógio. Renovação por uso (D154) apenas estende, nunca reduz.

**Confiança Beta-Bernoulli + Wilson** (D189): outcomes como ensaios; limite inferior 95% (Z=1.96) como confiança conservadora.

### 4.5 Grafo e ontologia leve

- **Arestas explícitas** (12 tipos fechados) — extração textual é sugestão, nunca aresta
- **Ontologia** (D207): `same_as`, `broader`, `narrower`, `related` — com detecção de ciclos e conflitos de claims
- **TMS defeasible** (D208): retracted (premissas esquecidas), defeated_dependents, defeated_by_replacement
- **Comunidades** (D193): Louvain determinístico sobre arestas + âncoras partilhadas

### 4.6 MCP (Model Context Protocol)

Integração com agentes externos via 4 gatilhos:

| gatilho | Momento |
|---|---|
| `knudge_pre_write` | Antes de gravar (recall de duplicatas) |
| `knudge_pre_edit` | Antes de editar (verificação de conflitos) |
| `knudge_session_end` | Fim de sessão (rewind/handoff) |
| `knudge_status` | Consulta de estado |

Os hints são **ponteiros** (id + statement + score) — o corpo fica no `kd`, nunca no contexto do modelo. Reduz tokens no contexto do agente.

---

## 5. Metodologia e Rigor

### 5.1 Disciplina de código

- `forbid(unsafe_code)`
- Sem `unwrap`, `expect`, `panic`
- Arquivos de produção ≤ 300 linhas
- Iteração determinística (`BTreeMap`/`IndexMap`)

### 5.2 Testes

- **883 testes** unitários e integração
- `proptest` nas funções puras
- **Goldens** para o contrato de bytes (TOON)
- **Matriz de aceite** (`17_matriz_aceitacao.md`) — pipe, `--json`, erro e exit por comando
- **Bordas documentadas** (`DIVERGENCES.md`, #1–#115) — cada borda tem mitigação + teste que a trava

### 5.3 Auditoria

- Log de eventos JSONL (`eventos/events.jsonl`) com `{id, op, note_id?, at, actor?, data?}`
- Escrita atómica (tmp + rename + fsync)
- Crash recovery: nota sem evento é recuperável

---

## 6. Avaliação e Limitações Conhecidas

### 6.1 Forças

- **Determinismo total** — reproduzível em qualquer ambiente
- **Sem dependências externas críticas** — funciona sem embeddings
- **Contrato de bytes** — auditável e extensível
- **Separação verdade/derivado** — resistente à corrupção do índice

### 6.2 Limitações

- Embeddings requerem servidor local (OpenAI-compatible)
- `learn`/`compact`/`prune` são read-only (propostas)
- Hierarquia de tarefas limitada a `epic ⊃ {issue ⊃ task | task}`
- Não substitui RAG sobre documentos grandes
- Não tem controlo de acesso (ACL) — é local e versionado por git

---

## 7. Implicações para Investigação

O knudge é relevante para várias áreas de investigação:

1. **Memória em LLM agents** — estudo de como persistir e recuperar conhecimento entre sessões
2. **Deduplicação semântica** — técnicas de MinHash/LSH para corpus densos
3. **Busca híbrida** — fusão RRF com canais determinísticos e semânticos
4. **Ciclo de vida de informação** — FSRS-like, shelf-life, confiança derivada de evidência
5. **Contratos de serialização** — TOON como alternativa a TOML/JSON para contratos de bytes
6. **Arquitetura port-adapter** — separação de domínio e infraestrutura em Rust

O repositório é também um **estudo de caso** em documentação técnica: decisões (D01–D212), bordas (#1–#115), specs e guiões de uso mantidos em sincronia.

---

## 8. Conclusão

O knudge demonstra que é possível construir **memória durável e auditável para agentes de IA** sem sacrificar determinismo, offline-first ou simplicidade operacional. A separação entre notas (verdade) e índice (derivado), o contrato de bytes TOON e a busca híbrida determinística são contribuições técnicas sólidas com aplicação imediata em contextos de desenvolvimento de software com agentes.

---

*Resumo gerado a partir da análise do repositório REF/knudge, incluindo README, SKILL.md, llms.txt, wiki/usage/, wiki/specs/, plan/ e AGENTS.md.*
