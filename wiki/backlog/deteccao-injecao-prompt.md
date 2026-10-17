# Deteção de injecção de prompt

- **Estado:** candidata (do `goose`) — **incorporar**.
- **Origem:** `REF/goose/crates/goose/src/security/{patterns,scanner}.rs`;
  `wiki/revisao-goose-profunda.md` §1.2.
- **Âncora:** `crates/ktu-core/src/guardrails.rs`, `crates/ktu-core/src/taint.rs`.

## O que é
Intercepta o **parâmetro** de cada tool call **antes de executar** e compara-o com **padrões
de ameaça** (`ThreatPattern { name, pattern, description, risk_level, category }`; categorias
`FileSystemDestruction`/`RemoteCodeExecution`/`DataExfiltration`/`SystemModification`/
`NetworkAccess`/`ProcessManipulation`/`PrivilegeEscalation`/`CommandInjection`). Cada risco tem
uma **confiança** (Critical 0.95 … Low); acima do limiar, a execução **pausa** e o utilizador
decide (**Allow Once** / **Deny**), com *finding ID* logado. *"Salvaguarda, não garantia."*

## Porque interessa
O ktu tem `taint.rs` (marca conteúdo não confiável) — a deteção é a **outra metade**:
reconhecer padrões perigosos **antes** de executar. É **dado declarativo** (padrões + limiar),
determinístico e barato — encaixa no `Guard`/guardrails.

## O que o ktu tem hoje
`taint.rs` (tag de conteúdo não confiável) + guardrails por caminho/padrão; sem padrões de
ameaça nem limiar de confiança.

## Pré-condições para reconsiderar
- Um caminho de execução que beneficie (ex.: `bash` a partir de conteúdo untrusted).

## Custo e risco
- **Esforço:** baixo-médio (padrões + limiar + alerta Allow Once/Deny).
- **Risco de não fazer:** injecção executa comandos perigosos.
- **Risco de fazer:** falsos positivos — limiar configurável + "Allow Once"; nunca bloquear
  silenciosamente.

## Decisão pendente
Promover. Padrões como **dado** (categorias + nível), limiar configurável, alerta com
Allow Once/Deny e *finding* logado; determinístico.
