# Revisão profunda do `pi` — o que a revisão inicial não pegou

> Segunda passagem pelo `REF/pi` (monorepo TS, v1.0.x), agora sobre **docs, `core/`, `utils/`,
> `extensions/` e `modes/`** além dos candidatos da [revisão inicial](./revisao-pi.md). O pi é o
> **análogo mais próximo** do ktu (harness mínimo) — a comparação mais directa. Data: 2026-10-17.

---

## 0. O que a revisão inicial já cobriu

Árvore de sessão, prompt templates, steering/follow-up, virtual models, pi packages, codemode,
temas, project trust, compactação parametrizada, sessões versionadas, chord. Fora: `ai`,
`durable`, `mcp`, `env`, `tui` (differential rendering), `telemetry`, `evals`, extensões TS
in-process, npm, share na nuvem.

Esta passagem lê o que ficou de fora dessa lista: `core/` (tools, compaction, trust, settings,
model-registry, cache), `utils/` (imagens, output-files, fs-watch), `extensions/tool-search`,
`modes/` e os docs `settings`/`models`/`security`/`json`/`keybindings`/`configuration`.

---

## 1. Candidatos novos a incorporar

### 1.1 Exposição de tools (declared/deferred) + `tool_search` · **INCORPORAR**

**O que é:** cada tool tem uma **exposição**: `declared` (vai ao modelo), `deferred` (não vai) ou
via `codemode`. O `tool_search` é um **ranker BM25** sobre os metadados das tools; devolve os
`top-8` e **carrega** os que faltam, ficando declarados no **próximo** pedido — registado no
transcript (sobrevive a `/tree`, resume e fork). O `codemode` mostra só as tools não declaradas.

**Porque interessa ao ktu:** o ktu declara **todas** as tools no catálogo. Com muitas tools, o
catálogo gasta tokens e confunde o modelo. A exposição por **dado** (declared/deferred) +
`tool_search` (BM25 determinístico) mantém o catálogo pequeno sem perder tools. **Distinto** do
que o katu rejeitou (A4, retrieval semântico de tools, 14 % < 20 %): aqui o ganho é **não declarar**
até ser preciso, não escolher por semelhança.

**Onde entra:** `ktu-core/src/tools.rs` + prompt. **Esforço:** médio. **Risco:** o modelo não
descobre a tool — `tool_search` sempre disponível + nomes claros.

### 1.2 Truncagem sistemática de output (linhas + bytes, output completo em ficheiro) · **INCORPORAR**

**O que é:** `truncate.ts` — dois limites **independentes**, o que vier primeiro: **2000 linhas**
e **50 KB**; nunca devolve linhas parciais (excepto o *tail* do bash). O `output-accumulator.ts`
acumula o output, trunca e guarda o **output completo** num ficheiro com caminho devolvido ao
modelo. O `grep` limita cada linha a 500 chars.

**Porque interessa ao ktu:** é a política de truncagem **canónica** — mais completa que caps
soltos. Consolida com [81 — resposta-grande-ficheiro](./backlog/resposta-grande-ficheiro.md)
(goose): a ideia é a mesma; a do pi acrescenta o **duplo limite** e o **caminho do ficheiro**.

**Onde entra:** `ktu-tools` (wrapper de resultado). **Esforço:** baixo.

### 1.3 Fila de mutação de ficheiro · **INCORPORAR**

**O que é:** `file-mutation-queue.ts` — uma fila **por caminho** (chave = `realpath`, com fallback
ao `resolve` quando o ficheiro não existe) serializa as mutações ao mesmo ficheiro. Evita
escritas concorrentes a partir de tool calls paralelas.

**Porque interessa ao ktu:** o ktu tem `write`/`edit` e pode correr tool calls em paralelo (lote
`Shared`). Sem serialização, dois `edit` ao mesmo ficheiro perdem dados. É **kernel** (uma fila
determinística por caminho).

**Onde entra:** `ktu-tools/src/fs_tools.rs`. **Esforço:** baixo-médio.

### 1.4 Normalização de line-endings/BOM no `edit` · **INCORPORAR**

**O que é:** `edit-diff.ts` — detecta o **line-ending** (`\r\n`/`\n`), normaliza para LF para
calcular o diff e **restaura** o ending original; `splitBom` trata o BOM.

**Porque interessa ao ktu:** editar um ficheiro CRLF num ambiente Unix (ou vice-versa) corrompe o
ficheiro inteiro num diff. Preservar o ending é correcção, não feature.

**Onde entra:** `ktu-tools/src/fs_tools.rs`. **Esforço:** baixo. **Risco:** baixo.

### 1.5 `SYSTEM.md` / `APPEND_SYSTEM.md` · **INCORPORAR**

**O que é:** o prompt de sistema pode ser **substituído** (`SYSTEM.md`) ou **estendido**
(`APPEND_SYSTEM.md`), a nível **utilizador** (`<agent-dir>/`) e **projecto** (`.pi/`); o ficheiro
de projecto **tem precedência** (não se combinam homónimos).

**Porque interessa ao ktu:** o ktu compõe o sistema; não deixa o utilizador substituí-lo nem
acrescentar-lhe instruções persistentes. É **dado** (dois ficheiros) e dá controlo sem tocar no
código.

**Onde entra:** `prompt.rs`. **Esforço:** baixo. **Risco:** prompt maior — caps.

### 1.6 Keybindings configuráveis · **INCORPORAR**

**O que é:** acções **nomeadas** (`app.session.new`) mapeadas para teclas num
`keybindings.json` (uma ou lista; lista vazia desliga); `/hotkeys` mostra os activos; `/reload`
aplica.

**Porque interessa ao ktu:** a TUI tem teclas fixas. Acções nomeadas + config é usabilidade
objectiva e **dado**. Baixo esforço.

**Onde entra:** `ktu-cli/src/tui.rs`. **Esforço:** baixo.

### 1.7 `settings.json` com JSON Schema + diagnóstico · **INCORPORAR**

**O que é:** `settings-schema.ts` + `settings-diagnostics.ts` — a config tem um **JSON Schema**
(autocompletação no editor) e um diagnóstico que valida chaves/valores; `settings.json` suporta
`"$schema"`.

**Porque interessa ao ktu:** se o ktu adotar config ([76](./backlog/config-precedencia.md)), um
schema + diagnóstico tornam a config **auto-documentada** e validada. Alinha com o `deny_unknown_fields`
do katu.

**Onde entra:** `ktu-cli` (config). **Esforço:** baixo-médio.

### 1.8 Precedência de credenciais · **INCORPORAR**

**O que é:** o pi resolve credenciais numa ordem explícita: `--api-key` **>** `auth.json`
guardado **>** `apiKey` do `models.json` **>** variáveis de ambiente / credenciais de cloud
ambiente. Extensões podem definir o seu próprio comportamento.

**Porque interessa ao ktu:** o ktu lê a chave do ambiente. Uma **ordem** explícita (runtime >
guardada > config > env) evita ambiguidade e permite CI sem escrever credenciais. É regra, não
subsistema.

**Onde entra:** `ktu-cli`/`ktu-provider` (borda). **Esforço:** baixo.

### 1.9 Visão / imagens · **AVALIAR**

**O que é:** `utils/image-*.ts` (resize, convert, exif-orientation), `clipboard-image.ts`,
`tool-result-images.ts`; mensagens com `ImageContent` (base64 + mime).

**Porque interessa ao ktu:** o ktu só lida com texto. Aceitar imagens (colar/anexar) e devolver
imagens de tools é usabilidade objectiva — mas exige um caminho de bytes no wire e no log.

**Onde entra:** `domain.rs` (content blocks) + `wire.rs`. **Esforço:** médio. **Risco:** log
maior — guardar referência, não bytes.

### 1.10 `/reload` (recarregar recursos sem reiniciar) · **AVALIAR**

**O que é:** `/reload` recarrega settings, keybindings, instruções, extensões, skills e
templates **sem reiniciar** a sessão; a extensão recarregada não pode reusar estado do runtime
antigo.

**Porque interessa ao ktu:** o ktu tem hot reload de **plugins** por geração; estender o conceito
a recursos (skills/guardrails/prompt) é usabilidade objectiva.

**Onde entra:** `ktu-cli` (reload). **Esforço:** baixo-médio.

### 1.11 Prompt cache warming · **AVALIAR** (consolidar com 73)

**O que é:** `cache-warmer.ts` + `cache-stats.ts` — mantém a **cache de prefixo** do provider
quente: refresca a **90 %** do TTL, só quando a poupança esperada ≥ **$0,05**; mede *cache misses*
(idle > 5 min, mudança de modelo) e publica avisos. O *refresh* conta para o uso mas **não** entra
no contexto do modelo.

**Porque interessa ao ktu:** é o complemento da cache de prefixo declarativa
([73](./backlog/cache-prefixo.md)): não só declarar, mas **manter quente**. Avançado (custo/TTL);
consolidar em 73.

### 1.12 Tool calls aninhadas · **AVALIAR** (consolidar com 45)

**O que é:** `nested-tool-calls.ts` — uma tool pode chamar `ctx.executeTool()`, que corre a tool
pelo **pipeline** (com hooks), emite eventos com `parentToolCallId` e regista as chamadas e o uso
no resultado da tool-mãe. Limites: 256 chamadas, 8 KB/call, 32 KB total.

**Porque interessa ao ktu:** é a infra-estrutura que o **codemode** precisa para compor tools.
Consolidar em [45 — codemode](./backlog/codemode.md); sem codemode, é YAGNI.

### 1.13 Atribuição de provider · **AVALIAR**

**O que é:** `provider-attribution.ts` — reconhece o host do endpoint (OpenRouter, NVIDIA NIM,
Cloudflare, opencode) e ajusta cabeçalhos/telemetria.

**Porque interessa ao ktu:** saber **qual** provider serviu um pedido é evidência útil (o ktu
publica números com `basis`). Baixo esforço; consolidar com [72 — retry-provider](./backlog/retry-provider.md).

### 1.14 Timings / instrumentação de fases · **AVALIAR**

**O que é:** `timings.ts` — mede fases (provider, tools, compaction) por turno.

**Porque interessa ao ktu:** o ktu mede por benchmark; timings por turno em produção (diag)
fecham o ciclo. Consolida com [61 — metodo-medicao](./backlog/metodo-medicao.md).

---

## 2. Consolidações (não criar páginas novas)

| Feature do pi | Onde consolidar |
|---|---|
| **Cache warming** (§1.11) | [73 — cache-prefixo](./backlog/cache-prefixo.md) |
| **Nested tool calls** (§1.12) | [45 — codemode](./backlog/codemode.md) |
| **Provider attribution** (§1.13) | [72 — retry-provider](./backlog/retry-provider.md) |
| **Timings** (§1.14) | [61 — metodo-medicao](./backlog/metodo-medicao.md) |
| **Truncagem** (§1.2) | [81 — resposta-grande-ficheiro](./backlog/resposta-grande-ficheiro.md) (goose) |
| **Catálogo remoto + overlay offline** (`remote-catalog-provider.ts`) | [24 — contexto-por-modelo](./backlog/contexto-por-modelo.md) |
| **Export HTML** (`export-html`) | [25 — export-sessao](./backlog/export-sessao.md) |
| **Diagnostics/crash log/bug report** | [89 — diagnostico-bundle](./backlog/diagnostico-bundle.md) (goose) |
| **`fs-watch` / `output-files`** | liga a [87 — hooks](./backlog/hooks.md) e [81](./backlog/resposta-grande-ficheiro.md) |
| **MCP servers / `env` (SSH) / `durable`** | fora (YAGNI) |
| **JSON event stream** (`json.md`) | confirma o stdout=dados do ktu (já acertado) |
| **RPC extension UI** | liga a [45 — codemode](./backlog/codemode.md) e à porta |

---

## 3. Confirmações (o ktu já acerta)

| Decisão do pi | O que o ktu já tem |
|---|---|
| stdout **reservado** para dados; diagnóstico em stderr | `stdout=dados, stderr=logs` |
| JSONL com framing estrito (LF; **não** usar `readline` por causa dos separadores Unicode) | log JSONL; parsing próprio |
| Skills anunciadas por nome+descrição, carregadas por demanda | `.agents/skills/` |
| Contexto reconstruído do **branch** activo | projeção `derive_messages` do log |
| Compactação insere um **resumo** e mantém o original endereçável | `compact.rs` (aditivo) |
| Sessões versionadas + migração automática | eventos versionados |
| Project trust antes de carregar recursos | guardrails fail-closed |
| Provider com retry e classificação de erro | `llama.rs` |

---

## 4. Não trazer (reafirma a revisão inicial)

`packages/ai` (15+ providers), `packages/durable` (runtime durável), `packages/mcp`,
`packages/env` (SSH), `packages/tui` (differential rendering), `packages/telemetry`,
`packages/evals`, extensões TS in-process (o ktu tem plugins por processo), npm como transporte,
share/bug-report na nuvem, shell aliases/terminal setup.

---

## 5. Recomendação

**Incorporar já (barato, alto valor):**
1. **Exposição de tools + `tool_search`** (§1.1) — catálogo pequeno sem perder tools.
2. **Truncagem sistemática** (§1.2) — duplo limite + output completo em ficheiro.
3. **Fila de mutação de ficheiro** (§1.3) — correcção sob paralelismo.
4. **Line-endings/BOM no `edit`** (§1.4) — correcção.
5. **`SYSTEM.md`/`APPEND_SYSTEM.md`** (§1.5) e **keybindings** (§1.6) — dados.
6. **`settings.json` com schema** (§1.7) e **precedência de credenciais** (§1.8) — config.

**Avaliar:**
7. **Visão/imagens** (§1.9), **`/reload`** (§1.10), e as consolidações do §2 (cache warming,
   nested tool calls, atribuição de provider, timings).

**Manter fora:** a lista do §4.

> **Nota de fecho.** Com as três revisões profundas (katu, goose, pi), o mapa está completo: o
> **katu** dá **governança** (ADRs/dossiê/método), o **goose** dá **segurança e consciência
> situacional** (injecção/egress/allowlist, MOIM), e o **pi** dá **higiene de tools e config**
> (exposição/truncagem/fila de mutação, schema). Nenhum deles exige copiar arquitectura — todos
> são **dados**.
