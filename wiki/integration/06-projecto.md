# 06 — O projecto: `.agents/`, `.ktu/` e o que a fachada lê

> **Público:** devs · **Índice:** [README](./README.md) · **Anterior:** [05 — Portas e adaptadores](./05-portas-e-adaptadores.md) · **Seguinte:** [07 — Erros e diagnóstico](./07-erros-e-diagnostico.md)

## 1. Estrutura de ficheiros

A separação não é cosmética: **`.agents/` é conhecimento** (o humano revê e versiona);
**`.ktu/` é runtime** (o agente produz e consome). `AGENTS.md` é o *router* do projecto.

```
<root>/
├── AGENTS.md                        # instruções do projecto → entram no system prompt
├── **/AGENTS.md                     # instruções locais (por directório; tectos em §5)
├── .agents/                         # conhecimento (versionável)
│   ├── config.toml                  # comportamento (provider/modelo/effort/max_steps)
│   ├── permissions.toml             # política por tool (always|ask|never)
│   ├── memory.toml                  # canal semântico opcional (H8)
│   ├── memory/notes.jsonl           # fonte da verdade da memória
│   ├── guardrails/*.md              # regras declarativas (fail-closed)
│   ├── skills/ktu/SKILL.md          # protocolo do agente para este projecto
│   └── skills/<outras>/             # skills do agente
└── .ktu/                            # runtime (derivado; normalmente fora do git)
    ├── sessions/<id>.jsonl          # log da sessão (a sessão)
    ├── sessions/<id>.checkpoint.json
    ├── plan/<sessão>.json           # artefacto do modo plano
    ├── plugins/<nome>/plugin.toml   # plugins externos
    └── trash/                       # apagar = mover para aqui (+ index.jsonl)
```

Constantes do núcleo (`ktu_core::layout`): `INIT_DIRS`, `NOTES_FILE`, `SESSIONS_DIR`,
`SKILL_FILE`, `CONFIG_FILE`. O `ktu init` da CLI cria esta árvore — usa-o como referência
(`ktu init` num projecto novo e compara).

## 2. O que a fachada lê no `build()`

| Caminho | Obrigatório | Efeito |
|---|---|---|
| `<root>/.agents/guardrails/*.md` | não | regras declarativas → `DeclarativeGuard` (**fail-closed**) |
| `<root>/.agents/permissions.toml` | não | política por tool → `PermissionGuard` (último elo) |
| `<root>/AGENTS.md` | não | router: texto condensado anexado à base (raiz **primeiro**) |
| `<root>/**/AGENTS.md` | não | instruções locais (aninhadas, com cabeçalho `## <caminho>`) |
| `<root>/.agents/SYSTEM.md` | não | **substitui** a base do agente (`user_dir` perde para o projecto) |
| `<root>/.agents/APPEND_SYSTEM.md` | não | **acrescenta** ao fim do prompt |
| `<root>/.agents/commands/*.md` | não | comandos/receitas (`/nome` no REPL, `ktu run --command`; `ktu commands ls`) |
| `<root>/.agents/agents/*.md` | não | agentes declarativos (**fail-closed**; `ktu agents ls`) |

Se um ficheiro existir mas for inválido, `build()` **falha** (`Error::Invalid`) nomeando o
ficheiro — nunca degrada em silêncio (I16). Se o directório/ficheiro não existir, segue sem ele.

```rust
match Ktu::builder()…build() {
    Ok(ktu) => …,
    Err(error) => {                       // ex.: "guardrails invalidos: x.md: frontmatter ausente ou malformado"
        eprintln!("configuracao invalida: {error}");
        std::process::exit(1);            // a decisão de sair é da app, não do kernel
    }
}
```

## 3. Guardrails declarativos (`.agents/guardrails/*.md`)

Um ficheiro `.md` por regra, com frontmatter YAML:

```markdown
---
id: deny-dotenv          # obrigatório, estável (entra no log)
version: 1               # obrigatório (schema actual = 1)
applies_to: '*'          # opcional: '*' | 'ktu' (primário) | 'explorer'
fact: path               # obrigatório: 'tool' ou 'path'
match: '*.env'           # obrigatório: nome exacto da tool (fact=tool) ou glob (fact=path)
decision: deny           # 'deny' | 'escalate' (qualquer outro valor ⇒ regra IGNORADA)
reason: segredos         # obrigatório: razão humana (vai para o log)
remedy: 'usa trash_file' # opcional: remédio acionável devolvido ao modelo
---

# Porquê
…
```

Regras de ouro:

- **Uma regra que tente permitir é ignorada** — os ficheiros nunca desligam invariantes
  estruturais (I9/I10/I16).
- `deny` → `Decision::Deny(DenyReason::Guardrail)`; `escalate` → `Decision::Escalate` (o
  front-end só pode **restringir**, nunca alargar).
- A ordem de avaliação é determinística (por `id`), e a primeira regra que casa decide.
- Falha ao ler/interpretar **qualquer** ficheiro impede o arranque (I16).
- `fact: path` casa sobre o **path pedido** (antes de resolver): o `StructuralGuard` já garante que
  nada sai do *root* (`PathEscape`).

## 4. Permissões explícitas (`.agents/permissions.toml`)

```toml
[permissions]
bash = "ask"          # always | ask | never
write_file = "always"
trash_file = "never"
```

- `always` — executa sem pedir; `ask` — devolve `Escalate` (requer aprovação); `never` — nega
  (`DenyReason::Policy`).
- É o **último** elo da cadeia: só pode restringir o que os guards anteriores permitiram.
- Valor desconhecido → erro no `build()` (`permissao desconhecida para bash: talvez`).
- Ferramentas úteis: `ktu_core::permission::{load, Store, Permission}` (`Store::get/set/entries/render`).

## 5. `AGENTS.md`, directriz e o system prompt

- O prompt final = `prompt::BASE` **ou** `.agents/SYSTEM.md` (substitui) + `AGENTS.md` da raiz +
  `AGENTS.md` aninhados + `.agents/APPEND_SYSTEM.md` (acrescenta) — ver
  [convenções](../convencoes.md) §2–§3. Precedência: **projecto > utilizador** (`user_dir`).
- A descoberta dos aninhados é determinística e tem tectos (profundidade 8, 8 KiB/ficheiro,
  32 KiB total, 512 directórios) — nunca estoura o orçamento.
- Para ver o resultado: `ktu prompt` (imprime o prompt efectivo; `--json` inclui `bytes`).
- O prompt final inclui o `AGENTS.md` **condensado**:
  blocos de comentário HTML (`<!-- … -->`) removidos, linhas em branco colapsadas.
- Para compor o mesmo prompt fora da fachada: `facade::system_prompt(&fs, &root)` — é a **mesma**
  função (usa-a se precisares do texto antes de montar o `Ktu`, ex.: para um `SubagentHost`).
- O prompt efectivo está acessível em `ktu.system()` (útil em diagnóstico).

## 6. Memória (`.agents/memory/`)

### 6.1 Fonte da verdade

`.agents/memory/notes.jsonl` — uma nota por linha (`{frontmatter, body}`), o mesmo formato do
knudge. O índice é **derivado** e descartável (`ktu_memory::drain`). Consultável pelo knudge:

```bash
kd ask "…"                       # recall sobre as notas
kd ai memory --limit 5           # lista/consulta a fonte
ktu memory project               # projeta o log de sessão em notas derivadas
```

### 6.2 Usar na fachada

```rust
use ktu_memory::JsonlMemory;
let memory = JsonlMemory::new(&fs, &clock, &root, root.join(".agents/memory"));
// … .memory(&memory) no construtor, e Some(&memory) no Toolbox da T2
```

Sem `.memory(...)`: não há recall no arranque nem tools `memory_ask`/`memory_write`, e o kernel não
grava a nota de fim de turno.

### 6.3 Canal semântico opcional (H8)

`.agents/memory.toml` (schema fechado, chaves iguais às do knudge):

```toml
[embeddings]
provider = "http"                                  # none (default) | http
endpoint = "http://127.0.0.1:8889/v1/embeddings"
model = "google/embeddinggemma-2"
revision = "main"
dimensions = 768
prefix = "auto"                                    # auto | none | embeddinggemma
batch = 32
```

```rust
use ktu_core::MemoryConfig;
use ktu_provider::HttpEmbedder;

let config = match fs.read(&MemoryConfig::path(&root)) {
    Ok(bytes) => MemoryConfig::parse(&String::from_utf8_lossy(&bytes))?,   // chave desconhecida ⇒ Err
    Err(_) => MemoryConfig::default(),
};
if config.embeddings.enabled() {
    let embedder = HttpEmbedder::from_config(&config.embeddings)?;
    memory = memory.with_embeddings(&embedder, config.embeddings.clone());
}
```

Degradação graciosa em três níveis: sem `memory.toml`/`provider="none"` → lexical; provedor fora →
o recall lexical continua (o índice fica pendente); índice ausente/ilegível → lexical.
O dreno (`--digest`) e o worker (`service --install`) são ferramentas da CLI/`ktu_memory::drain`.

## 7. `config.toml` — comportamento (a app resolve, não a fachada)

`.agents/config.toml` (projecto) e `~/.config/ktu/config.toml` (global) usam **as mesmas chaves**:
`provider`, `base_url`, `model`, `effort`, `max_steps` (schema fechado — chave desconhecida é
erro). Precedência: **flag > projecto > global > defaults**; o projecto **ganha** chave a chave
(M6). As credenciais vivem só no global (`auth.toml`, 0600) e nunca entram no log nem no prompt.

A fachada **não** lê este ficheiro: ela recebe o resultado já resolvido (`provider`, `max_steps`).
Padrão recomendado na tua app:

```rust
use ktu_core::config::{Config, Auth, local_config_path, global_config_path};

let local = fs.read(&local_config_path(&root)).ok().and_then(|b| Config::parse(&String::from_utf8_lossy(&b)).ok());
let global = global_config_path(xdg, home).and_then(|p| fs.read(&p).ok()).and_then(|b| Config::parse(&String::from_utf8_lossy(&b)).ok());
let effective = Config::merge(local.unwrap_or_default(), global.unwrap_or_default());
let max_steps = flag.or(effective.max_steps).unwrap_or(24);
```

> `Config::merge(local, global)` implementa a regra “o projecto ganha”; `Auth` tem `Debug`
> **redigido** (nunca imprime chaves) — mantém esse cuidado nos teus logs.

## 8. Directórios de runtime: quem os cria

A porta `Fs` **não** cria directórios. Antes do primeiro `save_session` (ou do `FsPlanHost`), a
app garante:

```rust
fn ensure_layout(root: &Path) -> Result<(), String> {
    for dir in [root.join(".ktu/sessions"), root.join(".ktu/trash"),
                root.join(".ktu/plan"), root.join(".agents/memory")] {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    Ok(())
}
```

Alternativa: chamar `ktu init` (CLI) no arranque do projecto — é o que garante também `AGENTS.md`,
a skill e o `.gitignore`/exclusões.

## 9. Checklist

- [ ] `AGENTS.md` presente e sem comentários HTML desnecessários (são removidos).
- [ ] Guardrails com `id`/`version`/`fact`/`match`/`decision`/`reason` (e `remedy` quando útil).
- [ ] Permissões explícitas só onde precisas (o default já nega o que é estruturalmente proibido).
- [ ] `.agents/memory/notes.jsonl` versionado; `.ktu/` fora do git.
- [ ] `.agents/memory.toml` com `provider = "none"` se não usares embeddings.
- [ ] Directórios de runtime criados no arranque da app.
- [ ] Precedência de config resolvida na app (não duplicar defaults no código).

Próximo: [07 — Erros e diagnóstico](./07-erros-e-diagnostico.md).
