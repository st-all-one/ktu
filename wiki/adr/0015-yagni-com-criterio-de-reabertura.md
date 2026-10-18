# 0015 — YAGNI com critério de reabertura

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `wiki/pendencias.md`, `wiki/implementation_phase2/ADIADOS.md`, `plan.md` §0

## Contexto

Um backlog de referenciais (goose, pi, opencode, katu, knudge) gera centenas de ideias boas. O katu
morreu por as aceitar todas. Ao mesmo tempo, "não fazer" sem rasto transforma-se em esquecimento —
ou em re-litígio da mesma discussão meses depois.

## Decisão

Nada entra sem **consumidor**: uma feature só se implementa quando há um **sintoma** (um teste que
falha, um número, uma utilização real). O que não entra fica registado no **dossiê de pendências**
(`wiki/pendencias.md`) com:

- estado (`⛔ Deferido` · `🚫 Rejeitado` · `⏳ Opt-in` · `🔴 Lacuna` · `⚪ Formalismo`) e eixo
  (`Q` qualidade · `P` performance · `S` simplificação);
- a **âncora** no código e o **critério de reabertura** (o teste/sintoma que a reabre);
- a rejeição com o **número** que a fundamentou, quando existe.

O teto de superfície ([0011](0011-limite-de-400-linhas-e-firewall-mecanico.md)) é a face mecânica
desta regra.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Implementar o que "claramente vamos precisar" | foi o erro do katu (70k linhas, 0 sessões) |
| Adiar sem registo | a ideia volta em três meses sem contexto, e a discussão recomeça |
| Backlog aberto (todas as ideias activas) | cresce sem poda; nenhuma página diz *porque* está lá |
| Regra só na cabeça do autor | não sobrevive a outra pessoa (nem ao autor, três meses depois) |

## Consequências

- O dossiê é a memória do "não fizemos isto, e eis quando o faremos".
- Uma reversão é **legítima** — quando o dono do projecto pede explicitamente, ou quando o sintoma
  aparece: passa a ADR nova (ex.: [0017](0017-implementar-a9-a11-antes-da-h11.md) substitui
  [0016](0016-h10-adiar-receitas-ricas-tool-search-e-agents-na-leitura.md)).
- Custo: manter o dossiê honesto (verificar contra o código a cada fase); é o preço de não repetir
  discussões.
