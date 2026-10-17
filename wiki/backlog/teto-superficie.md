# Teto de superfície

- **Estado:** candidata (do `katu`) — **incorporar**.
- **Origem:** `REF/katu/surface.toml`, `xtask check-surface`; `wiki/revisao-katu.md` §1.5.
- **Âncora:** `crates/ktu-cli/src/cli.rs`, `xtask/src/main.rs`.

## O que é
A superfície pública tem um **teto versionado**; `xtask check-surface` falha se crescer sem subir o
teto numa PR. Kill criterion K3: *"se o teto exceder em duas fases consecutivas sem valor, podar
antes de continuar"*.

## Porque interessa
O ktu preza minimalismo; um teto **mecânico** impede o crescimento silencioso da CLI/tools. É o
complemento natural do `check-layers` e do limite de 300 linhas por ficheiro.

## O que o ktu tem hoje
`xtask check-layers` (firewall) + `scripts/check_file_length.sh` (300 linhas). Sem teto de
superfície.

## Pré-condições para reconsiderar
- Definir o que conta como "superfície" (subcomandos + flags + tools).

## Custo e risco
- **Esforço:** baixo (`surface.toml` + alvo no `xtask`).
- **Risco de não fazer:** crescimento silencioso da CLI.
- **Risco de fazer:** teto mal medido bloqueia trabalho legítimo — subir o teto é explícito.

## Decisão pendente
Promover. Contagem determinística (subcomandos + flags + tools), teto versionado, subida explícita.
