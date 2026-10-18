---
id: task_01ng3be9
type: task
statement: ADRs com Alternatives considered + postmortems (59)
created_at: 2026-10-17T22:54:43.501Z
body_hash: "34625969"
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T03:57:02.387Z
    notes: "ADRs + postmortems entregues. wiki/adr/ com 17 ADRs (0001-0017) sob contrato verificado (xtask check-adr, no make check via make adr): nome NNNN-slug.md, titulo com o id, 4 seccoes obrigatorias (Contexto/Decisao/Alternatives considered/Consequencias), status aceite|substituida|proposta, substituicao simetrica nos dois sentidos e README a listar todos os ids. A ADR 0016 (adiar A9/A10/A11) e a 0017 (implementar antes da H11) exercitam a substituicao a serio. wiki/postmortems/ com PM-0001 (modulo de testes apagado pelo script de extraccao, duas vezes), PM-0002 (snippets da doc que nao compilavam, E0716) e PM-0003 (links da wiki a apodrecer). Check testado com 5 casos negativos (falta de Alternatives considered, referencia inexistente, id fora do indice, simetria quebrada, falta de Prevencao) - todos falham como devem. ADR 0011 consolida os limites mecanicos. Gate verde."
anchors: [wiki/backlog/adr-governanca.md]
status: closed
scope: task
---
<!-- knudge:parent task_01ehkt9h -->