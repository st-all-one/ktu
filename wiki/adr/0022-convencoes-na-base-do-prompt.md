# 0022 — As convenções do ktu vivem na base do prompt (não numa skill gerada)

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/src/prompt.rs` (`BASE`), `crates/ktu-core/src/layout.rs`
  (`SKILL_FILE`/`SKILL_BODY` removidos), `crates/ktu-cli/src/init.rs`

## Contexto

O `ktu init` (H1) gerava, por projecto, uma **skill do ktu**
(`.agents/skills/ktu/SKILL.md`) cujo corpo resumia o protocolo: o ciclo, a disciplina de memória
e as convenções (`.agents/` = conhecimento · `.ktu/` = runtime).

Depois de remover o bloco do `AGENTS.md` ([ADR 0021](0021-o-init-nao-escreve-no-agents-md.md)),
ficou o mesmo problema na skill: o corpo dela **entra no system prompt** (as skills locais são uma
camada — `prompt::compose_layers`) e é um **subconjunto** do que o agente já tem. Medido num
projecto recém-iniciado:

| | `ktu prompt` |
|---|---|
| com a skill `ktu` gerada | 995 bytes |
| sem ela | 458 bytes |

Ou seja: **537 bytes por turno** de conteúdo duplicado — o ciclo está no `ktu prime`, a disciplina
de memória está no `prompt::BASE`, e as convenções estavam a ser mantidas em dois sítios (a skill
gerada e o `prime`).

## Decisão

**O `ktu init` não gera a skill do ktu** e as **convenções passam para o `prompt::BASE`** — o
prompt do próprio agente, que é onde a identidade e a disciplina do ktu vivem:

> Convenções do projecto: `.agents/` é o conhecimento (skills, guardrails, memória — versionável);
> `.ktu/` é o runtime (sessões, plugins, planos, trash); o `AGENTS.md` (raiz e aninhados) traz as
> instruções do projecto e entra no prompt. Protocolo completo: `ktu prime`.

- `layout::{SKILL_FILE, SKILL_BODY}` **saem**; o `ktu init` deixa de escrever conteúdo sobre o
  próprio ktu em qualquer sítio (gera layout, `notes.jsonl`, guardrails *seed*, o template de
  config e o git).
- O `ktu doctor` passa a diagnosticar a **estrutura que o `init` cria**: `layout`,
  `memory_source` (`.agents/memory/notes.jsonl`) e `config` (`.agents/config.toml`) — existência,
  não conteúdo (o conteúdo tem porta por área).
- O ciclo **não** se duplica na base: continua no `ktu prime` (que é o protocolo completo) e a
  base já diz «trabalhas em passos».

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Manter a skill gerada | 537 bytes/turno de conteúdo que o `BASE` e o `prime` já dão — o mesmo defeito que a ADR 0021 removeu do `AGENTS.md` |
| Deixar as convenções só no `prime` | O `prime` é **a pedido**: as convenções (onde vive o conhecimento e o runtime) são estrutura que o agente usa em cada turno, não uma consulta ocasional |
| Pôr as convenções no `.agents/SYSTEM.md` | Seria escrever na directriz do humano (que *substitui* a base) para dizer o que é do agente — e um projecto sem `SYSTEM.md` ficava sem elas |
| Gerar a skill só com o `description` (sem corpo) | Uma skill vazia no prompt é ruído; e o `description` serve para o **modelo escolher** uma skill, não para o agente saber onde vive o conhecimento |
| Deixar o `ktu init` actualizar a skill se ela existir (`--force`) | O ktu deixaria de a gerar; manter um caminho de actualização para um ficheiro que já não é nosso é peso morto |

## Consequências

- O prompt do agente ganha as convenções **uma vez** (no `BASE`, sempre presente) e o projecto
  deixa de ter uma cópia a apodrecer: medido, **737 bytes** num projecto novo contra 995 antes
  (−258, −26%) — e sem perder a informação.
- `.agents/skills/` fica reservado às skills **do projecto** (e às globais do utilizador, que
  substituem por nome) — `ktu skills ls` deixa de mostrar uma skill que o ktu escrevia para si
  próprio.
- O `BASE` cresce ~280 bytes; quem quiser um prompt mais curto tem `.agents/SYSTEM.md` (substitui a
  base) ou `APPEND_SYSTEM.md` (acrescenta) — a via declarada, do humano.
- O gate **G14** trava-o: «o init não gera a skill do ktu» e «o init não toca no `AGENTS.md` do
  projecto».
