---
id: task_00povuok
type: task
statement: ktu init deixa de escrever no AGENTS.md: o protocolo vive em ktu prime e na base do prompt
created_at: 2026-10-19T07:06:07.621Z
body_hash: 29e9226c
schema_version: 2
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-19T07:06:16.526Z
    notes: "Feito: init sem AGENTS.md, git::agents removido, doctor = layout+skill, ADR 0021; G14 15/15, make check 0, typos 0, links 0."
anchors: [crates/ktu-cli/src/init.rs, crates/ktu-core/src/prompt.rs, crates/ktu-cli/src/prime.rs, wiki/adr/0021-o-init-nao-escreve-no-agents-md.md]
status: closed
scope: task
---
O `ktu init` escrevia um bloco gerido (`<!-- ktu:start -->` + marcador de versao) no AGENTS.md com o ciclo, a disciplina de memoria e as convencoes. Verificado: (a) o AGENTS.md e uma camada do system prompt (agents_md::discover -> prompt::compose_layers), logo entra em TODOS os turnos; (b) o bloco era um **subconjunto** do prompt::BASE (que ja diz "consulta memory_ask antes de decidir e grava cada aprendizado com memory_write") e do `ktu prime` (CICLO e CONVENCOES identicos); (c) o AGENTS.md e o ficheiro de instrucoes DO PROJECTO, escrito pelo humano. Removido: o passo do init, o modulo ktu_core::git::agents (bloco, marcadores, protocol_block, version_in) e o item agents_md do ktu doctor (que passa a diagnosticar layout e skill). O version_in, incorporado no doctor na tarefa anterior, caiu com isto (sem bloco gerado nao ha versao a ler). Prova: init.rs::the_project_agents_file_is_never_touched (o ficheiro fica byte-identico), tests/doctor.rs sem o teste do bloco, gate G14 15/15 com "o init nao toca no AGENTS.md do projecto". Decisao na ADR 0021.