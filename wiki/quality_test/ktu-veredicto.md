# Veredicto da bateria de qualidade — sujeito `ktu` (G4)

> Condições: provider `opencode-go`, modelo `longcat-2.5-preview-free` (os fundos do
> `deepseek-v4.1-flash` usado pelos referenciais estavam esgotados — HTTP 402), TTY
> não-interativo, sessão encadeada Q1→Q4, `compacted: true` na Q4.
> Julgamento contra a `expected_answer_ideia` de cada `pN.md`, com os artefactos do ktu
> arquivados ao lado dos referenciais (`pN/ktu-pN.md`).
>
> *Corrida de referência:* 2026-10-15, após a correção do gate de memória (escrita
> automática no fim do turno). A sonda de persistência passa **sem** qualquer instrução
> extra no prompt.

## Resultado por questão

| Q | Expected (ideia) | Entregue | Veredicto |
|---|---|---|---|
| **Q1** | (vazio — só "para criança de 5 anos") | resumo infantil fiel ao knudge (caderno mágico, nota=verdade, índice descartável, ciclo, MCP) | **Equivalência** |
| **Q2** | subcomando `human` / flag `--human` com saída amigável | propõe um **modo humano** (REPL interativo em TTY, com `ask`/`write`/`task` guiados) + tabela de fricções | **Equivalência** (forma diferente da `--human`, mesma intenção e cobertura) |
| **Q3** | listar algoritmos, modelo TOON, uso `ask→write→task` com "procurar antes de escrever" | `exploration/knudge/ktu/knudge.md` (8 KB), resumo para investigador com arquitetura, TOON, BM25/RRF, dedup, ciclo de vida, MCP, testes, limitações | **Equivalência** |
| **Q4** | construir a cadeia de tarefas, vantagens/desvantagens globais | `exploration/knudge/ktu/EPIC.md` (15 KB): plano mestre, 6 fases, cadeia de issues sequenciais, riscos, critérios de sucesso | **Equivalência** |

## Critérios do gate

| Critério (plan.md §5) | Estado |
|---|---|
| Bateria isolada por sujeito (root privado, `REF/` read-only) | ✅ |
| Q1–Q3 na mesma sessão; Q4 com `compacted: true` | ✅ (`compacted=true`) |
| Sonda de persistência (processo novo recupera o registo) | ✅ (notas em `.agents/memory`, **sem** regra de prompt) |
| Artefactos `p*/ktu-*.md` ao lado dos referenciais | ✅ |
| Equivalência de grau por questão | ✅ |

## Correção do gate de memória (§2.7/§2.12)

O `plan.md` exige que a persistência **não** dependa de o modelo escolher chamar a tool
(`kernel/memory_gate`). A primeira versão só fazia *recall* automático; a escrita exigia
uma chamada do modelo. Foi corrigido:

- `ktu-core/src/agent/support.rs::record_session_end` grava **automaticamente** uma nota
  `meta` no fim de cada turno (evento `memory.write`), com dedup idempotente 0.75/0.92.
- Teste de regressão `memory_is_written_automatically_at_turn_end` (`tests/loop.rs`).
- O harness deixou de injetar qualquer regra de memória no prompt (as perguntas voltaram
  ao enunciado original).

## Ressalvas

- O modelo difere do referencial (`longcat-2.5-preview-free` vs `deepseek-v4.1-flash`),
  por indisponibilidade de fundos. Condição declarada no frontmatter dos artefactos.
- A Q2 propôs um REPL em vez de uma flag `--human`; é uma resposta equivalente (dentro
  da ideia esperada de "algo equivalente").
- Entregáveis em `wiki/quality_test/ktu-files/knudge/ktu/` (`knudge.md`, `EPIC.md`).

**Veredicto: equivalência de grau. Gate G4 satisfeito.**
