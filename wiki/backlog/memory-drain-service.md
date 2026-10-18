# Memória: embeddings locais opcionais (worker `drain service`)

- **Estado:** **implementado** (H8) — porta `Embedder` + `ktu memory drain` + worker `drain service`;
  contrato em [memoria-semantica](../memoria-semantica.md). Desvio: o llama.cpp não é descarregado
  (pré-requisito do provider).
- **Origem:** [CLI e configuração](../cli-e-config.md) §5; tática: `kd drain service --install` (D170/D182/D184/D185/D186).
- **Âncora:** `crates/ktu-core/src/ports/embedder.rs`, `crates/ktu-memory/src/{embed,drain}/`,
  `crates/ktu-cli/src/{embeddings,memory_drain,script}.rs`, `scripts/ktu-idle.sh`.

## O que é
O canal **semântico** da memória como parte **opcional e incentivada** do ktu: um worker local que
mantém um **servidor de embeddings** e drena a fila ociosa.

```
ktu memory drain [--status | --digest | --force]
ktu memory drain service [--install | --subscribe | --unsubscribe | --status | --uninstall | --reconcile | --dry-run]
    [--every <DUR>] [--port <N>] [--model <PATH>] [--no-deps] [--keep-model | --remove-model]
```

- `--install`: agendador (`systemd --user`/`launchd`) + servidor de embeddings + cadastro do
  projecto; baixa `llama.cpp` + o **mesmo modelo do knudge** (`google/embeddinggemma-2`,
  `embeddinggemma-2-Q8_0.gguf`) se ausentes; porta **8889**, período `--every 1h`.
- Ciclo de vida: `--status` (default), `--subscribe`/`--unsubscribe`, `--reconcile`,
  `--uninstall` (`--keep-model` default; `--remove-model` → **trash**, nunca `rm`), `--dry-run`.
- Fila **derivada**: `--digest` drena em lotes; `--force` apaga `.idx/` e reindexa.

## Porque interessa
O ganho de recall com embeddings é **expressivo** (medido no knudge). O ktu é hoje **lexical**;
tornar o semântico **fácil de ligar** (um comando) e **incentivado** melhora a memória — o
diferencial do ktu — sem o tornar obrigatório (M1: mínimo, mas útil).

## O que o ktu tem hoje
Recall lexical (BM25 + âncoras); **sem** embeddings. A [revisão do knudge](../revisao-knudge.md)
e o [G11/121](./knudge-embeddings.md) diziam "o worker fica de fora" — **esta página reverte** essa
parte, a pedido do dono.

## Pré-condições para reconsiderar
- Nenhuma (é opcional por desenho).

## Custo e risco
- **Esforço:** médio-alto (instalar agendador + servidor; script por SO).
- **Risco de não fazer:** recall lexical limitado a paráfrases.
- **Riscos a mitigar:** dependência de `llama.cpp`/GGUF (opt-in, `--no-deps`); Windows (worker
  embutido é Unix — documentar); **uma só verdade** (`memory.toml` projeta no knudge); degradação
  graciosa (sem worker, o canal semântico desliga sem aviso).

## Decisão pendente
Incorporar como **opcional e incentivado**: o `prime`/`doctor` **sugerem** `ktu memory drain service
--install`; sem ele, tudo funciona (lexical). Usar o **mesmo modelo** do knudge e a **mesma tática**
(background, subscribe, reconcile, trash no uninstall).
