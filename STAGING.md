# STAGING — Estado do épico ktu (revisão 2026-10-15)

> Documento de staging: fotografia do estado após o fecho do **Gate G4**.
> Fontes: `epic.md`, `plan.md`, `wiki/quality_test/ktu-veredicto.md` e a memória `kd`.
> Portão único: `make check` (`fmt` + `clippy -D warnings` + `test` + `check-layers`
> + `file-length`). Verificado **verde** nesta revisão.

## 1. Gate G4 — confirmado fechado ✅

**Issue** `task_010xrtzt` (GATE inegociável) → `closed`, com 2 outcomes `success` e
todos os filhos fechados.

| Task | Estado | Evidência |
|---|---|---|
| `task_013dfilu` harness (`scripts/quality.sh` + `xtask quality`) | closed | `--dry-run` reproduz o plano p1–p4 |
| `task_00vq487c` executar p1–p4 | closed | `wiki/quality_test/p*/ktu-*.md` + `ktu-summary.md` |
| `task_00kz804y` julgar/arquivar | closed | `wiki/quality_test/ktu-veredicto.md` |
| `task_016ifxg0` repetir até equivalência | closed | veredicto final |

Critérios de `plan.md` §5, conferidos nos artefactos:

- **Bateria isolada por sujeito** — `xtask/src/quality/{workspace,exec}.rs`, root
  privado, `REF/` read-only.
- **Q1–Q3 `same_session: true`; Q4 `compacted: true`** — frontmatter de
  `p1..p4/ktu-p*.md`.
- **Sonda de persistência** — `true` no resumo (`p4`), via processo novo
  (`memory list`), sem regra extra no prompt.
- **Artefactos ao lado dos referenciais** — `p1/p1-goose.md`… e `p1/ktu-p1.md`, etc.
- **Veredicto de equivalência de grau por questão** — Q1–Q4 equivalentes.

A correção exigida pelo gate (escrita automática de memória no fim do turno,
`kernel/memory_gate`, em vez de depender do modelo) está em `fact_01eob2w8` e
coberta por teste de regressão.

## 2. Itens anteriores (G0–G3) — revistos, todos OK ✅

As **28 tasks de G0–G3 estão `closed`, cada uma com `outcome` + âncoras a ficheiros
reais**. Amostra verificada por leitura:

- **G0** (7/7): workspace + firewall de camadas (`xtask check-layers`), lints,
  portas/DI com fakes, logs estruturados, métricas por feature, harness
  (unit+property+goldens+CI), `ktu init`.
- **G1** (7/7): tipos/`SessionLog` JSONL, `derive_messages` + projeção de wire, loop
  cancelável/Stagnation, `guard.rs`+`taint.rs`, `BudgetGate`, provider falso,
  `read_file`+`ktu run`.
- **G2** (9/9): `LlamaProvider` (SSE, tool-calling+fallback, retry), test double
  HTTP, tools fs, `bash` fail-closed, resume/cache de prefixo, trash+doctor,
  sampling/thinking, GGUF doctor, modelo real (Ornith).
- **G3** (5/5): spike do knudge-core, porta `Memory`+adaptador JSONL, memory gate
  (recall+write automáticos, dedup 0.75/0.92), compactação+checkpoint durável,
  `memory export`/`doctor --memory`.

## 3. Caveats / pontos a vigiar (não invalidam o gate)

1. **Provider da corrida G4** — usou `opencode-go`/`longcat-2.5-preview-free`
   (remoto), **não** o `llama.cpp` local, por falta de fundos; condição declarada no
   frontmatter e no veredicto (`decision_01sexrmm`).
2. **Q2** — entregou um REPL/`--human` equivalente, forma diferente da ideia
   esperada; aceite como equivalência.
3. **Q3/p3** — resposta no artefacto curta (~117 chars) e remete para o ficheiro; o
   entregável real existe (`ktu-files/knudge/ktu/knudge.md`, 8 KB).
4. **`.agents/memory/notes.jsonl` do repo está vazio** — esperado: a bateria corre em
   workspace isolado; este é o runtime de dev.
5. **`task_01kf735v` está `forgotten`** — duplicado de `task_00o74kgy` (HookEvent,
   G7); inofensivo, convém limpar.
6. **G5–G8 continuam `active`/não iniciadas** — coerente com “G4 bloqueia as fases
   seguintes”; o épico está `active` a 32/51.

## 4. Conclusão

Até ao G4 inclusive, tudo está consistente e verde. G0–G3 fecham com evidência e
âncoras; G4 está formalmente fechado com veredicto de equivalência, artefactos com
frontmatter e sonda de persistência a passar; `make check` verde. O único desvio
relevante (provider remoto em vez do `llama.cpp` local) está declarado e é uma
condição transparente, não uma falha do gate.

Com o G4 verde, a próxima fase desbloqueada é **G5 — Subagentes read-only + modo
plano** (`task_00j43pma`).

---

*Registo knudge associado: `fact_00uz1ew4`.*
