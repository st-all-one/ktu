# H2 — Configuração global/local

> **Issue:** `task_00tx0uvu` · **Âncora:** [config-global](../backlog/config-global.md) ·
> **Desenho:** [cli-e-config](../cli-e-config.md) §3 · **Pré-requisitos:** H0, H1.

## Objetivo
Config em **dois níveis** — global (`~/.config/ktu/`) + local (`.agents/`) — com `auth.toml`
só-global, precedência declarada, e recursos globais (skills/guardrails/plugins/recipes).

## Passos
1. **Global** `$XDG_CONFIG_HOME/ktu/`: `auth.toml` (0600), `config.toml`, `memory.toml`,
   `skills/`, `guardrails/`, `plugins/`, `recipes/`. **Local** `.agents/config.toml` + dirs.
2. **Precedência** — comportamento: `flag > --params > local > global > default`; credenciais:
   `flag > env > auth.toml`.
3. **`ktu config get/set/unset/list --local|--global`** + schema + migrações.
4. **Recursos globais** — skills/plugins/recipes = *defaults* (local sobrepõe por nome);
   **guardrails globais = piso aditivo** (nunca removíveis).
5. **Segurança** — `auth.toml` 0600, fora do repo, nunca no log/contexto; teste dedicado.

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_00obedtc` | `~/.config/ktu/` + local |
| `task_00xgy3g9` | Precedência + schema + migrações |
| `task_007q2ui5` | `ktu config get/set/unset/list` |
| `task_01d5i5zs` | Recursos globais (sobreposição + piso) |
| `task_009fnilh` | Testes auth-não-ecoa + guardrail-global-não-removível |

## Gate / testes
- Local sobrepõe global **chave a chave**; `auth.toml` nunca replicável ao projecto.
- Teste **M3:** `auth.toml` não aparece no log nem no contexto.
- Teste **M5:** tentar desligar um guardrail global é **ignorado**.
- Migração de config antiga → nova versão (golden).

## Riscos
- Dupla verdade da memória → `memory.toml` **projeta** no knudge (não motor paralelo).
- Config global a sobrepor o projecto → **proibido** (M6); o projecto é a autoridade.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **N1:** chaves de config em `BTreeMap` (ordem estável no `config list`).
- **N5/N2:** segredos **nunca** passam pelo `Logger`; sem `unwrap` na leitura do TOML.
- **Determinismo:** a resolução de precedência é uma **função pura** (global+local+flags→efectiva);
  testável sem I/O.
- **N8:** validar contra schema; erros tipados com `remedy`.
- **Gate:** **M3** (auth não ecoa) e **M5** (guardrail global não removível) — parte de **G14**.

## Execução (parcial)
- **Implementado:** `ktu-core::config` (puro) — `Config` (chaves fechadas), `Auth` (**`Debug`
  redigido** — M3), `Config::merge` (local ganha chave a chave — M6), parse/render TOML,
  `set`/`unset`/`get`/`entries`, caminhos global (XDG) + local. `ktu config list/get/set/unset`
  com `--local`/`--global`; integração no `run`/`serve`
  (`flag > --params > local > global > default`; credenciais `flag > env > auth.toml`).
- **Verificado:** `set --local/--global`; `list` efectivo mostra a fonte; `run` usa
  `provider=mock` da config e falha com `provider=nope`; `--provider`/`--params` ganham; `Auth`
  nunca imprime a chave.
- **Adiado (YAGNI):** **recursos globais** (skills/guardrails/plugins) — o runtime ainda **não**
  carrega skills nem guardrails (só `ktu guardrails list`); **migrações** — o schema ainda é v1,
  sem versão anterior a migrar. Implementar quando essas features entrarem (H10/G1 e o
  carregamento de guardrails).
- **Gate:** `make check` + `typos` + `machete` verdes.
