# Project trust (confiança no projecto)

- **Estado:** candidata (do `pi`) — **avaliar**.
- **Origem:** `REF/pi/packages/coding-agent/docs/security.md`;
  `REF/pi/packages/coding-agent/docs/how-pi-works.md` (trust); `wiki/revisao-pi.md` §2.9.
- **Âncora:** `.agents/`, `crates/ktu-cli/src/init.rs`.

## O que é
Resolver a **confiança no projecto** antes de carregar settings/recursos do projecto —
porque extensões/skills/templates podem **executar código** ou **instruir o modelo**.

## Porque interessa
O ktu carrega `.agents/` (skills, guardrails, memória) sem confiança explícita. Num
repositório de terceiros, isso é uma superfície de **prompt injection** e de execução. Um
modelo de trust **fail-closed** é simples e defensivo.

## O que o ktu tem hoje
Carrega `.agents/` do directório do projecto; o `taint` (I13) cobre o output de tool, mas
não a **origem** dos recursos.

## Pré-condições para reconsiderar
- Uso do ktu em repositórios de terceiros (não confiáveis).
- Definição do que exige trust (guardrails? skills? plugins?).

## Custo e risco
- **Esforço:** baixo-médio (estado de trust + gate de carregamento + comando).
- **Risco de não fazer:** carregar recursos de um repo hostil.
- **Risco de fazer (YAGNI):** fricção a mais para uso local próprio.

## Decisão pendente
Promover se o ktu for usado em repos de terceiros. Modelo: **fail-closed** (sem trust, só
o essencial carrega); trust é decisão **explícita** e registada.
