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

---

## Re-validação do gate (2026-10-17, pós-G5/G5.5/G6)

Re-execução da bateria com o código actual (depois de subagentes + modo plano,
endurecimento estrutural e porta única), nas mesmas condições do veredicto
original. Os artefactos `pN/ktu-pN.md`, `ktu-summary.md` e `ktu-files/` foram
refrescados por esta corrida.

- **Condições:** provider `opencode-go`, modelo `longcat-2.5-preview-free` (o
  `deepseek-v4.1-flash` continua sem fundos — HTTP 402), TTY não-interativo,
  Q1–Q3 na mesma sessão, Q4 `compacted: true`, sonda de persistência.
- **Mecânica:** `compacted=true`, `probe=true`; processo novo recuperou o registo
  de `.agents/memory/notes.jsonl` sem qualquer regra de prompt.

| Q | Expected (ideia) | Entregue (2026-10-17) | Veredicto |
|---|---|---|---|
| **Q1** | (vazio) | resumo infantil do knudge (caderno mágico; notas; busca; âncoras) | **Equivalência** |
| **Q2** | subcomando `human` / flag `--human` ou equivalente | propõe modo interactivo (TUI `kd explore`) + alternativa `kd browse` | **Equivalência** |
| **Q3** | algoritmos, TOON v3, uso `ask`→`write`→`task` com "ask primeiro" | `exploration/knudge/ktu/knudge.md` (8,3 KB): ciclo `ask→write→task→sync` com **"ask — pesquisar antes de gravar"**, TOON, BM25/âncoras/RRF, dedup, tarefas, CLI, MCP, embeddings, limitações | **Equivalência** |
| **Q4** | cadeia de tarefas, vantagens/desvantagens | `exploration/knudge/ktu/EPIC.md` (16,7 KB): plano mestre (6 fases) + 18 issues encadeadas, vantagens/desvantagens | **Equivalência** |

| Critério (plan.md §5) | Estado |
|---|---|
| Bateria isolada por sujeito (root privado, `REF/` read-only) | ✅ |
| Q1–Q3 na mesma sessão; Q4 com `compacted: true` | ✅ |
| Sonda de persistência (processo novo recupera o registo) | ✅ |
| Artefactos `p*/ktu-*.md` ao lado dos referenciais | ✅ |
| Equivalência de grau por questão | ✅ |

**Veredicto: equivalência de grau mantida. Gate G4 continua satisfeito (2026-10-17).**

### Confirmação pós-G7 (2026-10-17)

Repetida a bateria **depois** de G7 (hooks + guardrails + plugins), para garantir
que as mudanças no loop não alteraram o comportamento. As entregas de Q3/Q4
voltam a cobrir as ideias esperadas (Q3: `ask`/`write`/`task`, TOON, BM25/RRF,
âncoras, dedup, MCP; Q4: fases, issues encadeadas, vantagens/desvantagens) e
`compacted=true`/`probe=true`. **Sem regressão.**

Observação (não é regressão da G7): nesta corrida o modelo escreveu os
entregáveis via `bash` com **caminhos absolutos para o repositório** em vez do
workspace isolado — a tool `bash` não é confinada ao *root* (registado como
`risk_01ua5ei2`). O gate mantém-se válido porque os artefactos canónicos são os
da corrida limpa; a robustez da isolação face a modelos que fogem ao *root* fica
como risco a mitigar.

### Corrida com o modelo do referencial (2026-10-17)

Com a credencial nova (`OPENCODE_API_KEY`), correu-se a bateria com o **modelo do
referencial** — `deepseek-v4.1-flash`, `thinking-level: low` — a condição original
de `p1`–`p4`. Resultado: **equivalência de grau em Q1–Q4**, `compacted=true`,
`probe=true`. Os artefactos canónicos (`p*/ktu-pN.md`, `ktu-summary.md`,
`ktu-files/`) foram refrescados por esta corrida.

| Q | Expected (ideia) | Entregue (`deepseek-v4.1-flash`) | Veredicto |
|---|---|---|---|
| **Q1** | (vazio) | história infantil do knudge (cartõezinhos, mapa derivado, âncoras, procura-antes-de-escrever) | **Equivalência** |
| **Q2** | subcomando `human` / flag `--human` ou equivalente | análise verificada de 6 fricções + **Fase 0** concreta (D223–D226) | **Equivalência** |
| **Q3** | algoritmos, TOON v3, `ask`→`write`→`task` com "ask primeiro" | `exploration/knudge/ktu/knudge.md` (329 linhas): TOON, BM25/RRF, âncoras, dedup, MCP | **Equivalência** |
| **Q4** | cadeia de tarefas, vantagens/desvantagens | `exploration/knudge/ktu/EPIC.md` (467 linhas) + medições | **Equivalência** |

**Correção de protocolo exigida pelo modelo do referencial.** O `deepseek-v4.1-flash`
(thinking mode) rejeitava pedidos com **HTTP 400** ("the `reasoning_content` in the
thinking mode must be passed back"). O ktu descartava o pensamento na projecção de
wire; passou a reenviá-lo como `reasoning_content` **em todas as mensagens de
assistente** para gateways `OpenAI` (o `llama-server` continua a não o receber).
`Message::Assistant` ganhou o campo `reasoning`; testes
`openai_flavor_echoes_reasoning_content` e `openai_flavor_echoes_empty_reasoning_too`.

**Veredicto: equivalência de grau com o modelo do referencial. Gate G4 satisfeito (2026-10-17).**



