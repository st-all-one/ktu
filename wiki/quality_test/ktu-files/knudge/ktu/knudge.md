# knudge — Síntese para Pesquisadores

> Repositório: `REF/knudge` (espelho local). CLI Rust **`kd`** + servidor **`knudge-mcp`**.
> Versão analisada: **0.5.4** · Licença: **MIT OR Apache-2.0** · MSRV: Rust **1.97+**.
> Idioma da documentação/decisões/código: **português**; identificadores: inglês.

## 1. Problema de Investigação

Como dar **memória durável e auditável a agentes de IA por projeto** sem infraestrutura de
servidor? O knudge responde com três teses:

1. **A nota é a verdade; o índice é derivado e descartável.** Conhecimento vive em Markdown com
   frontmatter canônico (TOON). O índice (BM25, grafo, embeddings) vive em `.idx/` e pode ser
   apagado/reconstruído sem perda de conhecimento (D95, D15).
2. **Buscar antes de gravar** como disciplina central. Todo `write` faz dedup probabilístico
   (`score < 0.75` cria; `0.75–0.92` merge; `≥ 0.92` rejeita), evitando duplicatas e consolidando
   conhecimento quase-idêntico.
3. **Sem servidor, sem banco, sem daemon.** Dois binários estáticos, memória local versionada por
   git, superfície de integração via **MCP (JSON-RPC 2.0 sobre stdio)**.

## 2. Arquitetura

Três crates, separação rígida entre **domínio puro** e **adaptadores**:

```
knudge-mcp ──┐
             ├── knudge-core  (lógica pura + ports; nunca toca FS/relógio/argv diretamente)
knudge-cli ──┘
```

| Crate | Papel |
|---|---|
| `knudge-core` | Modelo, schema, TOON (contrato de bytes), retrieval, lifecycle, **portas** (`Clock`, `Rng`, `Env`, `Fs`, `Git`, `HookRunner`, `Logger`). Adaptadores `std` isolados em `adapters/`. |
| `knudge-cli` | Binário `kd`: clap, envelope `--json`, montagem de adaptadores, saída em pipe. |
| `knudge-mcp` | Servidor MCP: gatilhos (`knudge_pre_write`, `knudge_pre_edit`, `knudge_session_end`, `knudge_status`) + transporte JSON-RPC/stdio. |

### Decisões de engenharia notáveis (ver `plan/03_decisoes-fechadas.md`, D01–D101)

- **Portas de injeção de dependência** (`ports/`): o core não chama `SystemTime::now`,
  `env::var`, FS ou git diretamente — recebe fakes nos testes (`FixedClock`, `SeqRng`, `MemFs`,
  `FakeGit`, `RecordingLogger`). Isso torna 883 testes + proptest **determinísticos**.
- **Determinismo como requisito**: `BTreeMap`/`IndexMap` em vez de `HashMap`; iteração estável;
  desempate `(score desc, id asc)`; `overflow-checks` ligado.
- **Contrato de bytes (TOON)**: 31 chaves em ordem canônica; opcionais omitidos (nunca `null`);
  `normalize` = NFC + trim + colapso; `id = <tipo>_<base36(8)>` derivado de
  `hash(type + U+001F + normalize(statement))`. Reclassificar tipo **não** reescreve o id.
- **Degradação graciosa (R33)**: falha de recurso opcional retorna resultado parcial + `warnings[]`
  (não interrompe), exceto com `strict` no config.
- **Limites duros de código**: sem `unwrap`/`expect`/`panic`/`unsafe` em produção; arquivos ≤
  300 linhas; macros `dbg!`/`todo!`/`unreachable!` proibidas; `#[allow(...)]` exige `reason`.
- **Erros como valores**: `Error` enum fechado (`thiserror`, `#[non_exhaustive]`); `ErrorKind::code()`
  é contrato de máquina; mapeamento de exit codes (`not_found=3`, `conflict=4`, `io=5`, …; 101
  reservado a panic; EPIPE → 0).

## 3. Ciclo de Operação

```
kd ask → kd write → kd task → kd sync
(buscar)  (gravar)   (executar) (commit)
```

1. **`kd init`** — funda `.knudge/`; escreve bloco gerenciado em `AGENTS.md`; instala skill do
   agente em `.agents/skill/kd/SKILL.md`.
2. **`kd ask "<query>"`** — busca híbrida: filtros → BM25 → âncoras → RRF, com canais vetorial
   (embeddings) e de autoridade (PPR) quando disponíveis. Modos: `--brief` (`id|statement`),
   revelação progressiva (padrão, D161), `--full-content`, `--rank`, `--tags`, `--suggest`,
   `--around` (expansão de grafo).
3. **`kd write --summary ... --type ... --tag ... --anchor ...`** — cria/atualiza notas com dedup;
   `--dry-run`, `--batch` (JSONL), `--update`, `--outcome` (anexa resultado a tarefa), `--claim`
   (SPO atômico), proveniência (agent/activity).
4. **`kd task new/list/close/show/graph`** — hierarquia fechada `epic ⊃ {issue ⊃ task | task}`;
   fechamento exige evidência (`--outcome success --note`).
5. **`kd rewind`** — estado/handoff ponto-no-tempo (manifest, orçamento, `context_id`).
6. **`kd doctor`** — 13 checks + auditoria; reparo reversível (`--fix`), explicação (`--explain`).
7. **`kd forget`** — soft-delete → restore → purge (com retenção D84).
8. **`kd sync`** — commit de `notas/` + `eventos/` no git.
9. **`kd prime`** — protocolo estático ("help da IA"), byte-idêntico por versão.
10. **`kd drain`** — fila de embeddings + worker (auto-drain ocioso, lazy, E11-T03).

## 4. Ontologia e Grafo

- **Tipos fechados**: 10 armazenáveis (`epic` derivado de `scope=epic`, D149).
- **Arestas leves**: `same_as`, `broader`, `narrower`, `related` — extração, ciclos, comunidades,
  TMS (truth maintenance) e sugestões de aresta/contradição (D158).
- **Âncoras (`--anchor PATH`)** ligam notas ao código; aceitam glob (`src/**`), são repetíveis;
  `kd ask --anchor` busca sem query textual; `kd doctor` lista âncoras quebradas.
- **Ciclo de vida** (`lifecycle/`): shelf-life, decay, purga, confiança derivada, clusters.

## 5. Busca e Recuperação (retrieval/)

- **BM25** + filtros estruturais (tags, tipos, classes, status, escopo, âncoras, intervalo
  temporal, `--as-of` histórico).
- **RRF** para fusão de canais: lexical, anchor, semantic, ppr, recent, stars, body.
- **Embeddings opcionais** (D170/D186): provedor HTTP (`llama.cpp` + GGUF, ex.
  `granite-embedding-97m-multilingual-r2`, 384 dims); cache persistente; fila com auto-drain
  ocioso (`mode=lazy`); sem embeddings o `ask` degrada para BM25 com `warnings`.
- **Provedor de embeddings isolado** em adaptador (`adapters/`), com exceção única para `unsafe`.

## 6. Interface Humana vs. Máquina

- **`stdout` = dados** (pipe/`--json`); **`stderr` = logs** (nunca `println!`). Redação de
  segredos no layer (`Redactor` + `TracingLogger`).
- **`--json`** emite envelope com `warnings[]`, métricas estruturadas, `body_snippet` etc.
- **Pipe formatado** para agentes (colunas `|`), com modos `--brief`/progressivo/`--full-content`.
- **Limitação identificada**: a saída padrão é desenhada para agentes; não há modo `--human` nem
  TUI exploratória — oportunidade de usabilidade humana (ver análises de exploração).

## 7. Artefatos de Referência no Repositório

| Caminho | Conteúdo |
|---|---|
| `README.md` | Visão geral, quick-start, mapa |
| `AGENTS.md` | Guia de contribuição + regras de ouro |
| `SKILL.md` | Guia ativo para agentes dos projetos que adotam knudge |
| `wiki/usage/` (00–19) | Guia por comando + quickstart/filosofia/troubleshooting |
| `wiki/specs/` | ARCHITECTURE, TOON, DIVERGENCES, ADRs |
| `plan/` | Visão, decisões Dxx, implementação por épico, políticas Rn |
| `crates/*/MODULE.md` | Papel de cada módulo (manter em sincronia) |
| `llms.txt` | Índice canônico para modelos de linguagem |
| `fuzz/`, `bench/`, `scripts/` | Fuzzing (TOON, JSONL), benchmarks, utilitários |

## 8. Oportunidades de Investigação / Extensão

1. **Usabilidade humana**: flag `--human` / TUI exploratória / `kd stats` — saída atual é
   orientada a máquina (ver análises de exploração).
2. **Modo interativo** para auditoria e exploração do grafo de conhecimento.
3. **Embeddings**: modelos multilíngues PT, benchmark de recall híbrido.
4. **MCP**: superfície de gatilhos e integração com clientes (claude/cursor/codex/pi).
5. **TMS e comunidades**: manutenção de verdade e detecção de clusters semânticos.
6. **Contrato de bytes**: evolução do TOON (migração de corpus, versão de schema).

## 9. Citação Rápida

knudge é uma CLI Rust que dá memória durável a agentes de IA por projeto — notas em Markdown como
fonte da verdade, índice derivado reconstruível e busca híbrida (BM25 + âncoras + embeddings
opcional via RRF) — sem servidor, sem banco e sem daemon, com ciclo `ask → write → task → sync`.
