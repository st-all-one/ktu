---
id: task_00oxye8w
type: task
statement: Convencoes do ktu movidas para a base do prompt; init deixa de gerar a skill ktu
created_at: 2026-10-19T07:14:57.429Z
body_hash: 22fb2e8f
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-19T07:15:04.627Z
    notes: "Feito: convencoes no BASE, init sem skill ktu, doctor = layout+memory_source+config; G14 16/16, make check 0, typos 0, links 0, prompt 995->737 bytes."
anchors: [crates/ktu-core/src/prompt.rs, crates/ktu-core/src/layout.rs, crates/ktu-cli/src/init.rs, crates/ktu-cli/src/doctor_cmd.rs]
status: closed
scope: task
---
A skill `.agents/skills/ktu/SKILL.md` gerada pelo `ktu init` duplicava o `ktu prime` (ciclo, memoria, convencoes) e entrava no system prompt como camada de skills. Medido num projecto novo: `ktu prompt` = 995 bytes com a skill e 458 sem ela => 537 bytes/turno de conteudo duplicado. Decisao (ADR 0022): o init deixa de gerar a skill e as **convencoes** (`.agents/` = conhecimento versionavel, `.ktu/` = runtime, `AGENTS.md` = instrucoes do projecto que entram no prompt) passam para o `prompt::BASE` - o prompt do proprio agente, que e onde a identidade e a disciplina do ktu vivem. O ciclo NAO se duplica: continua no `prime` e a base ja diz "trabalhas em passos". Removido: layout::{SKILL_FILE, SKILL_BODY}, o passo do init, e o item `skill` do doctor (que passa a diagnosticar a estrutura que o init cria: layout, memory_source = .agents/memory/notes.jsonl, config = .agents/config.toml - existencia, nao conteudo, que tem porta por area). Prova: teste base_carries_the_project_conventions (BASE contem .agents/, .ktu/, AGENTS.md, ktu prime), init.rs sem a skill, gate G14 16/16 com "o init nao gera a skill do ktu". Resultado medido: 737 bytes num projecto novo (contra 995) = -258 bytes/turno (-26%) e a informacao mantida.