# knudge — `git::sync` (versionar a memória)

- **Estado:** candidata (do `knudge` 0.6.1) — **avaliar** (consolida com [16](./git-versionado.md)).
- **Origem:** `crates/knudge/wiki/integration/11_git_e_config.md`,
  `crates/knudge/wiki/specs/git-e-worktree.md`; `wiki/revisao-knudge.md` §6.2.
- **Âncora:** `crates/ktu-cli/src/session_store.rs`, `.agents/`.

## O que é
`Project` resolve o **worktree principal** (submódulo no próprio top-level); `onboard` aplica
exclusões em `.git/info/exclude` (**nunca** `.gitignore`, D30) e `.gitattributes` (`merge=union`
nos eventos); `sync` faz `git -C <raiz> add <layout>/notas <layout>/eventos` + commit com mensagem
gerada do último evento. `Versioned` exclui só o derivado (`.idx/`/`cache/`/`.locks/`);
`LocalOnly` exclui o diretório inteiro.

## Porque interessa
O ktu tem `.agents/` (conhecimento) e `.ktu/` (runtime). O modelo do knudge separa
**conhecimento versionado** de **derivado local**, com blocos geridos e `merge=union` — exactamente
a página [16 — git-versionado](./git-versionado.md), mas **implementada**.

## O que o ktu tem hoje
`.agents/` versionado; sem `sync`/exclusões geridas nem `merge=union`.

## Pré-condições para reconsiderar
- Necessidade de versionar a memória do agente no git.

## Custo e risco
- **Esforço:** baixo-médio (adoptar `git::{exclude,attributes,sync}`).
- **Risco de não fazer:** memória fora do versionamento.
- **Risco de fazer:** commitar derivado — as exclusões geridas previnem; `merge=union` nos eventos.

## Decisão pendente
Consolidar em [16](./git-versionado.md): adoptar o modelo do knudge (exclusões geridas em
`.git/info/exclude`, `.gitattributes` com `merge=union`, `sync` por evento). Não duplicar.
