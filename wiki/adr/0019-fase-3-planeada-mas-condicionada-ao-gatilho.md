# 0019 — Fase 3 planeada mas condicionada ao gatilho

- **Status:** aceite
- **Supersedes:** —
- **Superseded by:** —
- **Âncora:** `wiki/implementation_phase3/README.md`, `wiki/pendencias.md`

## Contexto

Ao fechar a Fase 2 (H0–H12, gates G10–G15 verdes) ficaram **onze** itens adiados por YAGNI, num
ficheiro (`implementation_phase2/ADIADOS.md`) com **duas** listas da mesma verdade (a do próprio
ficheiro e a do dossiê de pendências §2) e premissas que **já tinham expirado** (dizia que o
runtime não carregava guardrails — falso desde a correcção do M5 na H12). Havia duas opções: (a)
executar já os itens «porque a fase fechou»; (b) não planear nada até aparecer sintoma.

## Decisão

**Planear tudo o que vale a pena, executar só com gatilho.** A Fase 3
(`wiki/implementation_phase3/`, `epic_00j1cgo3`) tem, por item: **plano** (`Fn-*.md`), **gate**
(G16–G22, corridos pelo mesmo runner `xtask gates`) e **tarefa** kd — e cada item declara o
**gatilho** que o abre. A lista **verificada** é uma só (o [dossiê](../pendencias.md) §2); a Fase 3
é o **plano de execução**, não uma segunda lista. Dois invariantes novos: **M7** (uma mensagem,
uma entrega) e **M8** (nada invisível).

O que **não** vale a pena foi fechado com razão: `.ktu/logs/` (é um caminho, não trabalho;
`task_00ttgc9f` → `abandoned`), A9–A11 (já implementados), e D8–D12 (sem consumidor) ficam no
dossiê. O épico antigo (`epic_00ey9lmk`) foi absorvido.

## Alternatives considered

| Alternativa | Porque não |
|---|---|
| Executar já os onze itens «porque a fase fechou» | contradiz o YAGNI do `plan.md` §0: nenhum tem sintoma hoje (a fila de mutação, por exemplo, só faz sentido **depois** do paralelismo de tools) |
| Não planear até aparecer sintoma | o sintoma chega sob pressão e o desenho sai apressado; planear custa documentação e o gatilho impede a execução prematura |
| Manter as duas listas (ADIADOS + dossiê) | **já divergiram** uma vez (A1 vs D1); duas listas da mesma verdade = deriva garantida |
| Reabrir o épico antigo em vez de criar um novo | o antigo misturava «adiados» com «implementados» (A9–A11) e itens já sem valor (A4); um épico de **execução** tem de ter só trabalho vivo |
| Gates só por teste unitário | a Fase 2 provou o valor do artefacto arquivado; um gate sem artefacto é afirmação |

## Consequências

- O backlog activo **não** cresce: um item fora da Fase 3 está no dossiê com a condição de
  reabertura e volta como **issue nova**, com o sintoma que o justifica.
- **Custo assumido:** sete documentos de plano escritos antes de haver sintoma. Aceita-se porque
  cada um é curto, verificado no código (com a coluna «estado verificado» datada) e porque o
  gatilho impede a execução — o plano é a parte barata; o risco está na execução.
- O `F2` (REPL) mexe num caminho coberto pelo **G12**; a regressão obrigatória está declarada no
  [GATES](../implementation_phase3/GATES.md) §0.
