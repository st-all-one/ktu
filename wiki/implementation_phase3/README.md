# Fase 3 — implementação condicionada (pós-H12)

> **Épico:** `epic_00j1cgo3` · **knudge:** `kd ask --scope epic_00j1cgo3` · **âncora:** este
> ficheiro · **Pré-requisito:** a **Fase 2 concluída** (H0–H12 e os gates G10–G15 verdes, com
> artefactos em [`wiki/gates/`](../gates/README.md)).
>
> **Tese da Fase 3.** A Fase 2 fechou a **mecânica** (cadeia longa, três caminhos, concorrência,
> project-bound, memória) e deixou um dossiê de **adiados por YAGNI**
> ([`ADIADOS`](../implementation_phase2/ADIADOS.md), [dossiê](../pendencias.md)). A Fase 3 pega
> **só** nos adiados que **valem a pena** e dá-lhes plano, gate e artefacto — os outros **saem**
> (§4). Nada aqui é «fase 2 outra vez»: cada item tem o **gatilho declarado** e só arranca quando
> ele disparar.

---

## 0. Regra da fase (não negociável)

1. **Gatilho antes do trabalho.** Um item só arranca com o sintoma/medição que o exige
   (`plan.md` §0, YAGNI). O gatilho está declarado em cada `Fn-*.md`.
2. **Sem regressão.** Os gates da Fase 2 (G10–G15) continuam verdes; o `F2` mexe no REPL, pelo que
   **G12** (log byte-idêntico `run`/REPL/crate) é o teste de regressão obrigatório.
3. **Uma fonte de verdade.** A lista **verificada** dos adiados é o [dossiê](../pendencias.md) §2;
   este directório é o **plano de execução** (ordem, passos, tarefas kd). Nada é listado duas
   vezes com texto diferente.
4. **Portão único.** Cada issue fecha com `make check` + `make supply-chain` verdes, e com o gate
   do item (G16–G22) corrido por `make gates` e arquivado em `wiki/gates/`.
5. **Sem commits** (gesto humano); **sem `unsafe`/`unwrap`/`expect`/`panic`** em `src/` **nem** em
   testes; `#[allow]` exige `reason`.

**Invariantes novos (travados por teste):**
- **M7 — uma mensagem, uma entrega:** o *steering* admitido é entregue **exactamente uma vez**,
  mesmo após *crash*/*resume* e com o front a repetir o pedido (F1).
- **M8 — nada invisível:** todo o recurso que o runtime usa aparece num `ls`, com a **origem**
  (global/local) — o piso M5 deixa de ser invisível (F3).

---

## 1. Sequência (por valor, não por antiguidade)

```
F1 Steering durável (M7) ──────────── perda de dados real · primeiro
F2 REPL interactivo ───────────────── o uso interactivo real vai pedir
F3 Recursos globais (M8) ──────────── gatilho: 2.º projecto
F4 Fiabilidade de edição ──────────── atrás do paralelismo de tools (R5)
F5 Migrações de config ────────────── gatilho: schema v2
F6 ktu self setup ─────────────────── fechado sem plano (init ja cobre AGENTS.md/skill)
F7 Travões (links/doc + knudge CI) ── já há gatilho (PM-0003, L1)

── saída da fase: gates dos itens abertos verdes ──────────────────────────
```

| # | Issue | Âncora (desenho) | Tarefas | Gatilho |
|---|---|---|---|---|
| F1 | `task_00ly3qez` *Steering* durável — **fechada** ✅ | [F1](./F1-steering-duravel.md) | 2 | *(disparou: não perder a mensagem)* |
| F2 | `task_01mrz94g` REPL interactivo — **parcial** (Tab adiada) | [F2](./F2-repl-interactivo.md) | 2 | *(disparou: comandos `/` + Ctrl-C)* |
| F3 | `task_004bvdzb` Recursos globais | [F3](./F3-recursos-globais.md) | 2 | um 2.º projecto a precisar da mesma skill/plugin/receita |
| F4 | `task_00sb14zh` Fiabilidade de edição | [F4](./F4-fiabilidade-edicao.md) | 1 | paralelismo de tool calls (hoje rejeitado — R5) |
| F5 | `task_00g0us7a` Migrações de config — **fechada** ✅ | [F5](./F5-migracoes-config.md) | 1 | *(não disparou — mecanismo implementado; ver [ADR 0020](../adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md))* |
| F6 | `task_00odyng2` `ktu self setup` — **fechada sem plano** ⛔ | [F6](./F6-self-setup.md) | 1 | um cliente que **não** leia `AGENTS.md`/skills (ou MCP, decisão à parte) |
| F7 | `task_01nopvll` Travões — **fechada** ✅ | [F7](./F7-travoes.md) | 2 | *(disparou: PM-0003, L1)* |

**Estado:** **F1 fechada** (gate **G16** verde, 9/9 — [relatório](../gates/G16-steering.md));
**F2 parcial** (gate **G17** verde, 10/10 — [relatório](../gates/G17-repl.md)): os comandos `/` e o
Ctrl-C estão entregues; a **Tab** fica adiada (exige *raw mode* + gestão do terminal em volta dos
logs do turno). **F3 fechada** (gate **G18** verde, 10/10 — [relatório](../gates/G18-recursos.md)):
recursos globais (skills/plugins/receitas) e o piso visível (M8). **F4 fechada** (gate **G19**
verde, 8/8 — [relatório](../gates/G19-edicao.md)): fila de mutação por caminho (do **processo**),
remédio por regra e âncoras próximas. **F7 fechada** (gate **G22** verde, 6/6 —
[relatório](../gates/G22-travoes.md)): `check-links` + `cargo doc -D warnings` dentro do
`make check` e o portão do knudge na CI — **fecha a Fase 3**. **F5 fechada** (gate **G20** verde,
7/7 — [relatório](../gates/G20-migracao.md)): versão de schema, migração idempotente, leitura não
destrutiva, fail-closed em versão futura e `ktu config doctor` — feita **sem o gatilho** (o
mecanismo é pré-condição de qualquer migração; desvio registado na
[ADR 0020](../adr/0020-schema-de-config-versionado-com-leitura-nao-destrutiva.md)).
**F6 fechada sem plano** (sem gate): o `ktu init` já entrega o que tem consumidor (bloco no
`AGENTS.md` + skill) e o ktu não tem servidor MCP — a análise está no
[F6](./F6-self-setup.md) e o critério de reabertura fica em **D3**. Com isto, **todos** os itens
da Fase 3 estão fechados.

**Total:** 7 issues · **11 tarefas**. Duas issues (F1, F3) e uma (F7) reutilizam tarefas já
existentes do épico antigo (`epic_00ey9lmk`), que foi **absorvido** por este.

---

## 2. Método (não negociável)

1. **Simples primeiro.** O mais simples que fecha o item; complexidade só com número.
2. **Através da porta.** Nenhum caminho novo tem lógica própria; todos falam à `ktu-port` (M4).
3. **O kernel não gera ids nem tempo.** O que é não-determinístico (ids, sinais, *raw mode*) vive
   na **borda** (I6).
4. **Teste antes de fechar.** Cada tarefa aponta o teste que a prova; `kd task close --outcome`.
5. **Artefacto ou não aconteceu.** O gate arquiva a prova em `wiki/gates/artifacts/<gate>/`.

---

## 3. Como executar

```bash
kd ask --scope epic_00j1cgo3           # contexto do épico
kd task list --parent task_00ly3qez    # tarefas de uma issue
kd task close --id <task> --outcome success --note '<evidência>'
kd doctor --fix                         # saúde do corpus
make gates ARGS='--only G16'            # gate do item (artefacto em wiki/gates/)
```

Cada documento `Fn-*.md` traz: **objetivo**, **estado verificado**, **desenho/passos**,
**tarefas kd**, **gate/testes** e **riscos**.

---

## 4. O que fica **fora** (e porquê)

O dossiê [pendencias.md](../pendencias.md) continua a ser o registo completo. Ficam **fora** da
Fase 3:

| Item | Porque não vale a pena |
|---|---|
| **F6 — `ktu self setup`** | O `ktu init` já escreve o bloco de protocolo no `AGENTS.md` (a convenção que os clientes lêem) e a skill; o ktu **não tem** servidor MCP (MCP fora por YAGNI), pelo que não há nada de novo a instalar num cliente. A análise completa está no [F6](./F6-self-setup.md) e a pendência (**D3**) fica no dossiê com o gatilho afinado. |
| **A4 — `.ktu/logs/`** | É um **caminho**, não trabalho: a condição de reabertura («existir log de runtime em ficheiro») é idêntica a «a feature existir». O consumidor real está no dossiê (D4 → `logging.rs` / `doctor --bundle`). Fechado como `abandoned`. |
| **A9–A11** | **Já implementados** antes da H11 (revisão do dono); deixaram de ser adiados. Ver [H10](../implementation_phase2/H10-convencoes.md). |
| **D8 — sandbox de SO** | Sem sintoma que o confinamento de *path* não resolva; custo alto e risco de portabilidade. |
| **D10/D11 — `sub_recipes`, `parameters`** | Sem consumidor: nenhuma receita real precisa de compor outra nem de validar por tipo. |
| **D12 — `model`/`hidden`/`color` nos agentes** | Sem consumidor: o provider é por **sessão**, não por agente; não há UI que os use. |
| **D9 — `--record`** | Enabler dos gates; o guião escrito à mão (`--provider scripted`) chega e é revisto como código. |
| **R1–R7** (rejeitados) | Cada um tem **número** ou `unpriced` no dossiê §3. Não voltam sem cruzar o número. |

> Um item fora desta lista **não** está esquecido: está no dossiê com a condição de reabertura.
> Reabre-se como **issue nova** (com o sintoma que o justifica), não por arrasto.
