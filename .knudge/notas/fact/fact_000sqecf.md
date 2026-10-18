---
id: fact_000sqecf
type: fact
statement: "H11: ADRs imutaveis com Alternatives considered e postmortems, com gate proprio no make check"
created_at: 2026-10-18T03:56:51.222Z
body_hash: 00c9642e
schema_version: 2
revision: 2
anchors: [xtask/src/governance.rs, Makefile]
---
H11 (task_01ng3be9). wiki/adr/ nasce com 17 ADRs (0001-0017) e contrato verificado por xtask check-adr (no make check via make adr): nome NNNN-slug.md com id unico, titulo com o id, quatro seccoes obrigatorias (Contexto/Decisao/Alternatives considered/Consequencias) e status aceite|substituida|proposta; a substituicao e verificada nos DOIS sentidos (0016 substituida por 0017 e 0017 supersedes 0016) e o README tem de listar todos os ids (indice sem deriva). A ADR 0016 (adiar A9/A10/A11 na H10) e a 0017 (implementar antes da H11) existem para exercitar o mecanismo de substituicao a serio - nao e cerimonia decorativa. wiki/postmortems/ nasce com PM-0001 (modulo de testes apagado por um script de extraccao, duas vezes no mesmo dia), PM-0002 (snippets canonicos da doc de integracao que nao compilavam, E0716) e PM-0003 (links internos da wiki a apodrecer em silencio), com Contexto/Impacto/Causa/Correcao/Prevencao. O check foi testado com casos negativos (remover Alternatives considered, referencia inexistente, id fora do indice, quebra de simetria, falta de Prevencao) - todos falham. ADR 0011 consolida os tres limites mecanicos (400 linhas, firewall de camadas, teto de superficie).