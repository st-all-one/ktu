# Git versionado de sessões/notas

- **Estado:** deferido (seam).
- **Origem:** `plan.md` §2.12-B (`[git] versioned`).
- **Âncora:** `.gitignore` (`.ktu/`), `.knudge/`.

## O que é
Versionar automaticamente **sessões e notas** (`.ktu/sessions`, `.agents/memory`) no git,
com commits controlados pelo runtime (ex.: no fim de um turno), para histórico e
recuperação.

## O que o ktu tem hoje
- `.ktu/` (runtime) é **gitignored**; o conhecimento versiona-se em `.knudge/`/`.agents/`.
- **Sem commits pelo agente** (regra local): o versionamento é humano.
- `kd sync` existe no knudge, mas o agente não o corre.

## Porque não entrou
Conflita com a regra local (**sem commits pelo agente**) e com a separação runtime
(`.ktu/`, descartável) vs conhecimento (`.agents/`, versionado). Não há consumidor.

## Pré-condições para reconsiderar
- Uma decisão explícita de que o runtime pode escrever no git.
- Um caso de uso (histórico de sessões, auditoria) que o justifique.

## Custo e risco
- **Esforço:** médio (porta `Git` + política de commits).
- **Risco de não fazer:** baixo.
- **Risco de fazer:** poluir o histórico com commits automáticos; contrariar a regra.

## Decisão pendente
Manter fora enquanto vigorar "sem commits pelo agente". Reabrir é decisão **humana**.
