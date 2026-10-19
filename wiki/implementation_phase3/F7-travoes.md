# F7 — Travões em falta (links/doc e knudge na CI)

> **Estado:** ✅ **fechado** (gate [G22](../gates/G22-travoes.md) verde 6/6, 2026-10-18).
> **Issue:** `task_01nopvll` · **Âncora:** [checks-review](../backlog/checks-review.md),
> [knudge-dependencia](../backlog/knudge-dependencia.md) · **Pré-requisitos:** H11 ·
> **Gatilho:** **já existe** — foi assim que dois links quebraram em silêncio (PM-0003) e que o
> `crates/knudge` ficou fora do `make check`.

## Objetivo

Fechar as duas lacunas de **travão** (não de funcionalidade) do dossiê:

- **L3** — nenhum gate verifica links internos da wiki nem intra-doc do Rust (`cargo doc` fora do
  `make check`);
- **L1** — o workspace do `knudge` está **excluído** do workspace raiz e não é verificado pela CI.

## Passos

1. **`xtask check-links`:** resolve links relativos `](...)` em `wiki/**/*.md` + `README.md` +
   `STAGING.md` + `plan.md`; ignora `http(s)`/`mailto`; falha com o par (ficheiro, alvo).
2. **`cargo doc` como travão:** `RUSTDOCFLAGS="-D warnings"` (com `broken_intra_doc_links` já em
   `deny`); entra no `make check` **só se** medir abaixo do limiar declarado (regra do
   [método de medição](../metodo-medicao.md)); senão fica num alvo próprio (`make doc`) + CI.
3. **knudge na CI:** alvo `make check-knudge` (`cd crates/knudge && make check`) chamado pela CI;
   o `make check` da raiz mantém-se (o knudge é workspace próprio — decisão registada).

## Tarefas kd

| Tarefa | Entrega |
|---|---|
| `task_016i3zv8` | `check-links` (wiki + intra-doc) no gate |
| `task_00susx00` | `check-knudge` na CI (workspace próprio) |

## Resultado

| Passo | Entrega | Onde |
|---|---|---|
| 1 | `xtask check-links` (0 quebrados, 0 avisos; `--root` para o gate) | `xtask/src/links.rs`, `make links` |
| 2 | `cargo doc` **dentro** do `make check` (medido ~4 s quente / ~34 s com alvo vazio; limiar 60 s) | `make doc` |
| 3 | `make check-knudge` + job `knudge` na CI | `Makefile`, `.github/workflows/ci.yml` |
| — | gate | `xtask/src/gates/brakes.rs` (G22, 6/6) |

O travão **pagou-se no primeiro dia**: `cargo doc` com `-D warnings` apanhou um alvo de link
explícito redundante em `crates/ktu-cli/src/instructions.rs` (`redundant_explicit_links`), que
estava fora do alcance de todos os travões desde o início do projecto.

## Gate / testes

- **G22** — ver [GATES](./GATES.md).
- Um link deliberadamente quebrado ⇒ `check-links` falha com o par (ficheiro, alvo).
- `check-links` sobre a árvore actual ⇒ verde — os **3** links pré-existentes do
  `wiki/quality_test/p4/` (que apontam a ficheiros de investigação não versionados) são
  **corrigidos ou excluídos com razão** no mesmo passo.
- A CI corre o gate do knudge (evidência: o passo no workflow).

## Riscos

- `cargo doc` lento no caminho quente → **medir primeiro**; alvo próprio se exceder o limiar.
- Falsos positivos em links (âncoras `#…`) → validar o **ficheiro**; âncoras só se existirem no
  ficheiro alvo (senão aviso, não erro).
