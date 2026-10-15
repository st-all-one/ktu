# Épico ktu — plano sequencial de execução

> **knudge (memória do projecto).** Este épico vive no próprio repositório, gerido por `kd`.
> Raiz: **`epic_00ynu6q8`** (scope `epic`). A árvore completa está em §4; o conhecimento original
> foi organizado em notas (fact/decision/error/risk) consultáveis por `kd ask`.
>
> **Tese:** *fazer funcionar, depois melhorar.* A 1.ª tentativa (`REF/katu`, 70k linhas) optimizou
> antes de funcionar. O ktu constrói um **slice vertical que funciona** e só avança quando o
> critério de qualidade é atingido.
>
> Referência de memória: **`REF/knudge`** (contrato adoptado — §2.7 do [`plan.md`](plan.md)).

---

## 0. O gate inegociável

> **O agente PRECISA alcançar o `quality_test` (bateria `wiki/quality_test/p1`–`p4`, como sujeito
> isolado, com persistência entre sessões). Caso contrário, não prossegue para as fases seguintes.**

- O gate é **G4** (`task_010xrtzt`). Enquanto G4 não passar, **G5–G8 ficam bloqueadas**.
- Reprovar o gate **não** autoriza acrescentar sistemas: autoriza **melhorar a qualidade** do que
  já existe (contexto, prompt, tools, memória) e repetir a bateria.
- O bar é **equivalência de grau** com goose/pi/opencode — não vitória, não derrota: mesma
  qualidade, mesmas condições declaradas no artefacto.

---

## 1. Método de trabalho (como construir — não negociável)

1. **Simplicidade primeiro.** O mais simples e óbvio que fecha o slice; complexidade só entra
   depois, e sempre com um teste que a exija. Nada de antecipar optimização.
2. **Programação funcional com injecção de dependência.** Domínio **puro** (funções sobre dados),
   efeitos confinados atrás de **portas** (`Clock`, `Rng`, `Fs`, `Env`, `Git`, `Logger`) com
   implementações reais na borda e **fakes** nos testes. Sem `unsafe`, sem `unwrap/expect/panic`,
   iteração determinística.
3. **Logs estruturados.** `stdout = dados`, `stderr = logs`; ids de evento **estáveis** (nunca
   texto livre como contrato); custo zero por omissão, ligados por feature.
4. **Métricas de performance por função** (como o katu): *spans* por função, zero-cost por omissão,
   export em JSON; um número sem artefacto não fundamenta decisão.
5. **Testes rigorosos.** Unit + *property* + *goldens* (contrato de bytes) + integração com
   **provider falso** e *test double* HTTP; guardrails têm **fixtures adversariais**.
6. **Seguro e estável.** Guardrails determinísticos (§0 do plan.md), fail-closed, `-D warnings`,
   ficheiros pequenos, revisão contra a lista do “não-fazer”.

---

## 2. Prioridade (ordem de valor, explícita)

| # | Objectivo | Traduzido |
|---|---|---|
| **1** | **Funcionar como agente moderno** | sessão persistente + contexto com orçamento + **cache** (prefixo) + tools |
| **2** | **Qualidade do resultado entregue** | atingir o `quality_test` (G4) |
| **3** | **Performance** | **terciária** — só depois do gate, só com número medido |

---

## 3. Protocolo knudge (como operar este épico)

```bash
kd prime                      # protocolo completo (1x por sessão)
kd ask "<pergunta>"           # ANTES de criar: consulte a memória
kd write --summary "<...>"    # registe cada aprendizado (uma afirmação por nota, --anchor PATH)
kd task list --scope epic --universe      # estado do épico
kd task show --id epic_00ynu6q8             # a árvore
kd task close --id <ID> --outcome success --note "<evidência>"
kd rewind                     # situe a próxima sessão ao encerrar
kd sync --message "ktu: <marco>"
```

Regras: **buscar antes de gravar** (dedup `0.75`/`0.92`); **ancorar código** (`--anchor plan.md`,
`--anchor crates/...`); **uma afirmação por nota**; tarefas fecham **com evidência**.

---

## 4. Sequência de execução

| Issue | Fase | Gate |
|---|---|---|
| **G0** | Fundação & método | — |
| **G1** | Loop mínimo com sessão e contexto (prova de vida) | pré-requisito |
| **G2** | Agente moderno real (llama.cpp + tools + cache) | pré-requisito |
| **G3** | Memória knudge & persistência | pré-requisito |
| **G4** | **Alcançar o quality_test** | **⛔ INEGOCIÁVEL** |
| **G5** | Subagentes read-only + modo plano | pós-gate |
| **G6** | Porta única + front-ends | pós-gate |
| **G7** | Plugins + hot reload + guardrails declarativos | pós-gate |
| **G8** | GDK, releases, otimização | pós-gate |

#### G0 — Fundação & método: workspace, portas/DI, logs estruturados, métricas por função, testes
*knudge:* `task_00qm4eu6`

- [ ] `task_01i0wipl` Criar workspace Rust 2024 com crates ktu-core/tools/provider/port/cli/memory e firewall de camadas (xtask check-layers)
- [ ] `task_01yiyq91` Fixar lints do repo (clippy.toml/rustfmt.toml) e Makefile check = fmt + clippy -D warnings + test
- [ ] `task_00ek2v0f` Definir portas com injecção de dependência (Clock/Rng/Fs/Env) com fakes; domínio puro e sem unsafe/unwrap/expect/panic
- [ ] `task_00mow2ux` Padrão de logs estruturados: stdout=dados, stderr=logs, ids de evento estáveis, custo zero por omissão
- [ ] `task_00krcevy` Métricas de performance por função (spans zero-cost atrás de feature instrument) com export JSON
- [ ] `task_017rdmtb` Harness de testes: unit + property + goldens + provider falso com fixtures; CI mínimo
- [ ] `task_00x02111` ktu init: criar layout .agents/ (skills, guardrails, memory) e .ktu/ (runtime); knudge ja inicializado

#### G1 — Loop mínimo com sessão e contexto (prova de vida, provider falso)
*knudge:* `task_01xvduai`

- [ ] `task_00f82mso` Tipos do núcleo (Message/ToolCall/ToolResult/Turn/StopReason) e log JSONL append-only como fonte da verdade
- [ ] `task_01bfp4a5` derive_messages + projeção de wire (1 passo = 1 assistant com N tool_calls + N tool results)
- [ ] `task_00o2ycqi` Loop completo: cancelável, StopReason obrigatório, detecção de estagnação (agent.stall)
- [ ] `task_013gz0iw` Guard estrutural mínimo: capacidade do AgentSpec, confinamento de path, caps; envelope taint (I13)
- [ ] `task_0118dhru` BudgetGate com dono único + fixar a instrução corrente (I14)
- [ ] `task_00mabsbe` Provider falso com fixtures: replay determinístico byte-a-byte (I6)
- [ ] `task_01fzd1u5` Tool read_file + comando ktu run embedded; system prompt composto (base + AGENTS.md condensado)

#### G2 — Agente moderno real: provider llama.cpp, tools de filesystem, sessão e cache
*knudge:* `task_00sv9qlq`

- [ ] `task_016rvfop` Provider llama.cpp via llama-server (chat/completions streaming SSE, tool-calling nativo + fallback texto)
- [ ] `task_01j2o6cg` Test double HTTP com fixtures para exercitar streaming/parsing sem modelo real
- [ ] `task_00si6z5t` Tools de filesystem: read/write/edit/list/glob/grep com erros accionáveis e paths resolvidos no root
- [ ] `task_01bzrwbx` bash com timeout/cap + análise de argv + retry/backoff do provider
- [ ] `task_01ptulp8` Sessão: resume por id + cache de prefixo (determinismo de prefixo; não invalidar) + checkpoint
- [ ] `task_01ma94kf` Trash em vez de apagar (recuperável, com índice) + ktu providers doctor

#### G3 — Memória knudge & persistência (sessão a sessão)
*knudge:* `task_01iz1xh1`

- [ ] `task_00uz2my4` Validar se knudge-core indexa corpus JSONL; definir schema físico de .agents/memory/notes.jsonl
- [ ] `task_00c0u0vr` Porta Memory (tipos do ktu) + adaptador knudge-core; hints são ponteiros (id+statement+score), nunca o corpo
- [ ] `task_00z0ju90` Memory gate no loop: recall de início de sessão + pre_write com dedup idempotente (0.75 cria / merge / rejeita)
- [ ] `task_01jeruz5` Compactação aditiva + checkpoint tipado durável (temp->fsync->rename) + recuperação de crash
- [ ] `task_01s1vlkb` ktu memory export --md (vista derivada) + ktu doctor --memory (âncoras quebradas, saúde do índice)

#### G4 — GATE inegociável: alcançar o quality_test p1-p4 (equivalência + persistência) ⛔ **GATE INEGOCIÁVEL**
*knudge:* `task_010xrtzt` — **bloqueia G5–G8 até passar.**

- [x] `task_013dfilu` scripts/quality.sh + xtask quality: bateria isolada por sujeito (mesmas perguntas/contexto, sem interferência)
- [ ] `task_00vq487c` Executar p1-p4 com frontmatter de condições + sonda de persistência entre sessões
- [ ] `task_00kz804y` Julgar por expected_answer_ideia e arquivar wiki/quality_test/p*/ktu-*.md ao lado dos referenciais
- [ ] `task_016ifxg0` Repetir até equivalência de grau; BLOQUEIA todas as fases seguintes enquanto não passar

#### G5 — Subagentes read-only (explorer) com fan-out + modo plano
*knudge:* `task_00j43pma`

- [ ] `task_00zjgygk` AgentSpec + registo de agentes (primário ktu + explorer read-only estrutural)
- [ ] `task_01rbaqmz` Tool subagent: sessão-filha com contexto fresco, relatório capado, profundidade máxima 1
- [ ] `task_01sai4yj` Fan-out: N subagentes em paralelo com join e caps; ordem de resultados determinística no log
- [ ] `task_018f3xhq` Modo plano: AgentSpec read-only + tool plan + artefacto .ktu/plan/<sessao>.json + transição aprovada (I15)

#### G6 — Porta única agnóstica + front-ends (CLI/TUI/web)
*knudge:* `task_00af69ek`

- [ ] `task_007t2rm8` ktu-port: protocolo versionado + transporte Channel (embedded) e Http (HTTP/1.1+HTTP/2, sem H3)
- [ ] `task_01s7s1ka` ktu serve (axum/hyper) com SSE de eventos, health e estado de sessão
- [ ] `task_01czs2ai` Comandos out-of-band na porta (cancel, SetMode, continue) logados e sobreviventes a resume
- [ ] `task_0144p1iv` TUI mínima (ratatui) como cliente da porta
- [ ] `task_01wwavuq` Página web de demonstração (fetch + SSE) como cliente da porta

#### G7 — Plugins em runtime com hot reload + guardrails declarativos
*knudge:* `task_002xypc1`

- [ ] `task_00o74kgy` HookEvent tipado + dispatcher + pontos essenciais (session/user/model/action/guard/kernel)
- [ ] `task_010xowkm` Schema e seed de .agents/guardrails/*.md com loader fail-closed (I16)
- [ ] `task_01gsm1nb` Runtime de plugins em processo externo (JSON-lines sobre stdio) + manifesto plugin.toml
- [ ] `task_01oo7e84` Hot reload por gerações: ordem kernel->gate->execução, gate fail-closed (I11/I12)
- [ ] `task_00uuq3x5` Plugin de exemplo: transforma user.input + gate que nega bash destrutivo

#### G8 — Pós-gate: GDK, releases e otimização terciária (só com número)
*knudge:* `task_00fgs8ld`

- [ ] `task_010qrmko` Adaptador GDK (goose-sdk) atrás da feature gdk para os demais providers
- [ ] `task_01r8grbe` install.sh + packaging + CI/MSRV + releases
- [ ] `task_005zfzx3` Benchmarks com evidência (métrica + artefacto) e otimização guiada por medição
- [ ] `task_00ovthd1` Opcionais pós-medida: paralelismo de tools, plugins WASM, dispatch paralelo

---

## 5. Gate G4 — critérios de passagem

1. `xtask quality` corre a bateria **isolada por sujeito** (mesmas perguntas/contexto, sem
   interferência; `.agents/memory` e sessão privadas).
2. Q1–Q3 na mesma sessão; **Q4 com `compacted: true`** (compactação real).
3. **Sonda de persistência**: mata o processo, abre sessão nova, recupera o que Q3 gravou em
   `.agents/memory/notes.jsonl`.
4. Artefactos em `wiki/quality_test/p*/ktu-*.md`, com frontmatter de condições, ao lado dos
   referenciais.
5. **Veredicto de equivalência de grau** por questão. Sem isso, o épico **não avança**.

---

## 6. Riscos herdados (ver `kd ask "risk"`)

| Risco | Resposta |
|---|---|
| Modelo local fraco em tool-calling | exigir GGUF 4–8B com tool-calling; testes do loop usam provider falso |
| `knudge-core` não indexar corpus JSONL | validar no início de G3; se não, projetar registos → vista de notas |
| Tentação de reintroduzir policy/audit “já” | nada entra sem teste que o exija (revisão contra o “não-fazer”) |
| Contexto pequeno → thrashing/loop | `BudgetGate` dono único, `raw_min` alto, pin da instrução |
| Um `assistant` por tool_call → HTTP 400 | projeção de wire separada (1 assistant + N tool results) |

---

## 7. Definição de pronto

1. `ktu run` / `ktu serve` / crate / TUI / web operam a **mesma** sessão pela **mesma** porta.
2. Kernel e tools em threads separadas, sem estado partilhado com o front-end.
3. Só llama.cpp built-in; o resto via GDK (feature).
4. Memória em `.agents/memory/*.jsonl` (fonte da verdade) com índice derivado (knudge-core).
5. Um agente; subagentes read-only com fan-out; modo plano read-only estrutural.
6. Conhecimento em `.agents/` (skills, guardrails, memória) e runtime em `.ktu/`.
7. Guardrails em 3 camadas (código → declarativos → plugins), fail-closed.
8. Plugins com hot reload por gerações, ordem kernel → gates → execução.
9. **G4 verde** (equivalência + persistência).
10. Só então: um único documento de optimização, com números.
