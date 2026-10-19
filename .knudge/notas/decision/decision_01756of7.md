---
id: decision_01756of7
type: decision
statement: "Skills compoem o prompt pelo corpo do SKILL.md (router), com o tecto do AGENTS.md - nao so o indice"
created_at: 2026-10-18T14:48:47.243Z
body_hash: bd1ce1f3
schema_version: 2
anchors: [crates/ktu-core/src/skill.rs, crates/ktu-core/src/prompt.rs, wiki/implementation_phase3/F3-recursos-globais.md]
---
F3 (skills). Decisao: o prompt compoe o CORPO de cada <nome>/SKILL.md (frontmatter fora), nao um indice nome+descricao. Motivo: o SKILL.md e um router (diz quando usar o protocolo e onde estao os detalhes, que o agente le com read_file) - sem o corpo a skill fica inerte (o agente teria de adivinhar pelo nome). Alternativas descartadas: (a) so indice (nome+descricao+path) - obriga a um read_file extra por skill e o "Use When" (que e o que decide usar) nunca chega ao modelo; (b) compor todas as sub-ficheiros da skill - contexto ilimitado, contra o orcamento; (c) lazy por tool (uma tool skill_read) - superficie nova e o modelo teria de saber que a skill existe, o que exige o indice de qualquer forma. Custo medido no repo: +4235 bytes (~1.1k tokens) para a skill rust de 4.4KB; tecto 8KB/ficheiro e 32KB total (o do AGENTS.md, H10) - acima do tecto a skill e ignorada com aviso. Corrigido em: ktu prompt 1952->6187 bytes.