# `ktu init` — preparar o ambiente do projecto

- **Estado:** **implementado** (H1; propósito: uso #4 + M6) — feito.
- **Origem:** [CLI e configuração](../cli-e-config.md) §4; tática: `kd init`/`onboard` (D30/D31/D34/D60/D162).
- **Âncora:** `crates/ktu-cli/src/init.rs`, `crates/ktu-core/src/layout.rs`.

## O que é
`ktu init` prepara **todo** o ambiente do projecto, de forma **idempotente**:

```
ktu init [--force] [--git-excluded | --git-tracked] [--no-prompt]
```

1. **Cria o layout:** `.agents/` (`skills/`, `guardrails/`, `memory/`, `recipes/`) + `.ktu/`
   (`sessions/`, `plugins/`, `trash/`, `plan/`, `logs/`).
2. **Copia elementos:** skill `.agents/skills/ktu/SKILL.md` (protocolo do agente), template
   `.agents/config.toml`, e o **bloco gerido** no `AGENTS.md`.
3. **Ajusta o git** (nunca `.gitignore`):
   - **`.git/info/exclude`** — modo `--git-tracked` (exclui só o derivado/runtime) ou
     `--git-excluded` (exclui **tudo** de `.agents/`+`.ktu/`);
   - **`.gitattributes`** — bloco gerido: `eol=lf` nas notas/eventos, `merge=union` no log de
     eventos, `binary` no derivado.
4. **Emite o `prime`** (salvo `--no-prompt`) e um **relatório** por item (`novo`/`atualizado`/`inalterado`).

## Porque interessa
Hoje `ktu init` só cria pastas. Falta: **git gerido**, **skill do ktu**, **bloco no `AGENTS.md`**,
**marcadores idempotentes** e **`--force`/`--git-excluded`**. É o que torna "um projecto ktu" um
gesto único e repetível — e o que liga a memória/config ao **projecto** (M6).

## O que o ktu tem hoje
`ktu init` cria `INIT_DIRS` (`.agents/skills|guardrails|memory`, `.ktu/sessions|plugins|trash|plan`)
e o `notes.jsonl`. Sem git, sem skill, sem bloco `AGENTS.md`, sem `--force`/`--git-excluded`.

## Pré-condições para reconsiderar
- Nenhuma.

## Custo e risco
- **Esforço:** médio (blocks idempotentes + git + skill).
- **Risco de não fazer:** ambiente preparado à mão; git poluído com runtime.
- **Riscos a mitigar:** `--force` a apagar conhecimento (nunca: só reescreve ficheiros geridos);
  operações de git fora de repo (no-op, não falha); `.gitignore` (nunca usado — D30).

## Decisão pendente
Incorporar. Táticas do knudge: **marcadores** (`<!-- ktu:start -->`/`<!-- ktu:end -->`,
`<!-- ktu:version: N -->`, `# ktu:start`/`# ktu:end`), **`upsert` por linha inteira** (idempotente),
**exclude gerido** que reverte ao mudar de modo.
