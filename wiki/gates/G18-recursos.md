# G18 — Recursos globais e piso visível (M8)

- **Veredicto:** ✅ **verde** (10/10) · **Artefactos:** `artifacts/G18/` · **Data:** 2026-10-18
- **Item:** [F3](../implementation_phase3/F3-recursos-globais.md) ·
  **Critério:** [GATES da Fase 3](../implementation_phase3/GATES.md) §G18.

## Cenário

Um projecto com a **mesma** skill e a **mesma** receita nos dois níveis (`~/.config/ktu/` e
`.agents/`), um guardrail em cada nível, e três plugins (dois globais, um local com o nome de um
global).

## Verificações

| # | Critério | Observado |
|---|---|---|
| 1 | skills: o local ganha por nome | uma só `rust`, com o corpo local |
| 2 | skills: a global sem homónimo continua visível | `kd global` |
| 3 | receitas: o local ganha por nome, com origem | `revisao local` + `so-global global` |
| 4 | guardrails: o piso global vem **primeiro** | `deny-env global`, `deny-bash local` |
| 5 | M8: nenhuma regra efectiva fica sem origem | 2/2 com origem |
| 6 | M5: o projecto **não** desliga a regra global | `allowed=false` com regras locais no projecto |
| 7 | plugins: **aditivos** (o mesmo nome nos dois níveis corre 2×) | `g1 global`, `g2 global`, `g1 local` |
| 8 | o prompt compõe as skills **efectivas** (local ganha) | `## rust — rust local`, sem o global |
| 9 | o prompt compõe também a skill só global | `## kd — kd global` |
| 10 | receitas: a precedência vale no **runtime** | o `max_steps` efectivo é o da receita local |

## Artefactos

`skills.txt` · `plugins.txt` · `guardrails.json` (com `origin`) · `prompt.txt`.

## Medição do risco (skills no prompt)

O risco declarado era «skills a inchar o contexto». Medido no próprio repositório (que tem a
skill `rust`, 4,4 KB): `ktu prompt` passa de **1952** para **6187** bytes (+4235 B, ~1,1 k
*tokens*) — uma skill, dentro do tecto de 8 KB por ficheiro e 32 KB no total (o mesmo tecto do
`AGENTS.md`, H10). Uma skill acima do tecto é **ignorada** com aviso, nunca truncada.

## Desvios (registados)

- **`ktu agents ls` não ganhou origem.** Os agentes declarativos não têm nível global (não há
  `~/.config/ktu/agents/`): a coluna seria uma constante. O M8 enumera guardrails, **skills**,
  plugins e receitas — os quatro tipos com dois níveis.
- **`ktu skills ls` é superfície nova** (subcomandos 47 → 49, flags 88 → 89, justificado em
  `surface.toml`): as skills compõem o prompt e não tinham `ls` — sem ele, o M8 seria vacuidade.
