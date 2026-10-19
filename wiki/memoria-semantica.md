# Memória semântica opcional (embeddings locais)

> Implementa a **H8** ([`implementation_phase2/H8-embeddings.md`](implementation_phase2/H8-embeddings.md))
> a partir de [`cli-e-config.md`](cli-e-config.md) §5 e
> [`backlog/memory-drain-service.md`](backlog/memory-drain-service.md). O motor continua a ser o
> `knudge-core`; o que muda é **haver um segundo canal** (vetorial) na fusão do recall.

---

## 1. Princípio

O canal semântico é **opcional e derivado**:

- **opcional** — sem provedor, o recall é lexical (BM25 + âncoras) e **tudo funciona**; o canal
  desliga **sem aviso** (nenhum erro, nenhum aviso, nenhuma mudança de contrato);
- **derivado** — o índice vetorial é reconstruível de `notes.jsonl` + provedor e descartável;
  apagá-lo não perde conhecimento;
- **off-path** — nada no turno depende dele: a fila é drenada por um comando explícito
  (`ktu memory drain --digest`) ou pelo *worker*.

Uma nota recém-escrita fica **escura** (sem vetor) até ser digerida — exactamente como no knudge
(D80): o agente nunca espera por embeddings para escrever.

## 2. A porta `Embedder`

`ktu_core::ports::Embedder` — **síncrona**, porque o recall do kernel
([`ktu_core::memory::Memory`]) é síncrono:

```rust
fn meta(&self) -> &EmbeddingMeta;                                  // identidade (invalida o índice)
fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError>;  // ordem preservada
```

| Implementação | Onde | Uso |
|---|---|---|
| `HttpEmbedder` | `ktu-provider::embed` | servidor local OpenAI-compatible (`llama-server`, porta 8889) |
| `FakeEmbedder` | `ktu-core::ports::fakes` | **determinístico** (hash FNV-1a), testes/CI/offline |
| `UnavailableEmbedder` | idem | prova a degradação graciosa (falha sempre) |

O adaptador HTTP é **bloqueante** (`reqwest::blocking`): o dreno corre fora do *runtime* e, no
loop, a chamada bloqueia o *worker* durante a consulta. O servidor é local e o canal é *opt-in*,
por isso o custo é aceite — e nunca há *deadlock* (o cliente tem o seu próprio *runtime*).

## 3. O índice derivado

`.agents/memory/.idx/embeddings.jsonl` — cabeçalho de identidade + uma linha por nota:

```json
{"meta": {"provider": "http", "model": "google/embeddinggemma-2", "revision": "main",
          "dimensions": 768, "similarity": "cosine", "prefix": "embeddinggemma"}}
{"id": "fact_…", "body_hash": "…", "vector": [0.013, -0.041, …]}
```

O **codec é o do knudge** (`knudge_core::embeddings::EmbeddingIndex`) — as duas vistas não
divergem e o índice é portável entre elas.

- **Invalidação**: se a identidade do cabeçalho não casar com a configuração (modelo, revisão,
  dimensão ou prefixo), o índice é **descartado** e tudo é re-embeddado.
- **Auto-reparação**: um índice ilegível é tratado como **ausente** (é 100 % derivado; o dreno
  seguinte reescreve-o).
- **Durabilidade**: a escrita é `temp → fsync → rename` (nunca um índice meio-escrito).
- A métrica é **cosseno** (a dos modelos `-cos`, como o `embeddinggemma`); o prefixo de instrução
  (`auto` → `embeddinggemma`) é resolvido pelo nome do modelo.

## 4. A fila e o dreno

A fila **não é armazenada** — é a diferença entre as notas e o índice, classificada pelo
`body_hash`:

| Estado | Significado |
|---|---|
| `indexed` | vetor presente e actual |
| `pending` | sem vetor (nunca digerida, ou o provedor falhou) |
| `stale` | vetor desactualizado (o corpo mudou desde o último dreno) |

```
ktu memory drain [--status | --digest | --force]
```

| Modo | Efeito |
|---|---|
| `--status` (*default*) | estado rico (provider/modelo/dimensão/canal/índice e as três contagens), **sem tocar no índice** |
| `--digest` | embeda em lotes de `embeddings.batch` até esvaziar; grava **uma vez por lote** |
| `--force` | ignora o índice existente e re-embeda **todas** as notas (último recurso) |

Garantias: uma falha do provedor **não descarta nota** — fica `pending` e o comando sai com
`exit 1` (dizendo quantas ficaram); `--digest` é **idempotente** (nada mudou ⇒ `indexed: 0`).

## 5. O canal na fusão RRF

Quando há provedor **e** índice reaproveitável, a consulta é embedada, os `top_k` (20) ids mais
próximos entram em `RecallQuery.vector` e o motor funde-os com o lexical e as âncoras via **RRF**
(`knudge_core::retrieval::recall`). Sem provedor, sem índice ou com o provedor fora, `vector`
fica `None` e a cascata degrada para lexical.

Prova (teste de integração): uma consulta que **não partilha nenhum termo** com a nota é
recuperada depois do dreno, e **não** é antes — o canal vetorial é o único que a pode trazer.

## 6. `memory.toml` — uma só verdade

`.agents/memory.toml` (project-bound) usa as **mesmas chaves** do knudge, para que a vista do ktu
e a do knudge nunca divirjam:

```toml
[embeddings]
provider   = "http"                                   # none (default) | http
endpoint   = "http://127.0.0.1:8889/v1/embeddings"
model      = "google/embeddinggemma-2"
revision   = "main"
dimensions = 768
prefix     = "auto"                                   # auto | none | embeddinggemma
batch      = 32
```

O *schema* é **fechado** (`deny_unknown_fields`): uma chave desconhecida **falha alto** (`exit 1`)
em vez de ser ignorada em silêncio. Ausente ⇒ *defaults* (canal desligado).

## 7. O worker (`drain service`)

```
ktu memory drain service [--install | --subscribe | --unsubscribe | --status | --uninstall | --reconcile | --dry-run]
    [--every <DUR>] [--port <N>] [--model <PATH>] [--no-deps] [--keep-model | --remove-model]
```

O worker é um **script embutido no binário** (`scripts/ktu-idle.sh`, sem *supply-chain* de rede),
materializado em `<config>/ktu/ktu-idle.sh` no `--install` (o agendador precisa de um caminho: o
worker copia-se para `~/.local/bin/ktu-idle`).

| Acção | Efeito |
|---|---|
| `--install` | agendador (`systemd --user`/`launchd`) + servidor de embeddings persistente (`ktu-embed`) + cadastro do projecto + `memory.toml` alinhado |
| `--subscribe`/`--unsubscribe` | cadastra/descadastra o projecto (`<config>/ktu/idle.projects`, um caminho por linha) |
| `--status` (*default*) | agendador, servidor (`/health`) e a fila de cada projecto |
| `--reconcile` | re-alinha `memory.toml` ao worker instalado |
| `--uninstall` | remove unidades + worker; `--keep-model` (*default*) preserva o GGUF, `--remove-model` move-o para o **trash** (recuperável; nunca `rm`) |
| `--dry-run` | só mostra o plano (não descarrega nem escreve) |

O canal é **incentivado, não exigido**: o `ktu prime` e o `ktu memory doctor` dizem que está
lexical e apontam `ktu memory drain service --install`; nada falha por não o fazer.

Período `--every 1h`, porta `--port 8889`, modelo `google/embeddinggemma-2`
(`embeddinggemma-2-Q8_0.gguf`), revisão **pinada** e SHA-256 verificado (falha *fail-closed* se não
casar). O worker drena cada projecto com `ktu memory drain --digest` e sobe um servidor **efémero**
de recurso se o persistente estiver fora (`-ub 2048`: o *default* 512 do `llama.cpp` rejeita notas
longas).

### O que o worker **não** faz (decisão)

O **llama.cpp não é descarregado**: é pré-requisito do provider (o ktu fala com o `llama-server`),
e o *installer* oficial é um `curl … | sh` cujo *hash* não é verificável aqui. O worker localiza-o
no `PATH` ou em `$KTU_LLAMA` e, se faltar, imprime o guia. `--no-deps` salta a descarga **do
modelo** (falha se faltar). Desvio consciente do plano (que previa descarregar ambos) — documentado
aqui em vez de silenciado.

## 8. Segurança

- O GGUF vem de uma **revisão pinada** (nunca `/resolve/main/`) com SHA-256 verificado; um *mismatch*
  aborta sem instalar nada.
- O `trash` é sempre usado em vez de `rm` (modelo, unidades, plist, binário do worker).
- Nada corre código de terceiros: o worker é o script embutido.
- O `stderr` do worker é herdado (log em directo); `stdout` continua a ser **dados**.

## 9. Testes

| Comportamento | Teste |
|---|---|
| Fake determinístico e normalizado | `ktu-core::ports::fakes::embedder::tests` |
| Config fechada (chaves, provedor, defaults) | `ktu-core::memory_config::tests` |
| Índice: round-trip, invalidação por modelo, índice ilegível ⇒ ausente | `ktu-memory::embed::tests` |
| Fila: pending/stale/indexed, lotes, idempotência, `--force`, falha do provedor | `ktu-memory::drain::tests` |
| **Canal semântico na fusão RRF** (paráfrase só encontrada depois do dreno) + degradação | `ktu-memory/tests/memory.rs` |
| Adaptador HTTP: forma OpenAI e legada, dimensão errada, endpoint inválido | `ktu-provider::embed::tests` |
| CLI: `--status`/`--digest`/`--force` contra um servidor falso, `provider = none`, `memory.toml` inválido, `service --dry-run`, `--uninstall --remove-model` ⇒ trash | `ktu-cli/tests/embeddings.rs` |

O gate **nunca** fala com um `llama-server` real: o determinismo vive nos *fakes*.

## 10. Adiado (YAGNI, com razão)

| Peça | Razão |
|---|---|
| **Cache vetorial** por `body_hash` | o índice já evita re-embedar o que não mudou; o cache só ajuda quando uma nota é apagada e recriada |
| **`min_score` configurável** | o `0.0` (como o knudge) é suficiente; subir o corte sem medição seria adivinhação |
| **Modo `lazy`** (drenar no fim de cada invocação) | o worker cobre o caso; um gancho em cada comando é custo sem sintoma |
| **`similarity = dot`** | o modelo é `-cos`; acrescentar a chave só alargaria o schema |
