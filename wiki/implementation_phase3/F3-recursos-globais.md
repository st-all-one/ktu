# F3 — Recursos globais (skills, plugins, receitas) e o piso visível

> **Issue:** `task_004bvdzb` · **Âncora:**
> [config-global](../backlog/config-global.md) · **Pré-requisitos:** H2 (config dois níveis), H10
> (convenções), H12 (piso M5) · **Gatilho:** um **2.º projecto** a precisar da mesma
> skill/plugin/receita.

## Objetivo

`~/.config/ktu/{skills,plugins,recipes}/` como *defaults* do utilizador — o **local sobrepõe por
nome** e o piso global **nunca** é removível. E tornar o piso **visível**: o runtime já carrega os
guardrails globais, mas `ktu guardrails list` só mostra os do projecto.

**Invariante novo — M8:** *nada que o runtime use é invisível* — todo o recurso efectivo
(guardrail, skill, plugin, receita) aparece num comando `ls`, com a **origem** (global/local).

## Estado verificado (2026-10-18)

| Peça | Estado |
|---|---|
| config global/local (`~/.config/ktu/config.toml`) | feito (H2) |
| **guardrails globais carregados no runtime** (piso M5) | **feito (H12)** — `facade/builder.rs:186` |
| `ktu guardrails list` mostra os **globais** | **falta** (só lê `.agents/guardrails`) |
| skills globais compõem o prompt | **falta** (o runtime não lê `.agents/skills/` para o prompt) |
| plugins globais (aditivos) | **falta** (`PluginHost::load(root/.ktu/plugins)`, só projecto) |
| receitas globais | **falta** (receitas ≡ `.agents/commands/`, só locais) |

> **A premissa antiga do A1 expirou:** dizia «o runtime **ainda não carrega** guardrails». É falso
> desde a H12 (foi a correcção que tirou M5 da vacuidade). O que sobra é o *listing* e os outros
> três tipos de recurso.

## Passos

1. **L6 — piso visível:** `ktu guardrails list` (e `--json`) mostra as regras dos **dois** níveis,
   com `origem: global|local`, na ordem efectiva (global primeiro — é o piso aditivo).
2. **Skills globais:** `~/.config/ktu/skills/<nome>/SKILL.md`; o prompt compõe por nome, local
   ganha (ordem estável); tecto de tamanho como o `AGENTS.md` (H10).
3. **Plugins globais:** o `PluginHost` carrega **dois** dirs (global + projecto) e a **geração**
   combina ambos; um plugin do projecto com o mesmo nome **não** desliga o global (aditivo).
4. **Receitas globais:** `command::load` lê `~/.config/ktu/recipes/` + `.agents/commands/`; local
   ganha por nome.
5. **Visibilidade:** `ktu commands ls`/`ktu plugins list`/`ktu agents ls` ganham `origem`.

## Tarefas kd

| Tarefa | Entrega |
|---|---|
| `task_00ni0ppt` | Recursos globais (skills/plugins/receitas + precedência) |
| `task_00g46zet` | Piso visível (`guardrails list` com origem) — **L6** |

## Gate / testes

- **G18** — ver [GATES](./GATES.md).
- Precedência por nome: local **ganha** (teste com o mesmo nome nos dois níveis).
- **M5:** uma regra do projecto **não** desliga uma global (já travado; estender ao `list`).
- Aditivo: um plugin global + um local ⇒ **dois** nomes na geração.
- `guardrails list --json` traz `origem` para **todas** as regras efectivas.

## Riscos

- Skills no prompt a inchar o contexto → tecto (como o `AGENTS.md`) e medição antes/depois.
- Ordem instável no prompt → `BTreeMap` (I6).
- Precedência ambígua → **regra declarada**: guardrails **somam** (piso); skills/receitas
  **substituem por nome**; plugins **somam**.

## Estado da implementação (2026-10-18) — **fechado**

Gate **G18** verde (10/10): [relatório](../gates/G18-recursos.md) · artefactos em
`wiki/gates/artifacts/G18/`.

| Entrega | Onde |
|---|---|
| `Origin::{Global, Local}` (M8/N8: enum, não `bool`) | `crates/ktu-core/src/origin.rs` (novo) |
| Guardrails: `load_layered` + piso visível | `crates/ktu-core/src/guardrails.rs`, `.../facade/{mod,builder}.rs`, `crates/ktu-cli/src/guardrails_cmd.rs` |
| **Skills** (novo): dois níveis, local ganha, tecto do `AGENTS.md`, secção do prompt | `crates/ktu-core/src/skill.rs` + `.../skill/tests.rs` (novos), `crates/ktu-core/src/prompt.rs`, `.../facade/mod.rs` |
| Receitas: `load_layered` (local ganha por nome) | `crates/ktu-core/src/command.rs`, `crates/ktu-cli/src/{commands_cmd,repl,run_cmd}.rs` |
| Plugins: dois níveis **aditivos** + `entries()` com origem | `crates/ktu-plugin/src/host.rs`, `crates/ktu-cli/src/{plugins_cmd,turn_runtime}.rs` |
| `ktu skills ls` (superfície nova, justificada) | `crates/ktu-cli/src/skills_cmd.rs` (novo), `surface.toml` (47→49 subcomandos, 88→89 flags) |

**Precedência declarada** (não emergente): guardrails e plugins **somam** (o piso global é
aditivo — o local nunca o desliga, M5); skills e receitas **substituem por nome** (o local ganha).

**Skill = router.** O que entra no prompt é o corpo do `<nome>/SKILL.md` (sem frontmatter): é um
*router* que diz quando usar o protocolo e onde estão os detalhes (que o agente lê com
`read_file`). O frontmatter das skills é o do pi/knudge (YAML com valores dobrados), pelo que a
leitura do cabeçalho é **tolerante** (`skill::split_header`) — o parser escalar do ktu recusaria
a skill `rust` real.

**Medição do risco** (skills a inchar o contexto), no próprio repositório: `ktu prompt` passa de
**1952** para **6187** bytes (+4235 B ≈ 1,1 k *tokens*) com a skill `rust` (4,4 KB) — dentro do
tecto (8 KB/ficheiro, 32 KB total, o mesmo do `AGENTS.md`); acima do tecto a skill é **ignorada**
com aviso.

**Desvios registados:** `ktu agents ls` **não** ganhou `origem` (os agentes não têm nível global;
a coluna seria constante — o M8 enumera guardrails, skills, plugins e receitas).

## Notas de execução (skills Rust)

> Ver [NOTAS-RUST](./NOTAS-RUST.md).
- **N1:** catálogo em `BTreeMap` (ordem estável no prompt e no `ls`).
- **N2:** recurso malformado = ignorado com aviso (`stderr`), nunca *panic*.
- **N8:** `origem: Origem::{Global, Local}` é enum, não bool.
