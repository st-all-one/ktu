---
id: decision_01iqvasu
type: decision
statement: "Leitura do frontmatter das skills tolerante ao YAML do pi/knudge (valores dobrados), sem segundo parser completo"
created_at: 2026-10-18T14:48:48.388Z
body_hash: 28298fe0
schema_version: 2
anchors: [crates/ktu-core/src/skill.rs]
---
F3 (skills). Decisao: o cabecalho das skills e lido por skill::split_header, tolerante - junta a continuacao indentada de um valor dobrado (> ou |) e nao valida nada; o parser escalar do ktu (frontmatter::split) recusaria a skill rust real do repo (description: > com 2 linhas). Motivo: as skills sao do ecossistema pi/knudge (YAML), nao dos documentos do ktu (escalares); recusar a skill real tornaria a funcionalidade inutil no proprio repo. Alternativas descartadas: (a) exigir frontmatter escalar - quebra a compatibilidade com as skills existentes; (b) usar um crate YAML - dependencia nova para ler 2 campos (nome vem da directoria, so a descricao vem do cabecalho); (c) tratar tudo como corpo - o cabecalho apareceria no prompt. O nome da skill e a DIRECTORIA (identidade estavel, sem parse), pelo que so a descricao depende do cabecalho.