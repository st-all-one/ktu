# Revisão por propósito — o ktu como agente mínimo, kernel e project-bound

> Segunda revisão crítica, agora **orientada ao propósito** (não aos referenciais). Pergunta única:
> **o ktu, depois de provar que é usável, consegue ser ao mesmo tempo *mínimo*, *kernel*, *seguro
> por padrão*, *independente do caminho*, *determinístico no controlo* e *project-bound* — e servir
> as quatro utilizações reais?** Data: 2026-10-17.
>
> Documentos irmãos: [análise crítica do backlog](./backlog_reviews.md) ·
> [consolidação das duplicatas](./backlog/consolidado.md) ·
> [CLI e configuração](./cli-e-config.md).

---

## 0. O que o ktu é hoje (verificado no código)

| Capacidade | Estado | Evidência |
|---|---|---|
| Núcleo puro + portas | ✅ | `ktu-core/src/ports/` (`Clock`,`Rng`,`Fs`,`Env`,`Logger`,`Metrics`); `#![forbid(unsafe_code)]` |
| Firewall de camadas | ✅ | `xtask check-layers` no `make check` |
| Loop determinístico | ✅ | I1–I6; `derive_messages(log)` única fonte; provider fake + replay |
| Guardrails fail-closed | ✅ | I10/I11/I16; `Guard::evaluate` sobre factos tipados; `.agents/guardrails/*.md` |
| Modo plano read-only | ✅ | I15; `AgentSpec` só-leitura + `plan`; transição aprovada |
| Taint (dado ≠ instrução) | ✅ | I13; envelope que não muda comprimento |
| Memória project-bound | ✅ | `.agents/memory/*.jsonl`; `ktu-memory` (porta `Memory` + knudge) |
| Convenções | ✅ | `.agents/` (skills/guardrails/memory) + `AGENTS.md` como *router* (condensado) |
| Caminhos de interacção | ⚠️ | `ktu run` (1 turno), `ktu serve`, `ktu tui` (cliente), **crate** (raw) |
| Configuração | ❌ | só defaults no código + env + flags; **sem ficheiro de config** |
| Usabilidade medida | ⚠️ | G4 verde **com o modelo de referência** e a bateria p1–p4 |

**Leitura:** os seis invariantes do novo propósito estão, na sua maioria, **já implementados** — o
que é excelente. A fase nova **não é construir o núcleo; é (a) completar os caminhos, (b) fechar a
configuração, (c) provar fiabilidade com modelos arbitrários.**

---

## 1. As tuas afirmações, avaliadas criticamente

### A1. "O ktu corrigiu o erro do katu; agora é usável de facto." — **✅ com ressalva**
**Verdade:** G4 fechado; `quality_test` **equivalente** aos referenciais; persistência entre sessões
verde (Q3→Q4, sonda de processo novo).
**Ressalva:** "usável" foi medido com **um** modelo de referência e uma bateria de 4 perguntas. Com
um modelo **fraco/arbitrário** (tool-call em texto, JSON quebrado, eco), o ktu ainda não está
provado — é precisamente o Tier 1 (`65` eco, `67`/`107` tool calls, `70` leitura). **Correcção à
afirmação:** *"usável de facto **com o modelo de referência**; a robustez a modelos arbitrários é
trabalho em aberto."*

### A2. "Agora o objetivo é garantir mínimo, kernel, seguro por padrão, independente do caminho, determinístico no controlo, e incorporar convenções globais." — **✅ como filtro; ⚠️ num ponto é trabalho novo**
**Verdade:** é o **enquadramento certo**, e — importante — a maioria já é *design* actual. O valor
da frase é mudar o **filtro**: deixa de se perguntar "que feature falta?" para "isto preserva os
invariantes em **todos** os caminhos?".
**Ponto crítico:** *"controlável de maneira determinística, agindo SOBRE o agente de forma
arbitrária"* é a parte mais forte **e** a menos construída. Hoje o controlo é sobre **efeitos**
(guardrails sobre tool calls). "Agir sobre o agente" — injectar, redireccionar, parar a meio —
é o **plano de controlo** (steering/admissão, G14; comandos out-of-band da porta, §2.12-B), que
está em *seam*. **Correcção:** o controlo determinístico existe sobre efeitos; o controlo sobre o
**fluxo** é um gap explícito.

### A3. As quatro utilizações — **cada uma tem um gap concreto** (ver §2)

### A4. "Tudo vinculado ao projecto, não ao agente." — **✅ princípio; ❌ a config ainda não o cumpre**
**Verdade:** memória (`.agents/memory`), guardrails, skills, sessões (`.ktu/sessions`) são
**project-bound**.
**Falha:** a **configuração** não é — vive em defaults do código + env + flags. Não há um ficheiro
do projecto que defina comportamento. **Correcção:** o princípio exige uma **config de projecto**
(G9), e a regra de que **não há config global que sobreponha o projecto**.

### A5. "Da maneira mais simples, com boa qualidade, previsível e otimizada." — **✅ filtro, com tensão**
**Tensão:** "4 usos + config + controlo" empurra para **superfície**; "mínimo" puxa para o
contrário. A resolução é uma **disciplina de três regras** (§3): caminhos novos reutilizam a porta;
config nova é **um** ficheiro declarativo; controlo novo reutiliza `Guard::evaluate`.

---

## 2. As quatro utilizações face ao estado real

> **Nota de prioridade (do dono):** o **CLI é o front primário** — é nele que se passa o maior
> tempo, directa ou indirectamente. O TUI desce a secundário (cliente da porta). O desenho da
> superfície e da configuração está em [CLI e configuração](./cli-e-config.md).

### Uso #1 — embutir o core noutro projecto Rust (crate)
- **Hoje:** `ktu-core` é puro e público, mas a entrada é **funcional e crua**:
  `run_turn(log, deps, config)` com `Deps`/`Ports`/`LoopConfig`/`Capabilities`/`Telemetry`.
- **Gap:** **fachada/builder** estável (`Ktu::builder()…build()`, `ktu.turn()`), que **esconde** o
  kernel (como `Knudge`/`KnudgeBuilder`, D214).
- **Item:** [crate-facade](./backlog/crate-facade.md).

### Uso #2 — CLI em pipelines, N ktus independentes
- **Hoje:** `ktu run --session <id>` persiste; `stdout`=dados/`stderr`=logs (bom para pipes). Mas o
  default é `--session default` → **dois processos em paralelo colidem** no mesmo JSONL.
- **Gaps:** **modo efémero** (`--ephemeral`) + **lock de turno** (`64`).
- **Item:** [modo-efemero](./backlog/modo-efemero.md).

### Uso #3 — conversa longa, multi-turno, CLI **e** TUI (especialmente CLI)
- **Hoje:** `ktu run` corre **um** turno e sai; `ktu tui` é interactivo mas **cliente de `serve`**.
- **Gaps:** **REPL no terminal** (multi-turno, mesma sessão, sem servidor); e, para conversa longa,
  **steering/admissão** (G14) e **snapshots/fork** (G12).
- **Itens:** [cli-repl](./backlog/cli-repl.md), [42 steering](./backlog/steering-followup.md),
  [40 sessão](./backlog/session-tree.md).

### Uso #4 — config na ponta (projecto): skills, tools, guardrails, personalidade
- **Hoje:** skills ✅, guardrails ✅, memória ✅, `AGENTS.md` ✅. **Falta:** config de
  **exposição de tools** (allow/deny), **directriz/personalidade**, **orçamento de contexto**.
- **Gaps:** **config de projecto** (G9); **preparação do ambiente** (`ktu init`); **exposição de
  tools** (`93`); **`SYSTEM.md`/`APPEND_SYSTEM`** (`97`); **agentes declarativos** (`51`);
  **`AGENTS.md` aninhados** (`52`); **config global** (`~/.config/ktu/`); **embeddings opcionais**.
- **Itens:** [ktu-init](./backlog/ktu-init.md), [config-global](./backlog/config-global.md),
  [76 config](./backlog/config-precedencia.md), [93 exposição-tools](./backlog/exposicao-tools.md),
  [97 system-prompt](./backlog/system-prompt-override.md), [51 agentes-declarativos](./backlog/agentes-declarativos.md),
  [52 AGENTS.md aninhados](./backlog/agents-md-aninhados.md),
  [memory-drain-service](./backlog/memory-drain-service.md).

---

## 3. Os invariantes do novo propósito (o novo filtro)

| # | Invariante | Exigência | Regra operacional |
|---|---|---|---|
| **M1** | **Mínimo** | nenhuma feature nova sem consumidor; cada uma reduz ou mantém a superfície | teto de superfície (`62`); uma config, uma porta, um parser |
| **M2** | **Kernel** | o núcleo é puro; efeitos só por portas; firewall intacto | `xtask check-layers` verde; nenhum caminho fala com o kernel directamente |
| **M3** | **Seguro por padrão** | o default nega; nada destrutivo irreversível; segredos fora | fail-closed (I10/I11/I16); `trash`; `deny-secret` |
| **M4** | **Independente do caminho** | **toda** interacção passa pela **mesma porta**; nenhum caminho tem lógica própria | teste "mesma sessão ⇒ mesmos eventos/log" para `run`/REPL/TUI/crate/serve |
| **M5** | **Controlo determinístico** | o controlador age **sobre** o agente; nunca depende da obediência do modelo; tudo é log | guardrails sobre factos; steering/admissão; comandos da porta; `StopReason` |
| **M6** | **Project-bound** | memória, config, guardrails, skills, sessões no projecto; **sem estado global do agente** | nada de `~/.config/ktu` que altere comportamento; env só como precedência de runtime |

**Teste transversal novo (proposto):** *"dois caminhos, uma sessão"* — correr o mesmo prompt pelo
`run`, pelo REPL, pelo `tui` e pelo crate (via porta) e provar que o **log é idêntico** (I3/I6). É
o que transforma "independente do caminho" de intenção em invariante travado por teste.

---

## 4. "Tudo project-bound" — o que falta

O princípio está certo e quase cumprido. A peça em falta é **uma config de projecto** (G9), com
estas propriedades:

- **Um** ficheiro declarativo, versionável: `.agents/ktu.toml` (ao lado do `AGENTS.md`).
- Cobre: **tools** (expor/ocultar), **orçamento** (contexto/geração), **provider/modelo** default,
  **directriz** (personagem/orientação do agente no projecto), **guardrails** (referência; as regras
  continuam em `.agents/guardrails/`).
- **Precedência:** runtime (flags) > projecto > global > env. O **projecto ganha** sobre qualquer
  default; **segredos só no global** (como o knudge).
- **Sem DSL:** campos simples e validados por schema (G9). Nada de plugins para configurar.

**Regra dura (M6):** nenhum ficheiro de config **global** altera o comportamento de um projecto sem
que o projecto o permita. O agente **não** tem preferências próprias — tem as do projecto.

---

## 5. Re-triage do backlog pela nova perspectiva

Depois da [consolidação](./backlog/consolidado.md) (36 páginas → 15 decisões), o backlog re-sortado
pelos invariantes M1–M6 e pelas 4 utilizações dá **três baldes**:

### 5.1 **Útil** — serve directamente o propósito (fazer)

| Eixo | Itens | Porquê |
|---|---|---|
| **Caminhos** | `crate-facade`, `cli-repl`, `modo-efemero` (64) | os 4 usos têm de existir |
| **Config project-bound** | `76` config, `93` exposição-tools, `97` SYSTEM.md, `51` agentes, `52` AGENTS aninhados | uso #4 + M6 |
| **Fiabilidade do loop** | `75`, `96`, `95`, `70`, `68`, `106`, `65`, `67` (G15), `94` (G3) | usável com **qualquer** modelo |
| **Controlo determinístico** | `21` (G7), `42` (G14), `64` (G13→efémero) | M5 |
| **Convenções** | `41` (G1), `20` slash, `84` nome-sessão | uso #4 |
| **Memória** | `114`, `115`, `116` (knudge) | o diferencial |
| **Governança (processo)** | `59`, `60`, `61` | mantém M1 |

### 5.2 **Cogitável** — avaliar com medição (não agora)

`22` compactação (G2) · `9` structured (G5) · `45` codemode (G6) · `73` cache (G4) · `24` contexto
(G11) · `49` sessões-versionadas (G13) · `23` tokens · `43` virtual models · `47` trust · `54`
formatter · `57` simulation · `58` custom tools · `71` prime · `72` retry · `77` layout · `80`
injecção · `87` hooks · `88` checks · `89` diagnóstico · `90` allowlist · `91` egress · `92`
success-checks · `98` keybindings · `100` credenciais · `102` reload · `105` execution-claim ·
`112` anexos · `113` network · `117` task · `119` grafo · `120` lifecycle.

### 5.3 **Descartado** — arquivar (não fazer)

| Grupo | Itens | Razão |
|---|---|---|
| **YAGNI §0** | `12` MCP, `13` GDK, `15` sandbox, `26` ACP, `27` desktop, `28` candle, `29` roaming, `30` scheduler, `31` OAuth, `32` voz, `33` OTel, `34` download, `35` state-machine, `38` distros, `55` PTY, `86` adversário, `101` visão, `113` proxy, `119` grafo, `120` lifecycle | fora do mínimo (já decidido) |
| **Formalismo prematuro** | `1` política (→seam), `2` auditoria, `3` confiança, `4` feedback, `5` diag-catalog, `7` preços, `8` modelo-por-fase, `10` TOON, `17` TUI completa, `18` stats | o katu mediu e rejeitou |
| **Já coberto** | `37`/`39` tool monitor | duplicado |

> Nota: `119` (grafo) e `120` (lifecycle) estavam em "avaliar" na 1.ª revisão; à luz de **M1
> (mínimo)**, passam a **arquivo** — não servem nenhuma das 4 utilizações.

---

## 6. A proposta de backlog integrada

O backlog deixa de ser "uma lista de referenciais" e passa a ser **um backlog do projecto**,
organizado por **eixo de propósito**. Proposta de índice:

```
0. Caminhos (as 4 utilizações)   crate-facade · cli-repl · modo-efemero
1. Fiabilidade do loop           75 · 96 · 95 · 70 · 68 · 106 · 65 · G15(67) · G3(94)
2. Controlo determinístico       G7(21) · G14(42) · [G8(1) seam]
3. Config project-bound          G9(76) · 93 · 97 · 51 · 52
4. Convenções                    G1(41) · 20 · 84
5. Memória                       knudge 114 · 115 · 116
6. Governança (processo)         59 · 60 · 61
7. Cogitável (medir)             §5.2
8. Arquivo                       §5.3 + consolidado (membros fundidos)
```

**Tamanho:** o backlog **activo** (eixos 0–6) tem **~28 itens**; o resto é bibliografia (eixos 7–8).
Isto cumpre a regra de poda: *uma página só é activa se citar o teste/sintoma que a exige*.

---

## 7. Riscos e disciplina

| Risco | Prob. | Impacto | Disciplina |
|---|---|---|---|
| Caminho novo com lógica própria (viola M4) | alta | alto | teste "dois caminhos, uma sessão"; REPL/TUI/crate só falam à porta |
| Config vira prompt-engineering/DSL (viola M1) | média | médio | um ficheiro, campos simples, schema; directriz = texto, não lógica |
| Controlo "sobre o agente" vira segundo kernel (viola M2) | média | alto | reutilizar `Guard::evaluate` + comandos da porta; nada paralelo |
| Config global a sobrepor o projecto (viola M6) | média | alto | precedência projecto>global; teste de isolamento |
| "Mínimo" na retórica, "máximo" no backlog (o erro do katu) | alta | alto | teto de superfície (62) + poda (§5) |

---

## 8. Recomendação

1. **Adoptar o novo filtro (M1–M6)** como critério de admissão de qualquer feature.
2. **Completar os caminhos (Eixo 0), liderado pelo CLI** — `cli-superficie`, `cli-repl`,
   `modo-efemero`, `config-global`, `ktu-init`, `memory-drain-service` (e `crate-facade` para o
   uso #1). Todos **através da porta**, nenhum com lógica própria.
3. **Fechar a config project-bound (Eixo 3)** — um ficheiro (G9) + exposição de tools + directriz.
4. **Fiabilidade do loop (Eixo 1)** — tornar "usável" independente do modelo.
5. **Travar M4 por teste** ("dois caminhos, uma sessão").
6. **Arquivar Eixos 7–8**; manter o activo em ~28 itens.
7. **Governança (Eixo 6)** como processo (ADRs + dossiê), não como feature.

> **A tese, em uma frase.** O ktu já provou que é usável; o próximo erro a evitar é o **simétrico**
> do katu: em vez de construir antes de funcionar, construir **a mais** depois de funcionar. A
> resposta é **completar os caminhos e a config do projecto, travar os invariantes por teste, e
> arquivar o resto** — para que "mínimo" continue a ser uma propriedade medida, não um slogan.
