# G22 — Travões (links, `cargo doc` e knudge na CI)

- **Veredicto:** ✅ **verde** (6/6) · **Artefactos:** `artifacts/G22/` · **Data:** 2026-10-18
- **Item:** [F7](../implementation_phase3/F7-travoes.md) ·
  **Critério:** [GATES da Fase 3](../implementation_phase3/GATES.md) §G22.

## Cenário

Fecha as duas lacunas de **travão** do dossiê: **L3** (nenhum gate olhava para a documentação —
links da wiki e links *intra-doc* do Rust) e **L1** (o `crates/knudge` é workspace próprio,
excluído da raiz, e não era verificado por nenhum travão).

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | a árvore actual **não** tem links quebrados | `0 quebrados, 0 avisos` (~770 links) |
| 2 | um link quebrado **falha** com o par (ficheiro, alvo) | `…/ensaio/wiki/index.md → nao-existe.md` |
| 3 | âncora inexistente é **aviso**, não erro | `ancora #seccao-que-nao-existe nao encontrada` (exit 0) |
| 4 | o **doc** não tem avisos | `RUSTDOCFLAGS="-D warnings"` exit 0, 0 avisos |
| 5 | a **CI** corre o portão do knudge | `run: make check-knudge` |
| 6 | a **suite** do knudge passa | 22 binários de teste, 0 com falhas |

## Artefactos

`links.txt` (a verificação real, verde) · `links-broken.txt` (a falha deliberada, com o par) ·
`links-anchor.txt` (o aviso da âncora) · `doc.txt` (o `cargo doc` sem avisos) ·
`ci-knudge.txt` (o passo do workflow) · `knudge-tests.txt` (os 22 resumos da suite).

## Notas

- **`check-links` (`xtask/src/links.rs`)** varre `wiki/**/*.md` + `README.md` + `STAGING.md` +
  `plan.md`, resolve cada destino relativo contra o directório do ficheiro, ignora
  `http(s)`/`mailto`/só-âncora e **ignora o que está em código** (blocos ```…``` e `código em
  linha`): ali um `](…)` é um exemplo, não um link. A âncora valida-se contra os títulos do
  destino (slug do GitHub) e uma âncora que não existe é **aviso em `stderr`**, nunca erro — uma
  mudança de título não deve quebrar o portão.
- **`--root`** existe para o gate: aponta a verificação a uma árvore de ensaio com o link
  deliberadamente quebrado (a verificação real corre na raiz). Sem ele, provar a falha exigiria
  quebrar a wiki verdadeira.
- **`cargo doc` entra no `make check`** (alvo `doc`, `RUSTDOCFLAGS="-D warnings"`) — medido
  **~4 s** com o alvo quente (o `test` do mesmo portão já construiu as rlibs) e **~34 s** com um
  alvo próprio vazio. Fica muito abaixo do limiar declarado (60 s), pelo que **não** precisa de
  um alvo separado. O travão já apanhou um defeito real: um alvo de link explícito redundante em
  `crates/ktu-cli/src/instructions.rs` (`redundant_explicit_links`) que ninguém via, porque
  `cargo doc` estava fora do portão desde sempre — **é a L3 a pagar-se no primeiro dia**.
- **O knudge** (L1) corre num alvo dedicado `make check-knudge` (`cd crates/knudge && make check`)
  e num **job próprio** da CI (`knudge`, com `rust-cache` sobre `crates/knudge`): o workspace é
  independente e o `make check` da raiz não o vê (`exclude` no `Cargo.toml` raiz). O gate prova
  as **duas** metades — o wiring (alvo + passo no workflow) e a suite (corre o mesmo comando:
  ~6 s com o alvo quente, ~45 s com alvo vazio).
- **Âmbito medido.** O travão cobre a documentação **do repositório ktu**: `wiki/**/*.md` +
  `README.md` + `STAGING.md` + `plan.md` (~770 links). Ficam de fora, com o número medido:
  `crates/knudge/wiki/**` (**677 links, 0 quebrados** — 1 aviso de âncora) e os `MODULE.md` do
  knudge (11 links); a documentação do knudge é o travão **do portão dele** (workspace próprio,
  `make check-knudge`) — a mesma razão que mantém o knudge fora do `make check` da raiz (L1) —, e
  `exploration/` (0 links). Sem âncora: `crates/ktu-*/MODULE.md` não existem.
- **O que continua fora:** links **externos** (`http(s)`) não são verificados (rede
  não-determinística — M4/determinismo); `cargo doc` cobre os links *intra-doc* do Rust, não os
  do Markdown, e é por isso que os dois travões existem lado a lado.
