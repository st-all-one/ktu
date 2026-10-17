---
question_id: Q3
question_issue: "Escreva o resumo do /REF/knudge, agora adaptado para um pesquisador, o resumo deve ficar em /exploration/knudge, em um único arquivo .md"
expected_answer_ideia: "Deve ser capaz de listar categoricamente os algoritmos de otimização, modelo toon v3 de dados e usabilidade geral via kd write, kd task e kd ask, com a orientação clara de primeiro usar o ask, para evitar escrita desnecessária"
provider: opencode-go
model: deepseek-v4.1-flash
thinking-level: low
cache: true
same_session: true
compacted: false
agent: ktu
---
Resumo escrito em **`exploration/knudge/ktu/knudge.md`** (único ficheiro, 329 linhas, ~21 KB).

## O que o documento contém

Orientado a quem quer **avaliar, replicar ou criticar** o sistema — não a quem quer usá-lo:

| Secção | Conteúdo |
|---|---|
| 1. Ficha técnica | versão/MSRV/edição, 3 crates, dependências de produção e **as proibidas** (`tokio`/`reqwest`/`hyper`/`axum` via `deny.toml`), tamanho (396 ficheiros `src`, ~56,9k linhas Rust + ~13,3k linhas de wiki), 200 ADRs |
| 2. Modelo de dados e contrato de bytes | 31 chaves TOON, `id = hash(type + U+001F + normalize(statement))`, `body_hash`, enums fechados, extensões D207 (`claims`/`provenance`/ontologia), e a separação **NFC (identidade) vs NFD+fold (busca)** |
| 3. Arquitetura | portas/adaptadores + os **8 fakes**, as três camadas de persistência (verdade/auditoria/derivado), atomicidade, ordem de commit, locks, rebuild *double-buffer* |
| 4. Núcleo quantitativo | fórmulas e constantes: BM25 por campo (`k1=1.5`, `b=0.75`, pesos 3/2/1), corte `df/N ≥ 0.9` só com `N ≥ 64`, RRF (`k=60`, pesos 1/2/30/0), dedup por limiares e **MinHash+LSH**, PPR/Louvain/TMS, FSRS-like + **Beta-Bernoulli com limite de Wilson**, JS-divergence para drift |
| 5. Interface | `stdout=dados`/`stderr=logs`, `prime` (byte-idêntico) vs `rewind` (orçamento de tokens), 4 gatilhos MCP com *hints* como ponteiros, tabela de exit codes |
| 6. Avaliação | 924 `#[test]` (**contados por mim**; o README diz 883), 20 proptests, 8 goldens, 2 alvos de fuzz, Miri, supply chain; desempenho (gargalos O(N²) → −94 %); qualidade Recall@k/MRR/nDCG@k |
| 7. Corpus de decisões | os 200 ADRs como objeto de estudo de processo |
| 8. Limites e questões abertas | validade externa, poder estatístico, e as fragilidades que **medi** |
| 9–10. Reprodução e glossário | comandos `make` e onde ler as fontes |

## Duas coisas que a investigação acrescentou (e que evitam erros de leitura)

1. **`R@1 = 12,5 %` com `MRR = nDCG@1 = 100 %` não é contradição** — parece um erro de métrica, mas verifiquei em `bench/src/quality.rs`: `PER_TOPIC = 8` relevantes por tópico, logo `recall@1 = 1/8`. A aritmética geral confere (30 % = 18/60; `working-set` com 1 relevante dá 100 %).
2. **O sweep da fusão é plano**: 8 × 4 × 4 configurações de `rrf_k`/`anchor_w`/`semantic_w` produzem linhas idênticas → a tese "`k=60` é inerte" fica suportada, mas o corpus **não tem poder discriminativo** para justificar pesos. Acrescentei ainda que os dois baselines de R@1 (34,4 % ≈ 11/32 vs 30,0 % = 18/60) usam denominadores diferentes e portanto não são comparáveis entre ficheiros.

## Notas de honestidade no próprio texto

Marquei as afirmações como **[doc]**, **[código: ficheiro:linha]** ou **[medido]**, e o documento diz explicitamente que **não** corri `make check`/`bench` (o `REF/` é read-only e o build escreveria `target/`): os números de desempenho e qualidade são os artefactos versionados no repo, não medições minhas. Também registei a diferença 0.6.1 (binário) vs 0.5.4 (snapshot) onde ela importa.
