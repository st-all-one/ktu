# F4 — Fiabilidade de edição (fila por caminho, remédio, âncoras)

> **Issue:** `task_00sb14zh` · **Âncora:**
> [fila-mutacao-ficheiro](../backlog/fila-mutacao-ficheiro.md) · **Pré-requisitos:** H5, H6 ·
> **Gatilho:** **paralelismo de tools** (hoje rejeitado — R5, `unpriced`).

## Objetivo

Três correcções de robustez de edição, **atrás** do paralelismo de tool calls (rejeitado sem
número — não vale a pena fazer a fila antes de haver concorrência a serializar):

1. **fila de mutação por caminho** — serializa `write`/`edit` ao mesmo ficheiro (chave canónica:
   `realpath`, com *fallback* ao `resolve` se o ficheiro não existe);
2. **remédio por regra** — o `Deny` de um guardrail declarativo transporta o `remedy` da **regra**
   (hoje há remédio estrutural por motivo, mas não por regra);
3. **âncoras próximas** — o `edit` sugere as âncoras únicas mais próximas quando a âncora não é
   encontrada.

## Estado verificado (2026-10-18)

| Peça | Estado |
|---|---|
| tools sequenciais numa única T2 (`WorkerCtx::serve` serve um pedido de cada vez) | **verificado** — não há escrita concorrente hoje |
| `remedy` estrutural por `DenyReason` | feito (H6) |
| `remedy` **por regra** | **falta** (o `Deny` não transporta dados da regra) |
| a mensagem do `edit` devolve a âncora procurada | feito (H5) |
| **sugestão** das âncoras mais próximas | **falta** |

## Passos

1. **Fila:** `FileMutationQueue` (por caminho canónico) na **borda** (T2); `write`/`edit` passam
   por ela; ordem de chegada, uma por caminho.
2. **Remédio por regra:** `DenyReason::Policy` ganha o `remedy` da regra que negou; a mensagem
   final junta o remédio da regra ao estrutural (nunca o substitui).
3. **Âncoras:** quando a âncora não é encontrada, o `edit` devolve as **N** linhas mais próximas
   (distância de edição), em ordem de linha (determinístico).
4. **Testes adversariais:** dois `edit` concorrentes ao mesmo ficheiro ⇒ **sem perda**; regra com
   remédio próprio ⇒ mensagem com esse remédio; âncora errada ⇒ sugestão útil.

## Tarefas kd

| Tarefa | Entrega |
|---|---|
| `task_01qsu6vf` | Fila por caminho + remédio por regra + âncoras próximas |

## Gate / testes

- **G19** — ver [GATES](./GATES.md).
- N `write` concorrentes ao mesmo caminho ⇒ o ficheiro final é **um** dos escritos, sem intercalar.
- Regra `deny` com `remedy: "..."` ⇒ a mensagem contém esse remédio **e** o estrutural.
- Âncora inexistente ⇒ a sugestão inclui a linha correcta.

## Riscos

- Fila a virar *lock* global → a chave é **por caminho**; caminhos distintos correm em paralelo.
- Sugestão a adivinhar demais → só as N mais próximas, sempre com número de linha e texto.
- Fila no kernel (efeito) → vive na **borda** (M2); o kernel não conhece ficheiros.

## Estado da implementação (2026-10-18) — **fechado**

Gate **G19** verde (7/7): [relatório](../gates/G19-edicao.md) · artefactos em
`wiki/gates/artifacts/G19/`.

| Entrega | Onde |
|---|---|
| Fila de mutação por caminho canónico | `crates/ktu-tools/src/mutation.rs` (novo) |
| `write_file`/`edit_file` passam pela fila (a vez é pedida **antes** da leitura) | `crates/ktu-tools/src/fs_tools.rs` |
| Remédio da regra **junto** ao estrutural | `crates/ktu-core/src/agent/delegation.rs` |
| Âncoras próximas na mensagem do `edit` | `crates/ktu-tools/src/text.rs` |

**O que já existia (verificado, não reimplementado):** o remédio *por regra* já vinha do
`Guard::remedy(&fact)` (`DeclarativeGuard` devolve o `remedy` da regra; `ChainGuard` delega no
elo que negou) — o que faltava era a mensagem **não substituir** o estrutural pelo da regra. E a
mensagem do `edit` já devolvia a âncora procurada; faltava a sugestão.

**Fila:** chave = `realpath` com *fallback* ao caminho já resolvido (N2); uma mutação por
caminho, ordem de chegada; caminhos distintos em paralelo. Poda as entradas sem ninguém à espera
a partir de 256 caminhos. É de **borda** (T2) — o kernel não conhece ficheiros (M2).

**A fila é do processo, não do turno** (correcção seguinte, mesmo item): o `ktu serve` corre um
actor **por sessão** e o `--batch --parallel` uma thread **por worker**, e cada turno cria a sua
T2 — com a fila dentro do `FsTools`, cada turno tinha a sua, pelo que duas sessões do mesmo
processo editavam o mesmo ficheiro sem se verem (defeito **existente**, não hipotético). A fila
passou a ser um `Arc` no `Inner` do hub (e uma por lote), injectada em `execute_turn` → `Toolbox`
→ `FsTools`; a chave canónica partilhada é o que o teste do hub prova.

**Limite assumido:** dentro de *um* turno a T2 serve um pedido de cada vez, pelo que a
concorrência só aparece entre turnos/sessões. A fila é também a garantia que o **paralelismo de
tool calls** (R5, `unpriced`) vai precisar.

**Fora do âmbito (registado em `pendencias.md`):** **D13** — lock de ficheiro entre **processos**
(a fila é em memória; o `TurnLock` só trava o mesmo id de sessão); **D14** — escrita **atómica**
(temp + `Fs::rename`), que evita ficheiro truncado se o processo morrer a meio (o log de sessão é
reescrito inteiro a cada turno).

## Notas de execução (skills Rust)

> Ver [NOTAS-RUST](./NOTAS-RUST.md).
- **N2:** chave canónica sem `unwrap` (o `realpath` pode falhar → *fallback*).
- **N1:** as sugestões saem ordenadas por linha.
- **N8:** `DenyReason::Policy { rule, remedy }` é dado tipado.
