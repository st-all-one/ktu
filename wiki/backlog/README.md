# Backlog — features não implementadas

> Uma página por feature **deferida** ou **rejeitada** na 2.ª tentativa (`ktu`), com o
> detalhe necessário para uma decisão posterior. Não é um plano de execução: é uma
> **lista de decisões em espera**.
>
> ⚠️ **Antes de usar este índice, leia a [análise crítica](./../backlog_reviews.md)** — ela
> classifica estas 123 páginas em tiers e propõe um backlog activo de ~24 itens (o resto é
> bibliografia).
>
> Regra herdada (`plan.md` §0/§2.12): *cada subsistema só entra com um consumidor
> actual e um teste que o exija*. Cada página diz **o que falta para reconsiderar**.

## Índice

| # | Feature | Estado | Origem principal |
|---|---|---|---|
| 1 | [Motor de política completo (dados)](./motor-de-politica.md) | deferido (seam) | `plan.md` §2.8/§2.12-B |
| 2 | [Auditoria densa (store + bloom + índice)](./auditoria-densa.md) | deferido | katu ADR 0009 |
| 3 | [Confiança estatística (FDR/conformal)](./confianca-estatistica.md) | deferido | katu `confidence/` |
| 4 | [Loop de feedback](./feedback.md) | deferido | katu `feedback.rs` |
| 5 | [Catálogo de diagnóstico extenso (122 eventos)](./diagnostico-extenso.md) | deferido | katu `diag` |
| 6 | [Contexto por rate-distortion / submodular](./contexto-rate-distortion.md) | deferido | katu `context/select` |
| 7 | [Preços/tiers completos](./precos-tiers.md) | deferido | katu `policy/prices.toml` |
| 8 | [Modelo por passo/fase + controlo de thinking](./modelo-por-fase.md) | deferido (seam) | katu ADR 0012 |
| 9 | [Gramática restringida (GBNF) / structured output](./gramatica-restringida.md) | deferido | katu ADR 0025 |
| 10 | [Economia de tokens: TOON/colunar + views + ToolReport](./economia-tokens-toon.md) | deferido | katu ADR 0005/0006/0007 |
| 11 | [Plugins WASM/ABI + dispatch paralelo + paralelismo de tools](./plugins-wasm-paralelo.md) | rejeitado por medição | `decision_01e2183g` |
| 12 | [MCP (Model Context Protocol)](./mcp.md) | fora (YAGNI) | `plan.md` §0 |
| 13 | [Adaptador GDK (goose-sdk)](./gdk.md) | decidido: não implementar | `decision_00aw5etj` |
| 14 | [Providers finos por dialeto (Anthropic, …)](./provider-dialetos.md) | deferido (alternativa ao GDK) | `decision_00aw5etj` |
| 15 | [Sandbox de SO (Landlock/seccomp)](./sandbox-so.md) | fora (YAGNI) | katu `18-jail-futuro` |
| 16 | [Git versionado de sessões/notas](./git-versionado.md) | deferido (seam) | `plan.md` §2.12-B |
| 17 | [TUI completa](./tui-completa.md) | deferido | katu `11-tui-e-ux` |
| 18 | [Stats/report](./stats-report.md) | deferido | katu `stats`/`report` |

## Candidatas vindas do `goose`

Identificadas na [revisão do goose](../revisao-goose.md), com o mesmo filtro
(simplicidade · kernel · usabilidade objectiva).

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 19 | [Receitas reutilizáveis (recipes)](./receitas.md) | candidata | **incorporar** |
| 20 | [Slash commands (atalhos)](./slash-commands.md) | candidata | **incorporar** |
| 21 | [Permission store (aprovações memorizadas)](./permission-store.md) | candidata | **incorporar** |
| 22 | [Resumo estruturado de compactação](./resumo-estruturado.md) | candidata | avaliar (medir Q4) |
| 23 | [Contagem de tokens precisa](./contagem-tokens.md) | candidata | avaliar |
| 24 | [Limite de contexto por modelo](./contexto-por-modelo.md) | candidata | avaliar |
| 25 | [Export de sessão (Markdown)](./export-sessao.md) | candidata | avaliar |

Fora de escopo (não trazer do goose) — cada uma com página própria para decisão:

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 26 | [ACP (Agent Client Protocol)](./acp.md) | candidata | não trazer |
| 27 | [App desktop / UI](./desktop-app.md) | candidata | não trazer |
| 28 | [Inferência local (candle)](./inferencia-local-candle.md) | candidata | não trazer |
| 29 | [Roaming agents / peerbook](./roaming-agents.md) | candidata | não trazer |
| 30 | [Scheduler (tarefas agendadas)](./scheduler.md) | candidata | não trazer |
| 31 | [OAuth / subscrições](./oauth-subscricoes.md) | candidata | não trazer |
| 32 | [Voz / ditado](./voz-dictation.md) | candidata | não trazer |
| 33 | [OpenTelemetry / telemetria](./otel-telemetria.md) | candidata | não trazer |
| 34 | [Download manager (modelos/binários)](./download-manager.md) | candidata | não trazer |
| 35 | [State machine do `goose-agent`](./state-machine-goose.md) | candidata | não trazer |
| 36 | [Code mode / tree-sitter](./code-mode-tree-sitter.md) | candidata | não trazer |
| 37 | [Pesquisa no histórico de sessões](./pesquisa-historico.md) | candidata | não trazer |
| 38 | [Custom distros / deeplinks](./custom-distros.md) | candidata | não trazer |
| 39 | [Tool monitor / inspection](./tool-monitor.md) | candidata | não trazer (já coberto) |

MCP (goose usa 70+ extensões) está coberto pela página [12 — MCP](./mcp.md).

### Revisão profunda do `goose`

Segunda passagem ([revisão profunda](../revisao-goose-profunda.md)), sobre docs, módulos
internos e código além dos candidatos iniciais.

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 79 | [MOIM — memória de trabalho (Turn Context)](./memoria-de-trabalho.md) | candidata | **incorporar** |
| 80 | [Deteção de injecção de prompt](./deteccao-injecao-prompt.md) | candidata | **incorporar** |
| 81 | [Resposta grande → ficheiro](./resposta-grande-ficheiro.md) | candidata | **incorporar** |
| 82 | [Permissões por tool (Always/Ask/Never)](./permissoes-por-tool.md) | candidata | **incorporar** |
| 83 | [Migrações de configuração](./migracoes-config.md) | candidata | **incorporar** |
| 84 | [Nome automático de sessão](./nome-sessao.md) | candidata | **incorporar** |
| 85 | [Editar em sessão / fork](./editar-em-sessao.md) | candidata | **incorporar** |
| 86 | [Modo adversário (revisor independente)](./modo-adversario.md) | candidata | avaliar |
| 87 | [Hooks (eventos de ciclo de vida)](./hooks.md) | candidata | avaliar |
| 88 | [Checks de revisão (`.agents/checks/*.md`)](./checks-review.md) | candidata | avaliar |
| 89 | [Diagnóstico empacotado (`doctor`)](./diagnostico-bundle.md) | candidata | avaliar |
| 90 | [Allowlist (de plugins/comandos)](./allowlist.md) | candidata | avaliar |
| 91 | [Inspector de egress](./inspetor-egress.md) | candidata | avaliar |
| 92 | [Success checks](./success-checks.md) | candidata | avaliar |

Consolidam-se com páginas existentes: **tool shim** → [67 — tool-calls-declaradas](./tool-calls-declaradas.md);
**final output tool** → [74 — saida-estruturada](./saida-estruturada.md); **smart approval** →
[21 — permission-store](./permission-store.md); **code mode** → [36](./code-mode-tree-sitter.md)/[45](./codemode.md);
**summarização de output antigo** → [22 — resumo-estruturado](./resumo-estruturado.md);
**auto-compaction em dois níveis** → [48 — compactacao-parametros](./compactacao-parametros.md).

Fora de escopo (não trazer do goose): MCP, ACP, desktop/GUI, candle, roaming, scheduler, OAuth,
voz, OTel/PostHog, download manager, custom distros/deeplinks, gateway (Telegram/pairing),
`goose_apps` e o state machine do `goose-agent` — ver a secção 4 da revisão profunda.

## Candidatas vindas do `pi`

Identificadas na [revisão do pi](../revisao-pi.md) — o *harness* mais próximo do ktu.

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 40 | [Árvore de sessão (branching/fork/clone)](./session-tree.md) | candidata | avaliar (alto valor) |
| 41 | [Prompt templates (Markdown → `/`)](./prompt-templates.md) | candidata | **incorporar** |
| 42 | [Steering / follow-up messages](./steering-followup.md) | candidata | avaliar |
| 43 | [Virtual models (roteamento)](./virtual-models.md) | candidata | avaliar |
| 44 | [Pi packages (bundlar/distribuir)](./pi-packages.md) | candidata | avaliar |
| 45 | [Codemode (compor tools com código)](./codemode.md) | candidata | avaliar (alto custo) |
| 46 | [Temas (TUI)](./temas-tui.md) | candidata | avaliar (baixo) |
| 47 | [Project trust (confiança no projecto)](./project-trust.md) | candidata | avaliar |
| 48 | [Compactação parametrizada + resumo de branch](./compactacao-parametros.md) | candidata | avaliar |
| 49 | [Sessões versionadas + migração](./sessoes-versionadas.md) | candidata | avaliar |
| 50 | [Chord (contratos/estado de plugins)](./plugin-contracts.md) | candidata | avaliar (avançado) |

Fora de escopo (não trazer do pi): `ai` (27 k, 15+ providers), `durable` (25 k), `mcp`,
`env` (SSH), `tui` (differential rendering), `telemetry`, `evals`, extensões TypeScript
in-process, npm como transporte e share na nuvem — ver a secção 3 da revisão.

### Revisão profunda do `pi`

Segunda passagem ([revisão profunda](../revisao-pi-profunda.md)), sobre `core/`, `utils/`,
`extensions/`, `modes/` e os docs de settings/models/security/json/keybindings.

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 93 | [Exposição de tools (declared/deferred) + `tool_search`](./exposicao-tools.md) | candidata | **incorporar** |
| 94 | [Truncagem sistemática de output (linhas + bytes)](./truncagem-output.md) | candidata | **incorporar** (consolida 81) |
| 95 | [Fila de mutação de ficheiro](./fila-mutacao-ficheiro.md) | candidata | **incorporar** |
| 96 | [Normalização de line-endings/BOM no `edit`](./edit-line-endings.md) | candidata | **incorporar** |
| 97 | [`SYSTEM.md` / `APPEND_SYSTEM.md`](./system-prompt-override.md) | candidata | **incorporar** |
| 98 | [Keybindings configuráveis](./keybindings.md) | candidata | **incorporar** |
| 99 | [`settings.json` com JSON Schema + diagnóstico](./settings-schema.md) | candidata | **incorporar** (c/ 76) |
| 100 | [Precedência de credenciais](./credenciais-precedencia.md) | candidata | **incorporar** |
| 101 | [Visão / imagens](./visao-imagens.md) | candidata | avaliar |
| 102 | [`/reload` (recarregar recursos sem reiniciar)](./reload-recursos.md) | candidata | avaliar |

Consolidam-se com páginas existentes: **cache warming** → [73 — cache-prefixo](./cache-prefixo.md);
**nested tool calls** → [45 — codemode](./codemode.md); **provider attribution** →
[72 — retry-provider](./retry-provider.md); **timings** → [61 — metodo-medicao](./metodo-medicao.md);
**catálogo remoto + overlay offline** → [24 — contexto-por-modelo](./contexto-por-modelo.md);
**export HTML** → [25 — export-sessao](./export-sessao.md); **diagnostics/crash/bug-report** →
[89 — diagnostico-bundle](./diagnostico-bundle.md).

Fora de escopo (não trazer do pi): `ai` (15+ providers), `durable`, `mcp`, `env` (SSH), `tui`
(differential rendering), `telemetry`, `evals`, extensões TypeScript in-process, npm como
transporte e share na nuvem — ver a secção 4 da revisão profunda.

## Candidatas vindas do `opencode`

Identificadas na [revisão do opencode](../revisao-opencode.md).

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 51 | [Agentes declarativos](./agentes-declarativos.md) | candidata | **incorporar** |
| 52 | [AGENTS.md aninhados (descoberta de instruções)](./agents-md-aninhados.md) | candidata | **incorporar** |
| 53 | [Comandos com shell embutido](./comandos-com-shell.md) | candidata | **incorporar** (unificar c/ 19/41) |
| 54 | [Formatter (auto-formatação)](./formatter.md) | candidata | **incorporar** |
| 55 | [PTY persistente](./pty-persistente.md) | candidata | avaliar |
| 56 | [Referências externas](./referencias-externas.md) | candidata | avaliar |
| 57 | [Simulation (testes determinísticos)](./simulation.md) | candidata | avaliar |
| 58 | [Custom tools](./custom-tools.md) | candidata | avaliar |

Consolidam-se com páginas existentes: **permissões declarativas + memorizadas** (§2.5) →
[21 — permission-store](./permission-store.md); **codemode** (§2.8) → [45 — codemode](./codemode.md)
(dois referenciais independentes o implementam).

Fora de escopo (não trazer do opencode): arquitectura Effect-TS, SQLite/`effect-drizzle`,
server multi-instância/`location`/`worktree`, desktop/GUI/web, enterprise/`managed-policy`/SSO,
`ai`/`merman`/`latex`, OAuth complexo e MCP — ver a secção 3 da revisão.

### Revisão profunda do `opencode`

Segunda passagem ([revisão profunda](../revisao-opencode-profunda.md)), sobre `specs/v2/`
(decision records + contratos de sessão/tools), `core/src/session/`, `core/src/{snapshot,
permission,plugin,reference,vcs,worktree}/` e os docs do site.

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 103 | [Snapshots + `/undo` / `/redo`](./snapshots-undo.md) | candidata | **incorporar** |
| 104 | [Admissão durável de prompts (inbox) + `steer`/`queue`](./admissao-inbox.md) | candidata | **incorporar** |
| 105 | [Execution claim — recuperação de crash](./execution-claim.md) | candidata | **incorporar** |
| 106 | [Prompt de `MaxSteps`](./max-steps-prompt.md) | candidata | **incorporar** |
| 107 | [Reparação de input de tool](./tool-input-repair.md) | candidata | **incorporar** (consolida 67) |
| 108 | [Session warming](./session-warming.md) | candidata | avaliar (consolida 73) |
| 109 | [Instruções como deltas de valor](./instrucoes-delta.md) | candidata | avaliar |
| 110 | [Provider policy (declarações ordenadas)](./provider-policy.md) | candidata | avaliar (consolida 1/21) |
| 111 | [Mover sessão](./mover-sessao.md) | candidata | avaliar |
| 112 | [Anexos (`uri` + gamas de linhas)](./anexos.md) | candidata | avaliar (consolida 101) |
| 113 | [Configuração de rede (proxy + CA)](./network-proxy.md) | candidata | avaliar |

Consolidam-se com páginas existentes: **snapshots/undo** → [40](./session-tree.md)/[85](./editar-em-sessao.md);
**inbox/steer** → [42](./steering-followup.md); **execution claim** → [64](./turn-lock.md)/[63](./durabilidade-politica.md);
**event feed com filas independentes** → a porta/`serve`; **plugin hooks por domínio** → [87](./hooks.md);
**compactação rolante + `keep`** → [22](./resumo-estruturado.md)/[48](./compactacao-parametros.md);
**affinity de sessão** → [73](./cache-prefixo.md).

Fora de escopo (não trazer do opencode): Effect-TS, SQLite/`effect-drizzle`, server
multi-instância/`location`/`worktree`/`workspace`, desktop/GUI/web, enterprise/`managed-policy`/SSO,
`ai`/`merman`/`latex`, OAuth complexo, MCP, `vcs` patch, `models-dev` remoto, `console` e `stats`
— ver a secção 4 da revisão profunda.

## Candidatas vindas do `katu` (revisão profunda das ADRs/planos)

Identificadas na [revisão do katu](../revisao-katu.md). O katu **confirma por medição** a
arquitectura do ktu (kernel/superfície, firewall, determinismo, `Model-visible ⟺ logged`,
`StopReason`, estagnação, `pin_last_user`, prefix-stable, torn-tail, retry, cancelamento). As
páginas abaixo são o que **falta** ao ktu — sobretudo **governança** e **robustez de loop**.

**Governança:**

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 59 | [ADRs (decisões com alternativas) + postmortems](./adr-governanca.md) | candidata | **incorporar** |
| 60 | [Dossiê de pendências](./dossie-pendencias.md) | candidata | **incorporar** |
| 61 | [Método de medição (medir → A/B → adotar-ou-reverter)](./metodo-medicao.md) | candidata | **incorporar** |
| 62 | [Teto de superfície](./teto-superficie.md) | candidata | **incorporar** |

**Robustez do loop:**

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 63 | [Política de durabilidade do log (`event`/`turn`)](./durabilidade-politica.md) | candidata | **incorporar** |
| 64 | [Lock de turno por sessão (entre processos)](./turn-lock.md) | candidata | **incorporar** |
| 65 | [Deteção de eco](./deteccao-eco.md) | candidata | **incorporar** |
| 66 | [Cancelamento dentro das tools (kill do grupo)](./cancelamento-tools.md) | candidata | avaliar |

**Modelo / prompt:**

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 67 | [Tool calls declaradas como texto](./tool-calls-declaradas.md) | candidata | **incorporar** |
| 68 | [Recusa acionável (`remedy` por regra)](./recusa-acionavel.md) | candidata | **incorporar** |
| 69 | [Visibilidade das mensagens (`User`/`Agent`)](./visibilidade-mensagens.md) | candidata | **incorporar** |
| 70 | [Continuação de leitura truncada](./read-continuacao.md) | candidata | **incorporar** |
| 71 | [`prime` byte-idêntico por versão](./prime-versionado.md) | candidata | avaliar |

**Provider:**

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 72 | [Retry endurecido do provider](./retry-provider.md) | candidata | **incorporar** |
| 73 | [Cache de prefixo declarativo + afinidade](./cache-prefixo.md) | candidata | avaliar |
| 74 | [Saída estruturada (`json_schema`)](./saida-estruturada.md) | candidata | avaliar (com ressalva) |

**Tools / sessão / config:**

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 75 | [`edit` multi-bloco atómico](./edit-multi-bloco.md) | candidata | **incorporar** |
| 76 | [Configuração global/local e `--params`](./config-precedencia.md) | candidata | avaliar |
| 77 | [Layout central versionado](./layout-versionado.md) | candidata | avaliar |
| 78 | [Snapshot no limite de fase (retomada rápida)](./snapshot-retomada.md) | candidata | avaliar |

Consolidam-se com páginas existentes: **structured output / gramática** (74) →
[9 — gramática-restringida](./gramatica-restringida.md); **cache de prefixo** (73) →
[24 — contexto-por-modelo](./contexto-por-modelo.md).

Fora de escopo (não trazer do katu): motor de política, auditoria densa, MCP, WASM, jail de SO,
dialetos extra, formalismos do Anexo A e `watch_service` — ver a secção 4 da revisão. E **não
repropor** os itens que o katu rejeitou com número (A4, C2, PERT/CPM, PPR, DPP, `batch`
declarativo, runtime programável, partilha estrutural, cache de condensação, `structured_output`
por omissão) — ver a secção 5 da revisão.

## Candidatas vindas do `knudge` (o epicentro de memória)

A instância **actual** (v0.6.1) foi clonada para [`crates/knudge`](../../crates/knudge) e a
migração do `ktu-memory` (que apontava a referência antiga `REF/knudge` 0.5.4) foi **validada**
(uma linha; `cargo check`/`test` verdes). Detalhe em [revisão do knudge](../revisao-knudge.md).

| # | Feature | Estado | Veredicto |
|---|---|---|---|
| 114 | [`Store` + `notas/` + `eventos/` + protocolo `write`](./knudge-store-write.md) | candidata | **incorporar** |
| 115 | [`handoff::rewind` (contexto de sessão com orçamento)](./knudge-rewind.md) | candidata | **incorporar** |
| 116 | [`health::doctor` (saúde da memória)](./knudge-doctor.md) | candidata | **incorporar** |
| 117 | [Hierarquia de trabalho (`epic ⊃ issue ⊃ task`)](./knudge-task.md) | candidata | avaliar |
| 118 | [Como consumir o `knudge-core` (dependência)](./knudge-dependencia.md) | candidata | **decidir** |
| 119 | [Grafo avançado (ontologia, TMS, PageRank/PPR, Louvain)](./knudge-graph.md) | candidata | avaliar |
| 120 | [Ciclo de vida (confiança Beta, shelf-life FSRS, decay, demolição)](./knudge-lifecycle.md) | candidata | avaliar |
| 121 | [Porta `Embedder` (canal semântico)](./knudge-embeddings.md) | candidata | avaliar |
| 122 | [Configuração em dois níveis (global/projeto)](./knudge-config.md) | candidata | avaliar (consolida 76) |
| 123 | [`git::sync` (versionar a memória)](./knudge-git-sync.md) | candidata | avaliar (consolida 16) |

Consolidam-se com páginas existentes: **graph avançado** (119) → [2 — auditoria-densa](./auditoria-densa.md);
**lifecycle** (120) → [6 — contexto-rate-distortion](./contexto-rate-distortion.md); **embeddings**
(121) → [24 — contexto-por-modelo](./contexto-por-modelo.md); **config** (122) →
[76 — config-precedencia](./config-precedencia.md); **git::sync** (123) →
[16 — git-versionado](./git-versionado.md).

Não adoptar: o binário `kd` e o `knudge-mcp` como dependências de runtime (o ktu tem a sua porta;
MCP é YAGNI §0), a CLI/envelope do knudge e o serviço de embeddings (`drain service`).

> **Dois papéis distintos:** a **memória de desenvolvimento do ktu** (`.knudge/` + `kd`, já em uso)
não se confunde com o **motor de memória do agente ktu** (`ktu-memory`, hoje *thin*).

## Como decidir

Cada página tem: **o que é**, **porque não entrou**, **pré-condições para
reconsiderar**, **custo/risco** e **decisão pendente**. Para promover uma feature,
abrir uma issue no knudge (`kd task new --scope issue --parent epic_00ynu6q8`) que
aponte o **teste** que a exige — a regra do §0.
