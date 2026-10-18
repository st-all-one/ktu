---
id: task_01j0lf38
type: task
statement: Metodo de medicao (61) + teto de superficie (62)
created_at: 2026-10-17T22:54:44.198Z
body_hash: 47ff6512
schema_version: 2
tags: [fase2]
revision: 3
outcomes:
  - status: success
    recorded_at: 2026-10-18T03:57:02.743Z
    notes: "Metodo de medicao + teto de superficie entregues. (a) wiki/metodo-medicao.md: formula + artefacto + teste que trava; basis measured/inferred/provider_reported/unpriced; A/B por mediana; adoptar-ou-reverter (>=20% no alvo ou remocao de complexidade, senao reverter e registar a rejeicao com o numero); caso real documentado (sem micro-optimizacao, GDK, BTreeMap). O artefacto do bench passa a schema 2 com basis por amostra e por bloco; plan.md ganhou a regra em 4.1 e a leitura do benchmark foi actualizada (10k eventos ~0,5 ms, turno 1 passo ~7 us). (b) surface.toml (teto 47/87/13) + ktu self surface (medida pelo binario: arvore do clap + catalogo builtin + tools do loop) + xtask check-surface no make check (make surface). Testado: com o teto baixo o gate falha com mensagem accionavel; o teto actual passa."
anchors: [wiki/backlog/metodo-medicao.md]
status: closed
scope: task
---
<!-- knudge:parent task_01ehkt9h -->