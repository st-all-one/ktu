# Revisão profunda do `goose` — o que a revisão inicial não pegou

> Segunda passagem pelo `REF/goose` (v1.54.0), agora sobre **docs, módulos internos e
> código** além dos candidatos da [revisão inicial](./revisao-goose.md). O objectivo é
> encontrar o que ficou por colher: features que cabem no ktu (simplicidade · kernel ·
> usabilidade objectiva) e que a primeira passagem **não** registou. Data: 2026-10-17.

---

## 0. O que a revisão inicial já cobriu

Receitas, slash commands, permission store, resumo estruturado, contagem de tokens, limite
de contexto por modelo, export de sessão. Fora: MCP, ACP, desktop, candle, roaming,
scheduler, OAuth, voz, OTel, download manager, state machine, custom distros.

Esta passagem lê o que **ficou de fora dessa lista**: `agents/`, `security/`, `hooks/`,
`checks/`, `session/`, `config/`, `context-engineering`, `managing-tools` e o `goose-agent`.

---

## 1. Candidatos novos a incorporar

### 1.1 MOIM — memória de trabalho (Turn Context) · **INCORPORAR**

**O que é:** o goose injecta, **a cada turno**, um bloco `<turn-context>` na memória de
trabalho do modelo (MOIM = *Model-Observed Internal Memory*): hora actual, directório de
trabalho, estado de compactação, **orçamento de turno** e contexto das extensões. O bloco é
**regenerado por turno** e o modelo é instruído a usá-lo para se orientar sem o confundir
com o pedido. O `system_prompt_block` explica o bloco; o `turn_context_message` produz o
conteúdo. As **instruções persistentes** (`GOOSE_MOIM_MESSAGE_TEXT`/`_FILE`) são injectadas
no mesmo sítio, **re-lidas a cada turno** (tecto 64 KB), pelo que não são "esquecidas".

**Porque interessa ao ktu:** o ktu compõe o sistema a cada turno, mas **não tem** um bloco
de estado sempre presente (hora, cwd, orçamento, modo). É **dado puro**, regenerado do log,
e dá ao modelo consciência situacional. Casa com o `prompt_state` que o katu mediu (Q-04) e
com o `pin_unit` que o ktu já tem.

**Onde entra:** `prompt.rs` (bloco de turno) + `budget.rs` (orçamento). **Esforço:** baixo.
**Risco:** gastar tokens — manter curto e omitir quando o contexto é pequeno (o goose só
liga acima de 32 k).

### 1.2 Deteção de injecção de prompt · **INCORPORAR**

**O que é:** intercepta o **parâmetro** de cada tool call **antes de executar** e compara-o
com **padrões de ameaça** (`patterns.rs`: `ThreatPattern { name, pattern, description,
risk_level, category }`, categorias `FileSystemDestruction`/`RemoteCodeExecution`/
`DataExfiltration`/`SystemModification`/`NetworkAccess`/`ProcessManipulation`/
`PrivilegeEscalation`/`CommandInjection`). Cada risco tem uma **confiança** (Critical 0.95 …
Low); acima do limiar, a execução **pausa** e o utilizador decide (**Allow Once** / **Deny**),
com um *finding ID* logado. Opcional: ML (endpoint de classificação). *"Salvaguarda, não
garantia."*

**Porque interessa ao ktu:** o ktu tem `taint.rs` (marca conteúdo não confiável) — a deteção
de injecção é a **outra metade**: reconhecer padrões perigosos *antes* de executar. É **dado
declarativo** (padrões + limiar), determinístico e barato — encaixa no `Guard`/guardrails.

**Onde entra:** `guardrails.rs` (padrões) + `guard.rs` (limiar + alerta). **Esforço:**
baixo-médio. **Risco:** falsos positivos — limiar configurável e "Allow Once".

### 1.3 Resposta grande → ficheiro · **INCORPORAR**

**O que é:** `large_response_handler.rs` — se o texto de um resultado de tool excede
`GOOSE_MAX_TOOL_RESPONSE_SIZE` (default **200 000** chars), é escrito num **ficheiro
temporário** e o modelo recebe uma **referência** ("a resposta era maior e está no ficheiro
…, usa outras tools para o examinar"). Protege a janela de contexto de um resultado gigante.

**Porque interessa ao ktu:** o ktu tem caps de bash/ficheiro, mas uma tool pode devolver
algo enorme (ex.: `grep` num repo grande). Guardar em ficheiro + referência é **simples** e
evita estourar o orçamento.

**Onde entra:** `ktu-tools` (wrapper de resultado) + porta `Fs`. **Esforço:** baixo.

### 1.4 Permissões por tool (Always/Ask/Never) · **INCORPORAR**

**O que é:** além do **modo** global (`Autonomous`/`Manual`/`Smart`/`Chat Only`), cada tool
tem um nível: **Always Allow** / **Ask Before** / **Never Allow**. O modo é o default; a
permissão por tool **sobrepõe**. (O goose avisa: < 25 tools no total, por desempenho.)

**Porque interessa ao ktu:** o ktu tem guardrails (por caminho/padrão) e `Approval`, mas
**não** uma política declarativa por **tool**. É o complemento natural — **dado** (o nome da
tool → nível), fail-closed, e mapeia directamente nos modos de modo plano/execução.

**Onde entra:** `.agents/guardrails/` (regra por tool) + `guard.rs`. **Esforço:** baixo.
**Risco:** alargar privilégio — "Never Allow" por omissão para tools sensíveis.

### 1.5 Migrações de configuração · **INCORPORAR**

**O que é:** `config/migrations.rs` — migra o esquema da config (ex.: `extensions`/`providers`)
com uma função `run_migrations(&mut Mapping) -> bool`; há ainda `run_read_migrations` **não
destrutivo** para caminhos de leitura em memória.

**Porque interessa ao ktu:** se o ktu adotar config (global/local, backlog 76), o esquema vai
mudar. Uma migração explícita, idempotente e **separada** da leitura evita corromper configs
antigas. É simples e evita a dor futura.

**Onde entra:** `ktu-cli` (loader de config). **Esforço:** baixo.

### 1.6 Nome automático de sessão · **INCORPORAR**

**O que é:** `session_naming.rs` — após `MSG_COUNT_FOR_SESSION_NAME_GENERATION = 3` mensagens,
gera um **título curto** para a sessão (limpa tags XML, extrai um título de ≤ 8 palavras).

**Porque interessa ao ktu:** listar sessões por `id` é pouco legível; um nome curto gerado do
conteúdo melhora a usabilidade da TUI/`serve`. Simples e determinístico (não precisa de LLM
se usar heurística; o goose usa o modelo).

**Onde entra:** `session_store.rs` + TUI. **Esforço:** baixo.

### 1.7 Editar em sessão / fork · **INCORPORAR**

**O que é:** `in-session-actions` — editar uma mensagem anterior e **truncar** tudo o que
vem depois (*Edit in Place*), ou **forkar** a sessão preservando a história até à mensagem
editada (*Fork Session*). Na CLI: `goose session --resume <id> --edit` abre a conversa em
YAML no `$EDITOR`.

**Porque interessa ao ktu:** corrigir uma mensagem sem recomeçar é usabilidade objectiva; o
**fork** é a versão simples da árvore de sessão (pi, backlog 40). O log append-only do ktu
suporta um fork por prefixo sem reescrever história.

**Onde entra:** `session_store.rs` + CLI/TUI. **Esforço:** médio. **Risco:** truncar contexto
— explícito e reversível por fork.

### 1.8 Modo adversário (revisor independente) · **AVALIAR**

**O que é:** um **revisor silencioso** que avalia cada tool call contra regras em linguagem
natural (`~/.config/goose/adversary.md`), devolvendo **ALLOW**/**BLOCK**; bloqueado não
repete; se o revisor falha, **fail-open**. Usa o mesmo modelo (sem chave nova).

**Porque interessa ao ktu:** é um "segundo par de olhos" contextual que os guardrails
determinísticos não dão. **Mas:** é **um LLM por tool call** (custo, não-determinismo) e
**fail-open** — contra o princípio fail-closed do ktu. **Veredicto:** só como opt-in, com
consciência do custo; o ktu prefere determinismo.

### 1.9 Hooks (eventos de ciclo de vida) · **AVALIAR**

**O que é:** scripts do utilizador em eventos (`SessionEnd`, edição de ficheiro, comando de
shell), descobertos de **plugins em disco** (`<plugin>/hooks/hooks.json`), com o payload em
JSON no stdin. Segue a *Open Plugins hooks specification*.

**Porque interessa ao ktu:** o ktu tem plugins (que transformam/negam) mas **não** hooks de
notificação/ciclo de vida (ex.: formatar após edição, notificar no fim). É uma forma leve de
extensibilidade. **Risco:** executa comandos locais — só de fontes confiáveis.

### 1.10 Checks de revisão (`.agents/checks/*.md`) · **AVALIAR**

**O que é:** `checks/mod.rs` — descobre `.agents/checks/*.md` e `**/.agents/REVIEW.md` (com
frontmatter `name`/`description`/`model`/`turn-limit`/`tools`/`severity-default`); cada check
corre como um **subagente** e emite achados atribuídos ao ficheiro de origem.

**Porque interessa ao ktu:** é uma forma **declarativa** de definir verificações (lint/review)
que correm como subagentes — reutiliza o fan-out existente. **Esforço:** médio.

### 1.11 Diagnóstico empacotado (`doctor`) · **AVALIAR**

**O que é:** `session/diagnostics.rs` + `doctor.rs` — recolhe sistema (versão/OS/arquitetura),
provider, config, tail de logs (com caps: `LLM_LOG_MAX_BYTES` 2 MB, `CONFIG_MAX_BYTES`
256 KB, `CLI_LOG_TAIL_LINES` 400), em `Summary`/`Full`.

**Porque interessa ao ktu:** o ktu tem `ktu doctor`? Um **pacote** de diagnóstico (com caps)
para reportar bugs é usabilidade objectiva e barata.

### 1.12 Allowlist (de extensões/comandos) · **AVALIAR**

**O que é:** `guides/allowlist.md` — uma allowlist YAML (`id` + `command`) buscada de um URL
(`GOOSE_ALLOWLIST`), cacheada; a instalação de uma extensão fora da lista é **recusada**.

**Porque interessa ao ktu:** o ktu instala plugins; uma allowlist (ou lista de comandos
permitidos) é uma trave de segurança simples para ambientes geridos. **Esforço:** baixo.

### 1.13 Inspector de egress · **AVALIAR**

**O que é:** `security/egress_inspector.rs` — monitoriza/inspeta o tráfego de **saída**
(exfiltração de dados), a par do `adversary_inspector` e do `scanner`.

**Porque interessa ao ktu:** o ktu confina o `bash` e marca taint; inspeccionar o que **sai**
(URLs, pipes para serviços externos) é a defesa complementar. **Esforço:** médio.

### 1.14 Success checks · **AVALIAR**

**O que é:** `agents/retry.rs` + `SuccessCheck::Shell` — após uma tarefa/run, corre comandos
de verificação (`shell`) com **timeout**; falha → não sucesso. Um "gate" declarativo do
resultado.

**Porque interessa ao ktu:** dá ao utilizador uma forma de exigir "só conta como feito se
`make check` passar" — **dado** declarativo, alinhado com a cultura de gates do ktu.

---

## 2. Consolidações (não criar páginas novas)

| Feature do goose | Onde consolidar |
|---|---|
| **Tool shim** (tool calls em texto → estruturadas, com modelo interpretador) | [67 — tool-calls-declaradas](./backlog/tool-calls-declaradas.md) (katu): mesma classe |
| **Final output tool** (`recipe__final_output`, resposta estruturada) | [74 — saida-estruturada](./backlog/saida-estruturada.md) (katu) |
| **Smart approval** (auto-aprovar baixo risco) | [21 — permission-store](./backlog/permission-store.md) |
| **Code mode** (código compõe tools) | [36 — code-mode-tree-sitter](./backlog/code-mode-tree-sitter.md) e [45 — codemode](./backlog/codemode.md) |
| **Summarização de output de tool antigo** (`GOOSE_TOOL_CALL_CUTOFF`) | [22 — resumo-estruturado](./backlog/resumo-estruturado.md) |
| **Auto-compaction em dois níveis** (gatilho + estratégia de recurso) | [48 — compactacao-parametros](./backlog/compactacao-parametros.md) |
| **Busca no histórico de sessões** | [37 — pesquisa-historico](./backlog/pesquisa-historico.md) (reavaliar com o novo peso) |
| **`tool_inspection` / `tool_monitor`** | [39 — tool-monitor](./backlog/tool-monitor.md) |
| **MCP elicitation** (pedir input estruturado ao utilizador) | liga a [12 — MCP](./backlog/mcp.md) e à approval |

---

## 3. Confirmações (o ktu já acerta)

| Decisão do goose | O que o ktu já tem |
|---|---|
| Erros do modelo (tool desconhecida/params errados) voltam como **tool result** com instruções de recuperação | o ktu devolve erros ao modelo como resultado |
| Subagentes com isolamento de contexto + fan-out | `subagent.rs` (explorer + fan-out, profundidade 1) |
| Hints aninhados (`AGENTS.md`/`.goosehints` por nível) | `AGENTS.md` (o ktu compõe) |
| Skills com frontmatter + carregamento por demanda | `.agents/skills/` |
| Permissões por modo + store | guardrails + `Approval` |
| Erros estruturados (`thiserror`) com taxonomia | `error.rs` (`ErrorKind` + `exit_code`) |
| Retry com backoff | `llama.rs` |

---

## 4. Não trazer (reafirma a revisão inicial)

MCP (70+ extensões), ACP server/providers, desktop/GUI, candle (inferência local), roaming,
scheduler, OAuth/15+ providers, voz/dictation, OTel/PostHog, download manager, custom
distros/deeplinks, gateway (Telegram/pairing), `goose_apps`, o **state machine** do
`goose-agent` (o ktu já tem o loop) e o crate `goose-context-management` (adoptar só a
**ideia** do resumo estruturado, já na página 22).

---

## 5. Recomendação

**Incorporar já (barato, alto valor):**
1. **MOIM / memória de trabalho** (§1.1) — estado situacional por turno.
2. **Deteção de injecção de prompt** (§1.2) — a outra metade do `taint`.
3. **Resposta grande → ficheiro** (§1.3).
4. **Permissões por tool** (§1.4) — dado declarativo.
5. **Migrações de config** (§1.5) e **nome de sessão** (§1.6) — baratos.
6. **Editar em sessão / fork** (§1.7) — usabilidade forte.

**Avaliar:**
7. **Modo adversário** (§1.8, opt-in, custo/não-determinismo), **hooks** (§1.9),
   **checks de revisão** (§1.10), **diagnóstico empacotado** (§1.11), **allowlist** (§1.12),
   **inspector de egress** (§1.13), **success checks** (§1.14).

**Manter fora:** a lista do §4.

> **Nota.** O goose dá **padrões de segurança** (injecção/egress/allowlist) e **consciência
> situacional** (MOIM) que o ktu ainda não tem — e ambos são **dados**, não subsistemas. É
> o complemento natural ao que o katu ensinou sobre governança.
