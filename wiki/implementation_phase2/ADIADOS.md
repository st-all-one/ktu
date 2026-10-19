# Adiados da Fase 2 — registo (absorvido pela Fase 3)

> **Épico antigo:** `epic_00ey9lmk` (**fechado** — absorvido por `epic_00j1cgo3`) ·
> **Plano de execução:** [`implementation_phase3/`](../implementation_phase3/README.md) ·
> **Lista verificada:** [dossiê de pendências](../pendencias.md) §2.
>
> Este ficheiro era a lista dos itens **adiados por YAGNI** durante a Fase 2. Depois de a Fase 2
> fechar (H0–H12, gates G10–G15 verdes), a lista foi **reavaliada** contra o código: o que valia a
> pena ganhou **plano, gate e tarefa** na Fase 3; o que não valia foi **fechado com razão** (§2).
> Não se lista mais nada aqui — a lista viva é o [dossiê](../pendencias.md) e o plano é a
> [Fase 3](../implementation_phase3/README.md).

---

## 0. Regra (continua a valer)

Um item só sai do adiamento quando existir o **sintoma/medição** que o exige (e, quando aplicável,
o subsistema que o consome). Sem isso, fica adiado. Cada item declara a **condição de reabertura**.

## 1. O que passou para a Fase 3

| Adiado (Fase 2) | Fase 3 | O que mudou na verificação |
|---|---|---|
| **A7** — inbox durável / idempotência do *steering* | [F1](../implementation_phase3/F1-steering-duravel.md) — **implementado** (gate G16 verde) | entregue na Fase 3: admissão durável no log, entrega no passo seguinte, idempotência por id |
| **A5** — REPL: Ctrl-C, Tab e `/` sem verbo | [F2](../implementation_phase3/F2-repl-interactivo.md) — **parcial** (gate G17 verde) | comandos `/` + Ctrl-C **entregues**; a **Tab** fica adiada (*raw mode* + terminal em volta dos logs) |
| **A1** — recursos globais | [F3](../implementation_phase3/F3-recursos-globais.md) | **parcialmente feito**: os guardrails globais **já são carregados** (correcção do M5 na H12); falta o `ls` com origem e skills/plugins/receitas globais |
| **A6** — fila de mutação, remédio por regra, âncoras | [F4](../implementation_phase3/F4-fiabilidade-edicao.md) — **fechado** (gate G19 verde) | fila por caminho canónico, remédio da regra + estrutural, sugestão de âncoras |
| **A2** — migrações de config | [F5](../implementation_phase3/F5-migracoes-config.md) | nada — é um **gatilho** (schema v2) |
| **A3** — `.agents/recipes/` + `ktu self setup` | [F6](../implementation_phase3/F6-self-setup.md) | **reduzido e fechado sem plano**: receitas ≡ `.agents/commands/` (H10) e o `ktu init` já escreve o bloco/skill que os clientes lêem; o `setup` não tem consumidor (a pendência **D3** fica no dossiê com o gatilho) |

## 2. O que foi fechado sem plano (e porquê)

| Item | Veredicto |
|---|---|
| **A4** — `.ktu/logs/` | **Não vale a pena.** É um **caminho**, não trabalho: a condição de reabertura é idêntica a «a feature existir». O consumidor real (log de diagnóstico) está no [dossiê](../pendencias.md) §2 (D4), ancorado a `logging.rs`/`doctor --bundle`. Tarefa `task_00ttgc9f` fechada como `abandoned`. |
| **A9, A10, A11** | **Já implementados** antes da H11 (revisão do dono). Deixaram de ser adiados — ver [H10](../implementation_phase2/H10-convencoes.md). |
| **A8** (kd) / **A7** (kd, fechado) | O *steering* entregue é a tarefa fechada do H6; o que falta (inbox durável) é o **F1**. |

## 3. Já decididos — não reabrir sem número

- **GDK** (`decision_00aw5etj`): não implementar (*alpha*; puxa MCP; desajuste arquitectural).
- **Paralelismo de tools / plugins WASM / *dispatch* paralelo** (`task_00ovthd1`): sem medição que
  o exija (o *benchmark* mostra microssegundos). **Nota:** o F4 (fila de mutação) é *consequência*
  deste — só arranca se ele arrancar.
- **Scope creep do knudge** (H7): promover só com **≥2** agentes consumidores.
- **Sandbox de SO, `sub_recipes`, `parameters`, `model`/`hidden`/`color`, `--record`**: ficam no
  [dossiê](../pendencias.md) §2, **fora** da Fase 3 (ver
  [README §4](../implementation_phase3/README.md)).

## 4. Saída

Cada item reaberto volta como **issue** na Fase 3, com o teste/sintoma que o exige e a medição que
o justifica. Até lá, o backlog activo **não** cresce com estas páginas.
