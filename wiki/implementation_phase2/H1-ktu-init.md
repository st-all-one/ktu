# H1 — `ktu init`

> **Issue:** `task_01v41q8n` · **Âncora:** [ktu-init](../backlog/ktu-init.md) ·
> **Desenho:** [cli-e-config](../cli-e-config.md) §4 · **Pré-requisitos:** H0.

## Objetivo
Preparar **todo** o ambiente do projecto de forma **idempotente**: layout, skill do ktu, bloco no
`AGENTS.md`, git gerido (`.git/info/exclude` + `.gitattributes`), `--force`/`--git-excluded`.

## Passos
1. **Blocos idempotentes** — `upsert` que casa a **linha inteira** (após `trim`); marcadores
   `<!-- ktu:start -->`/`<!-- ktu:end -->` + `<!-- ktu:version: N -->` (AGENTS.md) e
   `# ktu:start`/`# ktu:end` (`.gitattributes`).
2. **Layout + skill + template** — `.agents/` (skills/guardrails/memory/recipes) + `.ktu/`
   (sessions/plugins/trash/plan/logs); `.agents/skills/ktu/SKILL.md`; `.agents/config.toml`.
3. **Git gerido** — `.git/info/exclude` (modo `--git-tracked` exclui só o derivado;
   `--git-excluded` exclui tudo) com reversão ao mudar de modo; `.gitattributes` (`eol=lf`,
   `merge=union` no log, `binary` no derivado). **Nunca** `.gitignore` (D30).
4. **`--force`/`--no-prompt`** + **relatório** por item (`novo`/`atualizado`/`inalterado`).

## Tarefas kd
| Tarefa | Entrega |
|---|---|
| `task_01vf73ny` | Blocos idempotentes + marcadores |
| `task_00a032jf` | Layout + skill + template de config |
| `task_01x106b8` | `.git/info/exclude` + `.gitattributes` |
| `task_010dk52y` | `--force`/`--no-prompt` + relatório |

## Gate / testes
- Reexecutar `ktu init` **não** duplica nem reescreve blocos.
- `.git/info/exclude` reverte ao alternar `--git-tracked`/`--git-excluded`.
- Fora de repo git: no-op (não falha). `--force` não apaga memória.
- `AGENTS.md`/`.gitattributes` com os marcadores correctos (golden).

## Riscos
- `--force` a apagar conhecimento → **só** reescreve ficheiros geridos.
- `.gitignore` → proibido; usar sempre `.git/info/exclude`.

## Notas de execução (skills Rust)
> Ver [NOTAS-RUST](./NOTAS-RUST.md) (transversais).
- **Determinismo:** o `upsert` de blocos é uma **função pura** (texto→texto); reexecutar ⇒ mesmo
  resultado. Teste **golden** dos marcadores.
- **1.99:** `String::from_utf8_lossy_owned` (1.99) ao ler ficheiros geridos (§2); evita clone.
- **N2:** sem `unwrap` no parse de ficheiros existentes; tolerar ausência/UTF-8 inválido.
- **N1:** nenhuma estrutura de ordem não-determinística nos relatórios.
- **Gate:** contribui para **G14** (project-bound).

## Execução (concluída)
- **Implementado:** `ktu-core::git` (puro: `block`/`agents`/`attributes`/`exclude`) +
  `ktu-cli::init` (borda: layout, skill, config, git, relatório). Tarefas `task_01vf73ny`,
  `task_00a032jf`, `task_01x106b8`, `task_010dk52y` fechadas; issue `task_01v41q8n` fechada.
- **Verificado:** reexecutar `ktu init` ⇒ tudo `inalterado`; alternar `--git-tracked`/
  `--git-excluded` reverte o `.git/info/exclude` e o bloco do `.gitattributes`; fora de repo ⇒
  no-op; `--force` preserva memória e guardrails editados; envelope `--json`.
- **Desvios (YAGNI):** `.agents/recipes/` e `.ktu/logs/` **não** criados (nada os usa; `plan.md`
  §2.11 não os lista) — ficam para a H10.
- **Gate:** `make check` + `typos` + `machete` verdes; `init.rs` 397 linhas (limite 400).
