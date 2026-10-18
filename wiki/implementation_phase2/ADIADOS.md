# Fase 3 — Adiados (YAGNI), posterior a H12

> **Épico:** `epic_00ey9lmk` · **Âncora:** este ficheiro · **Pré-requisito:** a Fase 2
> **concluída** (H0–H12 e os gates G10–G15). Nada aqui entra **antes** de a Fase 2 fechar: cada
> item foi **adiado por YAGNI** (§0 do `plan.md`) — hoje não há teste/sintoma que o exija nem, em
> vários casos, o subsistema que o consome.

---

## 0. Regra

Um item só sai daqui quando existir o **sintoma/medição** que o exige (e, quando aplicável, o
subsistema que o consome). Sem isso, fica adiado. Cada item declara a **condição de reabertura**.

## 1. Itens adiados

### A1 — Recursos globais (skills/guardrails/plugins)

- **O quê:** `~/.config/ktu/{skills,guardrails,plugins,recipes}/` como *defaults*; o local
  sobrepõe por nome; **guardrails globais = piso aditivo** (M5, nunca removível).
- **Porque adiado (YAGNI):** o runtime **ainda não carrega** skills nem guardrails (só
  `ktu guardrails list`); a camada global não tem consumidor.
- **Reabrir quando:** o runtime carregar guardrails declarativos e/ou skills (composição do
  prompt). Então a precedência `local > global` estende-se aos recursos e o piso M5 é travado por
  teste.
- **Custo:** médio. **Tarefa kd:** `task_00ni0ppt`.

### A2 — Migrações de configuração

- **O quê:** versionar o schema da config + migração idempotente (destrutiva na **escrita**,
  não-destrutiva na **leitura**), no modelo do `goose`.
- **Porque adiado (YAGNI):** o schema é **v1**; não há versão anterior a migrar.
- **Reabrir quando:** o schema mudar (**v2**) e existirem configs v1 em uso.
- **Custo:** baixo. **Tarefa kd:** `task_015a04hb`.

### A3 — `.agents/recipes/` + `ktu self setup`

- **O quê:** dir de receitas + `ktu self setup <cliente>` (instala a *recipe* de
  `claude`/`cursor`/`codex`/`pi`).
- **Porque adiado (YAGNI):** o sistema de comandos existe desde a H10 (`.agents/commands/*.md`,
  `/nome args`, `ktu run --command`, `settings`); o que falta é **instalar** receitas de clientes
  externos (`claude`/`cursor`/`codex`/`pi`) e o comando `ktu self setup` — não há ainda essas
  receitas escritas nem consumidor.
- **Reabrir quando:** existirem receitas de cliente a instalar (ou um pedido concreto de setup).
- **Custo:** baixo-médio. **Tarefa kd:** `task_01oo6bry`.

### A4 — `.ktu/logs/`

- **O quê:** dir de logs de runtime.
- **Porque adiado (YAGNI):** nada escreve lá — os logs vão para `stderr` e o evento para o log da
  sessão.
- **Reabrir quando:** existir um log de runtime em ficheiro (ex.: telemetria/OTel).
- **Custo:** trivial. **Tarefa kd:** `task_00ttgc9f`.

### A5 — REPL: Ctrl-C, Tab e comandos `/` que faltam à porta

- **O quê:** (a) **Ctrl-C** para cancelar o turno em curso; (b) **Tab** para completar `/` e
  caminhos; (c) `/compact`, `/resume`, `/fork`, `/undo`, `/redo`, `/memory` no REPL.
- **Porque adiado (YAGNI/M4):** (a)/(b) exigem *raw mode* (crossterm) e/ou `stdin` assíncrono
  (*features* `signal` + `io-std` do `tokio`), que o REPL actual (linha a linha, sem *raw mode*)
  não tem; (c) os comandos `/` que faltam **não têm verbo na porta `Host`** — e o REPL **não**
  inventa lógica (M4: "todos comandos da porta"). Só foram implementados os que mapeiam verbos
  existentes (`/help`, `/quit`, `/plan`, `/build`, `/mode`, `/cancel`).
- **Reabrir quando:** a porta `Host` ganhar os verbos `compact|fork|undo|redo|memory` (então o
  REPL expõe-os) e quando houver *raw mode*/`signal` (Ctrl-C/Tab).
- **Custo:** médio. **Tarefa kd:** `task_00dbywbk`.

### A6 — H5: fila de mutação, remédio por regra e âncoras próximas

- **O quê:** (a) fila de mutação **por caminho** (chave canónica) que serializa escritas ao mesmo
  ficheiro; (b) `remedy` **por regra** de guardrail; (c) o `edit` a sugerir as **âncoras únicas
  mais próximas** quando a âncora não é encontrada.
- **Porque adiado (YAGNI):** (a) as tools correm **sequencialmente** numa única thread T2
  (`WorkerCtx::serve` serve um pedido de cada vez; os subagentes partilham a mesma T2) — não há
  escrita concorrente ao mesmo ficheiro; (b) `Decision::Deny(DenyReason)` não transporta dados, e
  o remédio **estrutural** por `DenyReason` já cobre todos os motivos; (c) a mensagem actual já
  devolve a âncora procurada.
- **Reabrir quando:** houver paralelismo de tools; uma regra precisar de remédio próprio; ou
  houver sintoma de âncoras erradas.
- **Custo:** baixo-médio. **Tarefa kd:** `task_01qsu6vf`.

### A7 — Inbox durável / idempotência do *steering*

- **O quê:** o enfileiramento **durável** do *steering* (gravado no momento da admissão, não
  só na entrega) e **idempotência por ID**.
- **Porque adiado (YAGNI):** o *steering* **já funciona** — é entregue numa fronteira segura
  (início de passo), **logado** como `Event::Steer` (durável no log da sessão, que é a fonte da
  verdade) e projectado para o modelo (*model-visible ⟺ logged*). O que falta é apenas o
  enfileiramento durável *antes* da entrega (perde-se se o processo morrer a meio) e o ID de
  idempotência — nenhum consumidor precisa de nenhum dos dois hoje.
- **Reabrir quando:** houver um front multi-cliente (web/TUI) onde a admissão tenha de ser
  idempotente, ou um requisito de não perder a mensagem num *crash*.
- **Custo:** médio. **Tarefa kd:** `task_01vw262v`.

### A9–A11 — implementados antes da H11 (revisão do dono)

Os três itens abaixo foram **adiados na primeira passagem da H10** e **implementados de seguida**,
a pedido do dono do projecto, antes de a H11 arrancar. Ficam aqui só como registo (as tarefas kd
`task_00mtxl9i`, `task_01moxmoz` e `task_01963li2` foram fechadas):

- **A9 — receitas com `settings`/*shell* guardado:** `CommandSettings` (`provider`/`base_url`/
  `model`/`max_steps`), argumentos `$1`…`$9`, `shell: true` com `!cmd` expandido **depois** de
  `facade::evaluate_fact` (estrutural → guardrails → permissões) e `ktu run --command`. Continua
  fora `sub_recipes` e `parameters` declarados (sem consumidor).
- **A10 — exposição `declared`/`deferred` + `tool_search`:** `ToolExecutor::deferred`,
  `Config::deferred_tools`, `Toolbox::with_deferred`, `tool_search` (BM25 determinístico) e
  promoção registada em `tool.loaded` para o pedido seguinte.
- **A11 — `AGENTS.md` na leitura de um ficheiro:** porta `Instructions`, anexação ao resultado de
  `read_file`/`edit_file` (uma vez por turno e por fonte), evento `context.local`.

## 2. Já decididos — não reabrir sem número

- **GDK** (`decision_00aw5etj`): não implementar (alpha; puxa MCP; desajuste arquitectural).
- **Paralelismo de tools / plugins WASM / dispatch paralelo** (`task_00ovthd1`): sem medição que
  o exija (o benchmark mostra microssegundos).
- **Scope creep do knudge** (H7): promover só com **≥2** agentes consumidores.

## 3. Saída

Cada item reaberto volta como **issue** na Fase 3, com o teste/sintoma que o exige e a medição que
o justifica. Até lá, o backlog activo **não** cresce com estas páginas.
