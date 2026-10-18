# H11 — Governança (contínuo)

> **Issue:** `task_01ehkt9h` · **Âncora:** [adr-governanca](../backlog/adr-governanca.md) ·
> **Desenho:** [revisao-proposito](../revisao-proposito.md) §6 · **Pré-requisitos:** nenhum
> (transversal; alimenta as outras issues).

## Objetivo
Manter o projecto **honesto e sustentável** — não é uma feature do agente, é **processo**. Evita o
erro simétrico do katu: construir **a mais** depois de funcionar.

## Passos
1. **ADRs com *Alternatives considered*** (`59`) — cada decisão regista o que foi rejeitado e porquê
   (com número, quando houver) + postmortems.
2. **Dossiê de pendências** (`60`) — o que ficou por fazer, com âncora e critério de reabertura.
3. **Método de medição** (`61`) — medir → A/B → adoptar-ou-reverter; + **teto de superfície** (`62`).

## Tarefas kd
| Tarefa | Entrega | Estado |
|---|---|---|
| `task_01ng3be9` | ADRs + postmortems | fechada |
| `task_01o2ilbx` | Dossiê de pendências | fechada |
| `task_01j0lf38` | Método de medição + teto de superfície | fechada |

## Gate / testes
- Toda decisão em `wiki/adr/` tem **Alternatives considered**.
- O dossiê cita **o teste/sintoma** que reabre cada pendência (regra de poda).
- O teto de superfície é conferido em revisão (nenhuma feature sem consumidor).

## Riscos
- Governança confundida com feature → **tier próprio**; não compete com o loop.
- Backlog a voltar a crescer → poda + teto de superfície (M1).

## Estado da implementação

A H11 é **processo** — não acrescenta caminhos ao agente, acrescenta **travões** e memória de decisão.
Duas peças são mecânicas (no `make check`) e duas são documentos com contrato:

| Entrega | Onde |
|---|---|
| ADRs imutáveis com `Alternatives considered` | `wiki/adr/` (17 ADRs: 0001–0017, uma delas **substituída** por outra) |
| Postmortems de incidentes reais | `wiki/postmortems/` (PM-0001–PM-0003) |
| Gate dos registos (secções, ids, simetria, índice) | `xtask check-adr` → `make adr` (no `make check`) |
| Dossiê de pendências verificado no código | `wiki/pendencias.md` (estados ⛔🚫⏳🔴⚪ × eixos Q/P/S + critério de reabertura) |
| Método de medição + base de evidência | `wiki/metodo-medicao.md`, `basis` no artefacto de `make bench` (schema 2), regra em `plan.md` §4.1 |
| Teto de superfície | `surface.toml` + `ktu self surface` + `xtask check-surface` → `make surface` (no `make check`) |

**Decisões de desenho.** (1) A medida da superfície é feita **pelo próprio binário**
(`ktu self surface`: árvore do `clap` + catálogo builtin), não por regex no `cli.rs` — mede o que o
utilizador vê, não a forma do código. (2) O `check-surface` corre o binário com `cargo run -q`
(nunca lê um binário possivelmente *stale* do `target/`). (3) O `check-adr` valida ADRs **e**
postmortems com o mesmo código: são o mesmo tipo de registo (decisão/incidente) com secções
obrigatórias e índice sem deriva. (4) A substituição é verificada **nos dois sentidos** — uma ADR
substituída tem de apontar para quem a substitui, e vice-versa. (5) O dossiê é o espelho do
`STAGING.md`: aquele diz o que falta, este o que está feito.

**Desvios ao plano.** (1) O plano previa o teto "conferido em revisão"; ficou **mecânico**
(`check-surface` no gate), porque o número só vale se falhar sozinho. (2) O plano falava de
`xtask check-docs`; o alvo chama-se `check-adr` (nome do backlog) e cobre os dois directórios.
(3) A medida inclui as tools do **loop** (`subagent`, `tool_search`), que não vivem no `Toolbox` —
está documentado em `crates/ktu-cli/src/surface.rs` (uma tool nova nesse nível tem de ser contada
ali).

## Prova (G10/G11)

```
$ cargo run -q -p xtask -- check-adr       # 17 ADRs + 3 postmortems: secções, ids, simetria, índice
$ cargo run -q -p xtask -- check-surface   # 47 subcomandos, 87 flags, 13 tools ≤ teto
$ ktu self surface                         # a mesma medida, pelo binário (--json = contrato)
$ make check                               # fmt + clippy + test + layers + adr + surface + file-length
```

| Critério do gate H11 | Prova |
|---|---|
| Toda a ADR tem `Alternatives considered` | `check-adr` (teste negativo: remover a secção falha) |
| A substituição é coerente (e simétrica) | `0016` ↔ `0017`; teste negativo: quebrar uma ponta falha |
| O índice não tem deriva | `check-adr` falha se um id não constar do `README.md` |
| O dossiê cita o teste/sintoma que reabre | §1 com nota de verificação + §2/§3 com critério de reabertura por linha |
| O teto de superfície é conferido | `check-surface` (teste negativo: acrescentar uma flag sem subir o teto falha) |
| Nenhum número sem base de evidência | `basis` por amostra no `ktu-bench.json` (schema 2) + §2 do método |
| Gate | `make check` verde (386 testes) |

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **`16` #7 (ADRs):** alinhado com a skill — registar decisões com **Alternatives considered**.
- **Determinismo:** o dossiê/índice em `BTreeMap` (ordem estável).
- **M1:** o **teto de superfície** é conferido em revisão; nenhuma feature sem consumidor.
- **Sem código de produção** nesta issue (é processo) — não afeta o gate de qualidade.
