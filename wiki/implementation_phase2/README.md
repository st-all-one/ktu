# Fase 2 — caminhos (CLI/config), fiabilidade e eixo de memória

> **Épico:** `epic_01j7ou2p` · **knudge:** `kd ask --scope epic_01j7ou2p` · **âncora:** este
> ficheiro. Documentos de desenho: [`revisao-proposito`](../revisao-proposito.md),
> [`cli-e-config`](../cli-e-config.md), [`knudge-eixo-memoria`](../knudge-eixo-memoria.md),
> [`backlog_reviews`](../backlog_reviews.md).
>
> **Tese da Fase 2.** A Fase 1 provou que o ktu é **usável** (G4). A Fase 2 garante que é
> **mínimo, kernel, seguro por padrão, independente do caminho, determinístico no controlo e
> project-bound** — servindo **quatro utilizações**: (1) crate, (2) pipeline, (3) conversa longa
> (CLI), (4) configuração no projecto. O **CLI é o front primário**.

---

## 0. Gate da Fase 2

Cada issue fecha com o **portão único** verde: `make check`
(`fmt` + `clippy -D warnings` + `test` + `xtask check-layers` + `check_file_length`) e
`make supply-chain`. **Sem `unsafe`/`unwrap`/`expect`/`panic`** em `src/`; testes **também** sem
`unwrap`/`expect`/`panic`; `#[allow]` exige `reason`. **Sem commits** (gesto humano).

**Invariantes novos (travados por teste):**
- **M4 — dois caminhos, uma sessão:** mesmo prompt por `run`/REPL/tui/crate ⇒ **mesmo log** (I3/I6).
- **M3 — auth não ecoa:** `auth.toml` nunca no log nem no contexto.
- **M5 — guardrail global não removível:** o piso do utilizador é aditivo.

---

## 1. Sequência (estritamente ordenada)

```
H0 Superfície do CLI ─┐
H1 ktu init          ├─ cadeia CLI (front primário)
H2 Config global/local│
H3 Efêmero + lock    │
H4 REPL ktu session ─┘
H5 Fiabilidade do loop ─┐
H6 Controlo determinístico ─┘  (loop)
H7 Eixo de memória (knudge) ─┐
H8 Embeddings opcionais ─────┘  (memória)
H9 Crate facade
H10 Convenções
H11 Governança (contínuo)
H12 Gates da Fase 2 (G10–G15) ── saída da fase

── Fase 2 fechada ─────────────────────────────────────────────────────────

Fase 3 — Adiados (YAGNI)      ── ver [ADIADOS](./ADIADOS.md) (pós-H12)
```

| # | Issue | Âncora (backlog) | Tarefas | Estado |
|---|---|---|---|---|
| H0 | `task_00qn6s14` Superfície do CLI | [cli-superficie](../backlog/cli-superficie.md) | 5 | fechada |
| H1 | `task_01v41q8n` ktu init | [ktu-init](../backlog/ktu-init.md) | 4 | fechada |
| H2 | `task_00tx0uvu` Config global/local | [config-global](../backlog/config-global.md) | 5 | fechada |
| H3 | `task_017rrpwn` Efêmero + lock | [modo-efemero](../backlog/modo-efemero.md) | 3 | fechada |
| H4 | `task_010l8e4e` REPL | [cli-repl](../backlog/cli-repl.md) | 4 | fechada |
| H5 | `task_00htbspv` Fiabilidade do loop | [consolidado](../backlog/consolidado.md) | 5 | fechada |
| H6 | `task_01s9n99d` Controlo | [permission-store](../backlog/permission-store.md) | 3 | fechada |
| H7 | `task_00104ugh` Eixo de memória | [kd-ai](../backlog/kd-ai.md) | 5 | fechada |
| H8 | `task_01t8rm0l` Embeddings | [memory-drain-service](../backlog/memory-drain-service.md) | 4 | fechada |
| H9 | `task_00dkc6dk` Crate facade | [crate-facade](../backlog/crate-facade.md) | 3 | fechada |
| H10 | `task_01tp3uj0` Convenções | [consolidado](../backlog/consolidado.md) | 4 | fechada |
| H11 | `task_01ehkt9h` Governança | [adr-governanca](../backlog/adr-governanca.md) | 3 | fechada |
| H12 | `task_00h7sxsu` Gates da Fase 2 | [GATES](./GATES.md) | 6 | fechada |

**Total:** 13 issues · **54 tarefas** (`kd task list --scope task --tag fase2`) — **13/13 issues
fechadas** (H0–H12). Os gates da Fase 2 (G10–G15) estão verdes, com artefactos em
[`wiki/gates/`](../gates/README.md). A **Fase 3** ([ADIADOS](./ADIADOS.md)) só abre com sintoma.

> **Depois de a Fase 2 fechar:** os itens **adiados por YAGNI** (recursos globais, migrações,
> `recipes`/`setup`, `.ktu/logs`) estão na **Fase 3** — [ADIADOS](./ADIADOS.md) — e **não**
> entram antes de a Fase 2 concluir.

> **Leitura obrigatória antes de executar:** [NOTAS-RUST](./NOTAS-RUST.md) (skills Rust × princípios)
> e [GATES](./GATES.md) (G10–G15, usabilidade em cadeias extensas).

---

## 2. Método (não negociável)

1. **Simples primeiro.** O mais simples que fecha a issue; complexidade só com número.
2. **Através da porta.** Nenhum caminho novo tem lógica própria; todos falam à `ktu-port`.
3. **Um só standard.** Config num ficheiro; memória no schema do knudge; um parser por conceito.
4. **Teste antes de fechar.** Cada tarefa aponta o teste que a prova; `kd task close --outcome`.
5. **Âncora no backlog.** Cada issue referencia a página do backlog que a justifica.

---

## 3. Como executar

```bash
kd ask --scope epic_01j7ou2p            # contexto do épico
kd task list --parent task_00qn6s14     # tarefas de uma issue
kd task close --id <task> --outcome success --note '<evidência>'
kd doctor --fix                          # saúde do corpus
```

Cada documento `Hn-*.md` traz: **objetivo**, **âncora**, **pré-condições**, **passos**,
**tarefas kd**, **gate/testes** e **riscos**.
