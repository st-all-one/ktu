# Revisão profunda do `opencode` — o que a revisão inicial não pegou

> Segunda passagem pelo `REF/opencode` (monorepo Bun/TS v2), agora sobre **`specs/v2/`,
> `packages/core/src/`, os docs do site e os plugins internos** além dos candidatos da
> [revisão inicial](./revisao-opencode.md). Data: 2026-10-17.

---

## 0. O que a revisão inicial já cobriu

Agentes declarativos, AGENTS.md aninhados, comandos com shell, formatter, PTY persistente,
referências externas, simulation, custom tools, temas; consolidações com `permission-store` (21)
e `codemode` (45). Fora: Effect-TS, SQLite, server multi-instância, desktop/GUI/web, enterprise,
`ai`/`merman`/`latex`, OAuth complexo, MCP.

Esta passagem lê o que ficou de fora: **`specs/v2/`** (as *decision records* + contratos de sessão
e tools), `core/src/session/`, `core/src/{snapshot,permission,plugin,reference,vcs,worktree}/`,
`services/www/.../docs/content/*.mdx` e os plugins internos (`warming`, `verbosity`,
`tool-input-repair`).

---

## 1. Candidatos novos a incorporar

### 1.1 Snapshots + `/undo` / `/redo` · **INCORPORAR**

**O que é:** por cada passo do modelo, o opencode tira um **snapshot** imediatamente **antes** da
chamada e outro no sucesso/falha; a mensagem do assistente registra os **caminhos alterados**
entre os dois. O `/undo` **encena** um rollback (restaura os ficheiros **e** devolve o prompt ao
composer), o `/redo` cancela; submeter o prompt editado **compromete** o rollback. Também se
reverte para uma mensagem anterior. A captura é *best-effort*; o escopo é o directório activo
dentro de um worktree Git.

**Porque interessa ao ktu:** o ktu tem `trash`/`restore` e checkpoint, mas **não** um
undo/redo **de conversa + ficheiros** encenado. É usabilidade objectiva e encaixa no log (a
reversão é uma operação registada). Casa com a árvore de sessão ([40](./backlog/session-tree.md))
e o `edit`/`fork` ([85](./backlog/editar-em-sessao.md)).

**Onde entra:** `session/mod.rs` (stage/commit/clear) + `ktu-tools` (snapshot de ficheiros).
**Esforço:** médio. **Risco:** tocar em ficheiros — snapshot *best-effort*, nunca bloquear.

### 1.2 Inbox — admissão durável de prompts + `steer`/`queue` · **INCORPORAR**

**O que é:** `Session.prompt(...)` publica um **facto durável** `session.inbox.enqueued` cuja
projecção insere uma linha `session_inbox` **antes** de começar a execução. Um item fica **fora
do histórico model-visible** até ser entregue; a projecção `session.inbox.delivered` consome a
linha e insere a mensagem visível **atomicamente**. Modos de entrega: **`steer`** (default,
entregue no próximo *Safe Step Boundary*) e **`queue`** (fica pendente enquanto a sessão pode
continuar). Admissão **idempotente** (reusar um ID: o primeiro vence). Compactação e *move* são
**itens de controlo** com identidade própria.

**Porque interessa ao ktu:** o ktu tem *steering* (pi, [42](./backlog/steering-followup.md)) e
uma porta com `Command`, mas **não** um modelo de **admissão durável** com entrega em fronteiras
seguras e idempotência. É **kernel** (o log é a fonte) e robustez (uma mensagem nunca se perde a
meio do turno).

**Onde entra:** `session/mod.rs` + `ktu-port` (fila). **Esforço:** médio-alto. **Risco:** tocar
no log — preservar `Model-visible ⟺ logged`.

### 1.3 Execution claim — recuperação de crash · **INCORPORAR**

**O que é:** quando começa um período *busy* do processo, a execução **compromete um claim
write-ahead**; sucesso/falha/interrupção libertam-no, mas *shutdown* sujo **preserva-o**. No
arranque, sessões de topo com claim são **retomadas** do histórico durável, com uma **instrução
de continuação** e **contagem limitada de tentativas**; as tool calls pendentes são **falhadas**
(*orphan reconciliation*) antes de continuar. **Não** garante exactly-once de provider/tools.

**Porque interessa ao ktu:** o ktu tem retomada e `reconcile_pending`? Um **claim** que sobrevive
a um kill e limita as tentativas de recuperação é robustez real — evita retomar em ciclo infinito
ou deixar tool calls pendentes.

**Onde entra:** `session/mod.rs` + `store.rs`. **Esforço:** médio. **Risco:** claim obsoleto —
idade/limite de tentativas.

### 1.4 Prompt de `MaxSteps` · **INCORPORAR**

**O que é:** ao atingir o máximo de passos, o runner injecta um prompt (`MAX_STEPS_PROMPT`) que
**desactiva as tools** e obriga o modelo a responder **só com texto**, com um resumo do que foi
feito, tarefas restantes e recomendações.

**Porque interessa ao ktu:** o ktu termina com `StopReason::MaxSteps` **sem** dar ao modelo a
chance de resumir. Injectar um passo final textual transforma um corte abrupto num desfecho útil.

**Onde entra:** `agent/mod.rs`. **Esforço:** baixo. **Risco:** baixo.

### 1.5 Reparação de input de tool · **INCORPORAR** (consolidar com 67)

**O que é:** `plugin/tool-input-repair.ts` — repara o input de tool calls quando o schema o
suporta **inequivocamente**: objecto em string (`'{"limit":"20"}'`), string numérica/booleana
(`"20"`→`20`), `null`/`{}` para opcional, arrays em string, tuplos posicionais, dicionários,
campos aninhados. Profundidade máx. 6; **só** quando é inequívoco.

**Porque interessa ao ktu:** modelos fracos emitem argumentos em string; a reparação (limitada e
verificada pelo schema) salva turnos que hoje falham em `Decode`. É a complementaridade da
declaração de tool calls em texto ([67](./backlog/tool-calls-declaradas.md)).

**Onde entra:** `agent`/`wire`. **Esforço:** baixo-médio. **Risco:** reparar errado — só quando
inequívoco; caso contrário, falha.

### 1.6 Session warming · **AVALIAR** (consolidar com 73)

**O que é:** `warming.mdx` + `plugin/warming.ts` — envia pedidos *keep-alive* periódicos para
preservar as **caches de prefixo** do provider durante pausas; **desligado por omissão**;
configurável (`prompt`/`interval`/`duration`). Aquece uma sessão após 4 min sem pedido, a cada
4 min, até 30 min desde o último pedido real.

**Porque interessa ao ktu:** complementa a cache de prefixo declarativa
([73](./backlog/cache-prefixo.md)) e o cache warming do pi: não só declarar, mas **manter
quente**. Avançado (custo/TTL); consolidar.

### 1.7 Instruções como deltas de valor · **AVALIAR**

**O que é:** a sincronização de instruções persiste valores **content-addressed** (SHA-256);
`session.instructions.updated { delta, text? }` mapeia cada fonte alterada a um hash, com
`"removed"` para ausência; os corpos vivem uma vez num store `instruction_blob`; a projecção
`instruction_state` fornece os valores correntes e iniciais; uma **epoch** de instruções
atravessa compactações.

**Porque interessa ao ktu:** o ktu recompõe o prompt a cada turno. Persistir instruções como
**deltas** (com epoch) evita reprocessar tudo e dá rasto de **o que mudou**. Avançado.

### 1.8 Resposta de tool com três valores (`output`/`content`/`metadata`) · **AVALIAR** (consolidar 45)

**O que é:** uma resposta de tool pode trazer o `output` **validado por schema** (valor de
máquina efémero que o Code Mode recebe), o `content` **model-facing durável** e `metadata`
(JSON compacto para UI). Uma tool sem `output` devolve só `content`/`metadata`.

**Porque interessa ao ktu:** separa **o que o modelo vê** do **que o código vê** — útil para o
codemode (filtrar antes de mostrar). Consolidar em [45](./backlog/codemode.md).

### 1.9 Provider policy · **AVALIAR** (consolidar com 1/21)

**O que é:** `policies.mdx` + `provider-policy.md` — declarações
`{effect: allow|deny, action: provider.use|tool.use|integration.use, resource}`; wildcard;
**última correspondência vence**; utilizador > repositório > organização. `tool.use` **nunca**
concede acesso (as permissões decidem `allow`/`ask`).

**Porque interessa ao ktu:** o ktu exclui um *policy engine* (§0), mas tem guardrails. A ideia
"declaração ordenada + wildcard + última vence" é **dado** simples; consolidar com
[1 — motor-de-politica](./backlog/motor-de-politica.md) e [21 — permission-store](./backlog/permission-store.md).

### 1.10 Mover sessão · **AVALIAR**

**O que é:** `session/move.ts` — move uma sessão para outro directório/location, re-resolvendo os
serviços de *placement*; os eventos da sessão ficam fora do contexto da location para que o
movimento não os fixe à antiga.

**Porque interessa ao ktu:** o ktu associa uma sessão a um `--root`. Mover uma sessão entre
projectos é usabilidade avançada; baixo valor hoje.

### 1.11 Anexos (`uri` + linhas) · **AVALIAR** (consolidar 101)

**O que é:** `attachments.mdx` — um anexo é `{uri, name?, description?}`; URLs `file://` absolutos;
`start`/`end` (1-based) para gamas de linhas; imagens/PDF passados ao modelo (até 20 MiB).

**Porque interessa ao ktu:** formaliza o anexo (ficheiro/gama de linhas) como **dado**. Consolidar
com [101 — visao-imagens](./backlog/visao-imagens.md) e a leitura por `offset`/`limit`.

### 1.12 Configuração de rede (proxy + CA) · **AVALIAR**

**O que é:** `network.mdx` — `HTTP_PROXY`/`HTTPS_PROXY`/`NO_PROXY` (loopback **excluído** por ser
usado entre CLI e serviço) + certificados personalizados.

**Porque interessa ao ktu:** o ktu fala HTTP com o provider; suportar proxy/CA é necessário em
redes corporativas. Baixo esforço; documentar a exclusão do loopback.

---

## 2. Consolidações (não criar páginas novas)

| Feature do opencode | Onde consolidar |
|---|---|
| **Snapshots/undo** (§1.1) | [40 — session-tree](./backlog/session-tree.md) + [85 — editar-em-sessao](./backlog/editar-em-sessao.md) |
| **Inbox/steer/queue** (§1.2) | [42 — steering-followup](./backlog/steering-followup.md) |
| **Execution claim** (§1.3) | [64 — turn-lock](./backlog/turn-lock.md) + [63 — durabilidade-politica](./backlog/durabilidade-politica.md) |
| **Tool input repair** (§1.5) | [67 — tool-calls-declaradas](./backlog/tool-calls-declaradas.md) |
| **Warming** (§1.6) | [73 — cache-prefixo](./backlog/cache-prefixo.md) |
| **Provider policy** (§1.9) | [1 — motor-de-politica](./backlog/motor-de-politica.md), [21 — permission-store](./backlog/permission-store.md) |
| **Anexos** (§1.11) | [101 — visao-imagens](./backlog/visao-imagens.md) |
| **Event feed com filas independentes** (`event-stream-architecture.md`) | a porta/`serve` do ktu |
| **Plugin hooks por domínio** (`plugin/hooks.ts`) | [87 — hooks](./backlog/hooks.md) |
| **Compactação: resumo rolante + `keep`** (`compaction.mdx`) | [22 — resumo-estruturado](./backlog/resumo-estruturado.md), [48 — compactacao-parametros](./backlog/compactacao-parametros.md) |
| **Affinity de sessão** (`session/affinity.ts`) | [73 — cache-prefixo](./backlog/cache-prefixo.md) |
| **Formatter** | já na inicial (54) |
| **MCP / SQLite / Effect-TS / desktop / enterprise** | fora (YAGNI) |

---

## 3. Confirmações (o ktu já acerta)

| Decisão do opencode | O que o ktu já tem |
|---|---|
| Tool call tem **identidade durável** (`messageID` + `callID`) | ids de evento estáveis |
| Cada tool call **durável antes** dos efeitos; outcome terminal serializado | ordem §42 / I2 |
| Orquestração **nunca** delegada a um loop in-memory; continua em novo *Step* | `derive_messages` do log |
| Tool calls pendentes **falhadas** no arranque (*orphan reconciliation*) | `reconcile_pending`? |
| Instruções: `AGENTS.md` global + do workspace **para a home**, combinados sem resolver conflitos | `AGENTS.md` composto |
| Eventos duráveis com **replay** + watermark de sincronização | log JSONL + `serve` |
| `provider.use`/`tool.use` **nunca** concede acesso | guardrails fail-closed |
| Retry com backoff exponencial + jitter, honrando `Retry-After` | `llama.rs` |
| Formatter só corre quando o executável existe | (backlog 54) |

---

## 4. Não trazer (reafirma a revisão inicial)

Effect-TS, SQLite/`effect-drizzle`, server multi-instância/`location`/`worktree`/`workspace`,
desktop/GUI/web, enterprise/`managed-policy`/SSO, `ai`/`merman`/`latex`, OAuth complexo, MCP,
`vcs` patch, `models-dev` remoto, `console`, `stats`.

---

## 5. Recomendação

**Incorporar já (barato, alto valor):**
1. **Prompt de `MaxSteps`** (§1.4) — desfecho útil em vez de corte abrupto.
2. **Reparação de input de tool** (§1.5) — salva turnos com modelos fracos.
3. **Snapshots + `/undo`/`/redo`** (§1.1) — undo de conversa **e** ficheiros.
4. **Execution claim** (§1.3) — recuperação de crash com tentativas limitadas.

**Avaliar:**
5. **Inbox/steer/queue** (§1.2, médio-alto), **instruções-delta** (§1.7), **warming** (§1.6),
   **provider policy** (§1.9), **mover sessão** (§1.10), **anexos** (§1.11), **rede** (§1.12).

**Manter fora:** a lista do §4.

> **Nota de fecho.** O opencode confirma, mais uma vez, que o ktu acertou na arquitectura (log
> durável, tool calls com identidade, orquestração fora do loop in-memory). O que ele acrescenta
> é **undo/redo**, **admissão durável de prompts** e **recuperação de crash** — robustez de
> sessão, que nenhum dos outros referenciais tinha.
