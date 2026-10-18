# ADRs (Architecture Decision Records)

- **Estado:** **incorporada** (H11) — `wiki/adr/` (17 ADRs, contrato de 4 secções com
  `Alternatives considered`, substituição ligada nos dois sentidos) + `wiki/postmortems/`
  (PM-0001–PM-0003), com gate próprio (`xtask check-adr` no `make check`).
- **Origem:** `REF/katu/wiki/_ref/adr/` (27 ADRs); `wiki/revisao-katu.md` §1.1.
- **Âncora:** `plan.md`, `STAGING.md`, `wiki/`.

## O que é
Cada decisão vive numa ADR imutável (`NNNN-titulo.md`) com secções obrigatórias
`Contexto` / `Decisão` / **`Alternatives considered`** / `Consequências`. Uma ADR não se edita:
**substitui-se** por outra ligada por `Supersedes`/`Superseded by`. Um gate (`xtask check-docs`)
falha se faltar a secção das alternativas.

> *"Uma decisão arquitectural sem o que venceu convida a re-litigá-la."*

## Porque interessa
O ktu decide em `plan.md`/`STAGING.md` (prosa corrida). Uma ADR por decisão dá rastreabilidade e
impede re-litigar. As decisões já registadas no knudge (`decision_00aw5etj` GDK, `decision_01e2183g`
sem micro-optimização) ganhariam um lar versionado.

## O que o ktu tem hoje
`plan.md` (§0/§2/§3/§6) + `STAGING.md`; decisões no knudge (`decision_*`).

## Pré-condições para reconsiderar
- Vontade de manter o processo (uma ADR por decisão de fase).

## Custo e risco
- **Esforço:** baixo (`wiki/adr/NNNN-*.md` + `xtask check-adr`).
- **Risco de não fazer:** decisões re-litigadas; contexto perdido.
- **Risco de fazer:** cerimónia — mitigar mantendo as ADRs curtas.

## Decisão pendente
Promover. Formato de 4 secções obrigatórias, imutável, substituída-e-ligada; `xtask check-adr`
verifica as secções. Inclui **postmortems** (`wiki/postmortems/`) para incidentes reais.
