# Análise crítica do backlog — o ktu como agente útil e mínimo

> Revisão crítica das **123 propostas** do [backlog](./backlog/README.md), com um único filtro:
> **isto torna o ktu um agente realmente útil *sem* deixar de ser mínimo?** Não é um índice: é um
> **tribunal**. Data: 2026-10-17.

---

## 0. O diagnóstico (antes das propostas)

O ktu tem **13 124 linhas** de `src/`, 8 crates, `make check` verde e a bateria `quality_test`
**equivalente aos referenciais** (G4). O backlog tem **123 páginas** — cerca de **9× o código**.
Isso não é uma fila de trabalho: é um **catálogo de tudo o que 6 referenciais fazem**.

O §0 do `plan.md` proíbe subsistemas sem um teste que os exija. O backlog, ao listar 123
"candidatas", contradiz o próprio §0: tornou-se a forma de manter vivo o "talvez" que o §0 manda
matar. **O maior risco do ktu não é faltar uma feature — é ter 123 razões para as construir.**

**Auto-crítica.** As seis revisões (`katu`, `goose`, `pi`, `opencode`, `knudge` + iniciais) foram
**exaustivas mas acríticas**: catalogaram tudo com um veredicto local ("incorporar/avaliar") sem
um **orçamento global**. Uma feature pode ser boa *isolada* e má *em conjunto* — porque cada uma
custa manutenção, superfície e atenção. Este documento corrige isso: **o veredicto é agora
relativo ao todo**.

---

## 1. O teste de utilidade (5 perguntas)

Uma proposta só entra se responder **sim** a ≥ 2, e **não** violar a última:

| # | Pergunta | O que mede |
|---|---|---|
| **U1** | Reduz turnos **falhados** (tool inválida, eco, JSON quebrado, leitura repetida)? | fiabilidade |
| **U2** | Reduz **turnos por tarefa** (menos idas e voltas)? | eficiência |
| **U3** | Torna as **edições** mais correctas (atomicidade, endings, paralelismo)? | correcção |
| **U4** | Sobrevive a condições reais (contexto longo, modelo fraco, crash, cancel, output grande)? | robustez |
| **U5** | **Não** acrescenta um subsistema a manter (é dado, correcção ou porta)? | minimalismo |

> Corolário: **governança** (ADRs, dossiê, método) não passa U1–U4 — mas passa U5 e é **barata**.
> Tem um tier próprio. **Formalismo** (rate-distortion, confiança, PPR) falha U1–U5: é o que o
> §0 proíbe.

---

## 2. Crítica transversal

### 2.1 O backlog duplica-se

Vários "itens" são **o mesmo mecanismo** visto de referenciais diferentes. Contando-os como um,
123 → ~**90 decisões reais**:

| Mecanismo | Páginas | Consolidar em |
|---|---|---|
| Templates / receitas / comandos | 19, 41, 53 | **um** (templates + shell) |
| Compactação (resumo, parâmetros, gatilho) | 22, 48 | um |
| Output grande / truncagem | 81, 94 | um |
| Cache de prefixo / warming / afinidade | 73, 108 | um |
| Structured output / gramática | 9, 74 | um |
| Codemode / nested / code-mode | 36, 45 | um |
| Permissões / política / por-tool / store | 1, 21, 82, 110 | um |
| Config (precedência, schema, migrações, dois níveis) | 76, 83, 99, 122 | um |
| Git (versionado, sync) | 16, 123 | um |
| Contexto (por modelo, embeddings, rate-distortion) | 6, 24, 121 | um |
| Sessão (tree, fork, snapshot, versões, mover) | 40, 49, 78, 85, 111 | um |

### 2.2 Categoria errada (o que o backlog revela)

- **Plumbing de provider** (72, 73, 74, 107): o provider (gateway) já resolve grande parte; o ktu
  deve ser **fino**, não reimplementar o gateway.
- **Segurança a mais** para um agente **local de dev** (55 PTY, 80 injecção, 86 adversário, 90
  allowlist, 91 egress): valem num produto multi-utilizador; num kernel local são
  over-engineering — excepto o que protege o **confinamento** já existente.
- **Governança** (59–62): não torna o agente útil, torna o **projecto** sustentável. Barata e
  valiosa — mas noutro tier.
- **Formalismo** (2, 3, 4, 5, 6, 119, 120): o katu mediu, rejeitou e **provou** que não paga sem
  consumidor. Repropor isto é o erro que o §0 proíbe.

### 2.3 O backlog como sintoma

Um backlog de 123 páginas num projecto cuja tese é "mínimo" é uma **contradição performativa**. O
katu morreu com 70 k linhas e **0 sessões** por ter construído policy/audit/rate-distortion *antes*
do loop. O ktu **já tem o loop a funcionar**; o risco agora é o inverso — afogá-lo em features
"candidatas" que ninguém pediu.

---

## 3. Triagem — os 123 em quatro tiers

### Tier 1 — **Núcleo útil** (fazer; ~14 itens, alto impacto no uso real)

O agente é útil se: **edita bem**, **não falha turnos**, **gere contexto**, **lembra**, **é seguro**.

| # | Item | U1 | U2 | U3 | U4 | U5 |
|---|---|---|---|---|---|---|
| 75 | `edit` multi-bloco atómico | | ✅ | ✅ | ✅ | ✅ |
| 96 | line-endings/BOM no `edit` | | | ✅ | ✅ | ✅ |
| 95 | fila de mutação de ficheiro | | | ✅ | ✅ | ✅ |
| 94 | truncagem sistemática (linhas+bytes) | | ✅ | | ✅ | ✅ |
| 70 | continuação de leitura truncada | ✅ | ✅ | | | ✅ |
| 68 | recusa acionável (`remedy`) | ✅ | ✅ | | | ✅ |
| 106 | prompt de `MaxSteps` | ✅ | | | ✅ | ✅ |
| 65 | deteção de eco | ✅ | | | ✅ | ✅ |
| 67 | tool calls declaradas (modelos locais) | ✅ | | | ✅ | ✅ |
| 107 | reparação de input de tool | ✅ | | | ✅ | ✅ |
| 63 | política de durabilidade do log | | | | ✅ | ✅ |
| 114 | knudge: `Store`+`notas/`+`eventos/`+`write` | | | | ✅ | ✅ |
| 115 | knudge: `rewind` (contexto de sessão) | | ✅ | | ✅ | ✅ |
| 116 | knudge: `doctor` (saúde da memória) | | | | ✅ | ✅ |

**Porquê estes:** todos passam U1–U4 **e** U5 (são correcções, dados ou portas — nenhum é um
subsistema novo). São o que separa "funciona no `quality_test`" de "funciona no dia-a-dia com um
modelo qualquer".

### Tier 2 — **Usabilidade real** (fazer a seguir; ~10 itens)

Não são correcção nem fiabilidade — são **conforto de operação**. Valem depois do Tier 1.

| # | Item | Porquê |
|---|---|---|
| 104 | inbox/`steer` durável | falar durante o turno sem o matar |
| 103 | snapshots + `/undo`/`/redo` | corrigir sem recomeçar |
| 85 | editar em sessão / fork | idem |
| 64 | lock de turno entre processos | CLI + serve |
| 105 | execution claim (crash) | retomar sem ciclo |
| 41 | templates/commands (unificar 19/53) | repetir prompts |
| 20 | slash commands | atalhos |
| 84 | nome de sessão | listar sessões |
| 52 | AGENTS.md aninhados | instruções por directório |
| 51 | agentes declarativos | agentes próprios |

### Tier 3 — **Avaliar com medição** (só se doer; ~30 itens)

Nada aqui entra sem um **sintoma medido**. Ficam como estão (páginas), sem prioridade.

`22` resumo estruturado · `23` tokens · `24`/`121` contexto/embeddings · `6` selecção ·
`9`/`74` structured output · `72` retry · `73`/`108` cache · `42` steering · `43` virtual models ·
`45` codemode · `47` trust · `48` compactação · `49` versões · `76`/`83`/`99`/`122` config ·
`77` layout · `78` snapshot · `117` task · `88` checks · `87` hooks · `89` diagnóstico ·
`92` success checks · `71` prime · `57` simulation · `56` referências · `58` custom tools ·
`98` keybindings · `100` credenciais · `102` reload · `54` formatter · `82` permissões por tool.

### Tier 4 — **Podar / arquivar** (não fazer; ~60 itens)

| Grupo | Páginas | Veredicto |
|---|---|---|
| **YAGNI §0 explícito** | 12 MCP, 13 GDK, 15 sandbox, 26 ACP, 27 desktop, 28 candle, 29 roaming, 30 scheduler, 31 OAuth, 32 voz, 33 OTel, 34 download, 35 state-machine, 38 distros, 55 PTY, 86 adversário, 90 allowlist, 91 egress, 101 visão, 111 mover sessão, 113 proxy, 119 grafo, 120 lifecycle | **arquivar** (já decidido) |
| **Formalismo prematuro** | 1 política, 2 auditoria, 3 confiança, 4 feedback, 5 diag-catalog, 7 preços, 8 modelo-por-fase, 10 TOON, 17 TUI completa, 18 stats | **arquivar** (o katu mediu e rejeitou) |
| **Duplicados** | 19→41, 53→41, 48→22, 94→81, 108→73, 9→74, 36→45, 82→21, 83/99/122→76, 123→16, 121→24 | **fundir** (não eliminar) |
| **Já coberto** | 11 (rejeitado por medição), 14 (alternativa ao GDK), 37/39 (tool monitor) | **manter como referência** |
| **Nota de rodapé** | 46 temas, 50 chord, 59–62 governança, 61 método | **reclassificar** (governança ≠ feature) |

> **Nota.** "Arquivar" ≠ apagar: a página fica como **registo da decisão** (é o que o dossiê do
> katu faz). O que muda é **sair da fila** — deixa de competir por atenção.

---

## 4. O conjunto mínimo (a proposta)

Se o ktu tivesse de escolher **12 itens** e nada mais, seriam estes, **por esta ordem**:

1. **75** `edit` multi-bloco atómico — a edição é o acto central do agente.
2. **96** line-endings/BOM — correcção de edição.
3. **95** fila de mutação — correcção sob paralelismo.
4. **70** continuação de leitura — mata o ciclo de leitura repetida.
5. **68** recusa acionável — mata o ciclo de recusa repetida.
6. **65** deteção de eco — mata a resposta-lixo de modelos fracos.
7. **67**+**107** tool calls declaradas + reparação — torna modelos locais utilizáveis.
8. **94** truncagem sistemática — protege o orçamento.
9. **106** prompt de `MaxSteps` — desfecho útil.
10. **63** durabilidade do log — robustez.
11. **114**+**115**+**116** knudge (`Store`/`write`, `rewind`, `doctor`) — o **epicentro de memória**
    fica robusto.
12. **59**+**60**+**61** governança (ADRs, dossiê, método) — mantém o projecto honesto.

**Porque esta ordem:** as 1–9 tornam o **loop** fiável com qualquer modelo; a 10–11 tornam a
**memória** robusta (o diferencial do ktu); a 12 impede o backlog de voltar a crescer. Tudo o
resto é Tier 2+.

**O que isto custa:** nenhum subsistema novo. São correcções, um wrapper de ficheiro, dados e
portas. O binário continua ~6 MB.

---

## 5. O que podar — e a regra de poda

**Podar não é perder; é parar de olhar.** Proposta:

- **Arquivar** os Tier 4 num `wiki/backlog/arquivo/` (com um cabeçalho "decisão registada; não
  reabrir sem número novo"), tirando-os do índice activo.
- **Fundir** os duplicados (uma página, com as variantes dos referenciais citadas).
- **Reclassificar** a governança (59–62) como **processo**, não feature — vive no `plan.md`/ADRs,
  não no backlog.
- **Regra de poda (nova, no `plan.md`):** *uma página só entra no backlog activo se citar o
  **teste** ou o **sintoma** que a exige; caso contrário, é uma nota de leitura, não uma decisão.*

O backlog activo deve caber em **≤ 25 páginas**. As outras 100 são bibliografia.

---

## 6. Riscos

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| **Scope creep** — o backlog vira roadmap | alta | **alto** | podar (§5); teto de superfície (62) |
| **Construir o que o katu rejeitou** | média | alto | Tier 4 arquivado; "não repropor sem número novo" |
| **Confundir governança com utilidade** | alta | médio | tier próprio; não competir com o loop |
| **Plumbing de provider a mais** | média | médio | provider fino (Tier 3) |
| **Perder o diferencial** (memória) | média | alto | knudge no Tier 1 |

---

## 7. Recomendação

1. **Aceitar este tribunal:** o backlog activo passa a **Tier 1 (14) + Tier 2 (10) = 24 itens**.
2. **Arquivar Tier 4** e **fundir duplicados** (§5).
3. **Executar o Tier 1** pela ordem do §4 — são **correcções e portas**, não subsistemas.
4. **Não abrir o Tier 2** antes de o Tier 1 estar verde em `make check`.
5. **Só reabrir o Tier 3 com um número** (medição) — nunca por analogia com um referencial.
6. **Reclassificar a governança** como processo (ADRs + dossiê), fora da fila de features.

> **A tese, em uma frase.** Um agente mínimo é útil quando **edita bem, não falha turnos e
> lembra-se** — tudo o resto é o backlog a tentar virar produto. O ktu já tem o loop; falta-lhe
> **fiabilidade** (Tier 1) e **disciplina** (poda). As 123 páginas são um mapa excelente e uma
> armadilha perfeita.
