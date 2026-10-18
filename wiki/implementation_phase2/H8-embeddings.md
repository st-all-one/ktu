# H8 — Embeddings locais opcionais

> **Issue:** `task_01t8rm0l` · **Âncora:** [memory-drain-service](../backlog/memory-drain-service.md) ·
> **Desenho:** [cli-e-config](../cli-e-config.md) §5 · **Pré-requisitos:** H2, H7.
>
> **Estado: concluída.** Porta `Embedder` (síncrona) + `FakeEmbedder`/`UnavailableEmbedder`; índice
> derivado `.agents/memory/.idx/embeddings.jsonl` (codec do knudge); `ktu memory drain
> --status|--digest|--force`; worker `drain service` (script embutido, systemd/launchd) e
> `memory.toml` com as chaves do knudge. Contrato em [memoria-semantica](../memoria-semantica.md).
> **Desvio do plano:** o llama.cpp **não** é descarregado (pré-requisito do provider; o
> *installer* oficial não é verificável) — o worker localiza-o e imprime o guia se faltar.

## Objetivo
O canal **semântico** da memória como parte **opcional e incentivada**: porta `Embedder`, fila
derivada (`drain`) e worker em background (`drain service --install`), com o **mesmo modelo** do
knudge. Sem worker, o recall é lexical e tudo funciona.

## Passos
1. **Porta `Embedder`** (`meta` + `embed`) + `FakeEmbedder` determinístico; canal semântico entra na
   **fusão RRF**; sem provedor, desliga **sem aviso**.
2. **`ktu memory drain [--status|--digest|--force]`** — fila **derivada**; `--force` apaga `.idx/` e
   reindexa (último recurso).
3. **`ktu memory drain service [--install|--subscribe|--unsubscribe|--status|--uninstall|--reconcile|--dry-run]`**
   — agendador (`systemd --user`/`launchd`) + servidor de embeddings + cadastro do projecto.
4. **Modelo `google/embeddinggemma-2`** (`embeddinggemma-2-Q8_0.gguf`), porta **8889**,
   `--every 1h`; `--remove-model` → **trash** (nunca `rm`).

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_0027aiti` | Porta `Embedder` + `FakeEmbedder` + RRF |
| `task_00z3gemm` | `ktu memory drain` (status/digest/force) |
| `task_018q1vht` | `drain service` (install/subscribe/… ) |
| `task_00t9pviy` | Modelo + degradação graciosa + trash |

## Gate / testes
- Sem worker: recall lexical funciona; canal semântico **desligado**.
- Com `FakeEmbedder`: recall idêntico em duas execuções (determinismo).
- `--remove-model` move para o **trash** (recuperável).
- `memory.toml` projeta no knudge (uma só verdade).

## Riscos
- Dependência de `llama.cpp`/GGUF → **opt-in** (`--no-deps`); documentar Windows.
- Dupla verdade da config → `memory.toml` é **vista** do ktu sobre o knudge.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **Porta + fake:** `Embedder` é porta; `FakeEmbedder` **determinístico** nos testes (nunca o
  servidor real no gate).
- **§2:** o modelo real produz floats **não-reprodutíveis**; por isso o canal semântico é
  **opcional** e **fora** do caminho determinístico (o recall lexical é o determinístico).
- **N2:** instalação/arranque do worker sem `unwrap`; falha = degradação graciosa.
- **Gate:** complementa **G15** (não bloqueia; o gate usa o fake).

## Prova (canal semântico opcional)

Sem `memory.toml` (o *default*) o canal está desligado e o recall é lexical:

```
$ ktu memory drain --status
memory.embeddings.provider: none
memory.embeddings.model: google/embeddinggemma-2
memory.embeddings.dimensions: 768
memory.embeddings.channel: lexical
memory.embeddings.index: absent
memory.embeddings.indexed: 0
memory.embeddings.pending: 0
memory.embeddings.stale: 0
memory.embeddings.next: canal semantico desligado; `ktu memory drain service --install` liga-o
```

| Critério | Prova |
|---|---|
| Sem worker: recall lexical funciona; canal semântico desligado | `ktu-cli/tests/embeddings.rs::provider_none_is_lexical_and_digest_is_a_noop`; `--digest` com `provider = none` não escreve nada em `.idx/` |
| Com `FakeEmbedder`: recall **idêntico** em duas execuções | `ktu-memory/tests/memory.rs::semantic_channel_recovers_a_paraphrase_that_lexical_misses` (asserta `warm == again`) |
| O canal vetorial entra na **fusão RRF** | idem: a paráfrase («como se detectam duplicados») só é recuperada **depois** do dreno — lexical não a encontra |
| `--digest` indexa a fila e é idempotente | `ktu-cli/tests/embeddings.rs::digest_indexes_the_queue_and_status_reports_it` (`indexed: 1` ⇒ `0` ⇒ `--force` ⇒ `1`) |
| Falha do provedor **não descarta** nota | `digest_without_a_server_fails_but_keeps_the_queue` (`exit 1`, `pending: 1`, índice ausente) |
| `--remove-model` move para o **trash** | `uninstall_moves_the_model_to_the_trash` (o ficheiro sai do lugar e aparece 1 entrada no trash) |
| `memory.toml` projeta no knudge (uma só verdade) | mesmas chaves `embeddings.*`; *schema* fechado ⇒ `invalid_memory_toml_fails_high` |
