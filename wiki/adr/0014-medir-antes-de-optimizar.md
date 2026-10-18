# 0014 — Medir antes de optimizar (adoptar-ou-reverter)

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `crates/ktu-core/benches/core.rs`, `wiki/benchmarks/`, `wiki/metodo-medicao.md`

## Contexto

O katu fechou um plano de optimização (Q-21) **antes** de operar uma sessão: complexidade paga sem
sintoma. O ktu inverte isso — mas "medir" também pode virar cerimónia se cada número não declarar
de onde vem.

## Decisão

Nenhuma optimização entra sem **número** (ver `wiki/metodo-medicao.md`):

1. **Fórmula + artefacto + teste que o trava**: o que se mede, em que condições, com o JSON em
   `wiki/benchmarks/` e um teste que garante que o comportamento se mantém.
2. **Base de evidência** declarada em cada número: `measured` · `inferred` · `provider_reported` ·
   `unpriced`.
3. **A/B** com recorte cru (mesma máquina, mesma fixture, mediana).
4. **Adoptar-ou-reverter**: adopta se ganhar ≥ 20 % no alvo **ou** se remover complexidade; senão
   reverte e **escreve a rejeição com o número** (dossiê de pendências).

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Optimizar por intuição | foi o caminho do katu (plano Q-21 sem sessão); o resultado foi complexidade sem operação |
| Micro-benchmark contínuo no `make check` | medições são ruidosas: bloqueariam o gate por variação de máquina |
| Perfilar primeiro, decidir depois sem artefacto | o número não fica registado e a decisão não é re-litigável |
| Publicar números sem base de evidência | um `inferred` apresentado como `measured` é uma mentira útil — até deixar de ser |

## Consequências

- A prioridade do `plan.md` §2 (performance **terciária**) é verificável: o núcleo custa
  microssegundos e o gargalo é o modelo.
- As rejeições ficam registadas com número (ex.: "sem micro-optimização",
  `decision_01e2183g`) — não se repetem.
- Custo: escrever a medição e a rejeição; em troca, nenhuma complexidade entra por fé.
