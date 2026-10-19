---
id: decision_00g2g6aq
type: decision
statement: Convencoes do ktu na base do prompt; init nao gera a skill ktu (ADR 0022)
created_at: 2026-10-19T07:15:05.097Z
body_hash: 575334fb
schema_version: 2
anchors: [crates/ktu-core/src/prompt.rs, crates/ktu-core/src/layout.rs, wiki/adr/0022-convencoes-na-base-do-prompt.md]
---
Decisao (ADR 0022). O `ktu init` deixa de gerar a skill `.agents/skills/ktu/SKILL.md` e as convencoes (`.agents/` = conhecimento versionavel; `.ktu/` = runtime; `AGENTS.md` = instrucoes do projecto que entram no prompt) passam para o `prompt::BASE`. Motivo: o corpo da skill entra no system prompt (camada de skills) e era um subconjunto do `prime`/`BASE` - medido, 537 bytes/turno num projecto novo; as convencoes sao estrutura que o agente usa em cada turno, logo pertencem ao prompt do proprio agente, nao a um ficheiro gerado por projecto. Alternativas descartadas: (a) manter a skill - o mesmo defeito que a ADR 0021 removeu do AGENTS.md; (b) deixar as convencoes so no `prime` - o prime e a pedido, e as convencoes sao usadas em cada turno; (c) por as convencoes no `.agents/SYSTEM.md` - seria escrever na directriz do humano (que substitui a base) e um projecto sem SYSTEM.md ficava sem elas; (d) gerar a skill so com o description - uma skill vazia no prompt e ruido e o description serve para o modelo escolher skills; (e) manter o init a actualizar a skill com --force - caminho de actualizacao para um ficheiro que ja nao e nosso. O ciclo nao se duplica na base (continua no `prime`; a base ja diz "trabalhas em passos"). Medido depois: 737 bytes num projecto novo contra 995 (-258, -26%) sem perder informacao.