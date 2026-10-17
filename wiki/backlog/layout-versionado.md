# Layout central versionado

- **Estado:** candidata (do `katu`, ADR 0021) — **avaliar**.
- **Origem:** `REF/katu/wiki/_ref/adr/0021-layout-central-katu-e-versionamento.md`;
  `wiki/revisao-katu.md` §3.14.
- **Âncora:** `.agents/`, `crates/ktu-cli/src/init.rs`.

## O que é
Um directório de estado do projecto com layout explícito; **sempre fora do git** (efémero/sensível),
o resto segue `git.versioned`; blocos geridos em `.gitignore`/`.git/info/exclude`; `--init --force`
preserva o conhecimento personalizado.

## Porque interessa
O ktu tem `.agents/` (config/travas). Falta uma política **explícita** do que é versionado (config,
notas, travas) e do que nunca é (estado local). O `.git/info/exclude` evita versionar decisões
locais.

## O que o ktu tem hoje
`.agents/` (guardrails, skills) + `.knudge/`; sem layout/política de versionamento explícita.

## Pré-condições para reconsiderar
- Estado local a distinguir do versionado.

## Custo e risco
- **Esforço:** médio.
- **Risco de não fazer:** estado local versionado por acidente.
- **Risco de fazer:** exclusões escondidas — blocos geridos com marcadores, explícitos.

## Decisão pendente
Avaliar. Layout explícito; sempre-fora-do-git para efémero/sensível; resto segue `git.versioned`;
blocos geridos com marcadores.
