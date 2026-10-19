# F6 — `ktu self setup <cliente>` — **fechado sem plano**

> **Estado:** ⛔ **fechado sem plano** (2026-10-18) — não vale a pena agora. A pendência
> correspondente (**D3**) fica no [dossiê](../pendencias.md) §2 com o critério de reabertura.
> **Issue:** `task_00odyng2` · **Tarefa:** `task_01oo6bry` (fechadas como `abandoned`).
> **Âncora:** [receitas](../backlog/receitas.md).

## Porque não vale a pena

O plano pedia `ktu self setup <cliente>` para instalar a integração do ktu em clientes externos
(`claude`/`cursor`/`codex`/`pi`) — receita por cliente, bloco gerido no ficheiro do cliente,
`--dry-run`, `--list`. Verificado o que existe hoje, **não há nada a instalar**:

1. **O ktu não se anuncia a ninguém.** O `AGENTS.md` é o ficheiro de instruções **do projecto**
   (escrito pelo humano) e o `ktu init` **não escreve nele** ([ADR 0021](../adr/0021-o-init-nao-escreve-no-agents-md.md)):
   o protocolo do agente vive em `ktu prime` e na base do prompt. Uma receita de cliente seria o
   ktu a anunciar-se num ficheiro de outro produto — sem consumidor.
2. **O único caminho que o `init` não cobre não existe no ktu.** O plano falava de «servidor MCP
   onde exista»: o ktu **não tem** servidor MCP (MCP está **fora por YAGNI**,
   [`backlog/mcp.md`](../backlog/mcp.md)); quem tem é o knudge (`knudge-mcp`). Sem MCP, a receita
   de um cliente só poderia apontar para o `AGENTS.md`/skill (feito), para um
   `ktu run --command <nome>` (receitas do H10) ou para a porta HTTP do `ktu serve` — nenhum
   deles precisa de um ficheiro novo no cliente.
3. **O precedente do knudge é mais leve — e não é o que o plano descrevia.** O
   `kd self setup <cliente>` valida o cliente contra `CLIENTS = ["claude","cursor","codex","pi"]`
   e grava uma **receita JSON na própria árvore do knudge** (`.knudge/setup/<cliente>.json`),
   imprimindo «aponte o cliente para `knudge-mcp` (stdio) e `AGENTS.md`» — **não toca** nos
   ficheiros do cliente. Ou seja: até o precedente se reduz a *emitir dados*, e a fiação fica com
   o humano. Copiar isso para o ktu seria escrever um ficheiro que ninguém lê.
4. **O gatilho é o consumidor, e o consumidor é hipotético.** O gatilho declarado («existirem
   receitas de cliente para instalar, ou um pedido concreto») não disparou: nenhum cliente foi
   apontado como não lendo `AGENTS.md`/skills. Construir a máquina antes do sintoma é exactamente
   o que a [ADR 0015](../adr/0015-yagni-com-criterio-de-reabertura.md) proíbe.

## O que fica registado (e o que reabre)

- **O que já existe e cobre o caso comum:** `ktu init` (bloco no `AGENTS.md` + skill + config +
  layout), receitas de comando (`ktu run --command <nome>`, H10), porta HTTP (`ktu serve`).
- **Critério de reabertura (em D3):** um cliente **concreto** que precise de configuração própria
  (ou um pedido explícito de *setup*) — e, se for MCP, a decisão de reabrir o MCP no ktu, que é uma
  decisão à parte ([`backlog/mcp.md`](../backlog/mcp.md)). O `AGENTS.md` deixou de contar: o ktu
  não escreve nele.
- **Se reabrir, o desenho a preferir** é o do knudge (receita como **dado versionado**, sem
  escrever em ficheiros de terceiros), não o bloco gerido em ficheiro alheio — a menos que o
  cliente só aceite configuração no próprio ficheiro.
- **O travão de forma** que valia a pena deste item (idempotência byte-a-byte, `--dry-run`,
  marcador único) já está provado onde existe escrita gerida: o `ktu init` tem os seus testes de
  idempotência e blocos geridos (H1) e o gate G14 cobre-o.

## Nota: as duas lacunas da vizinhança foram **resolvidas**

Ao fechar o F6 ficaram registadas duas lacunas reais na vizinhança — foram implementadas logo a
seguir (mesmo dia, gate **G14** a 15/15):

- **`git::agents::version_in` não era chamado por ninguém** (o marcador de versão do bloco de
  protocolo era escrito e nunca lido): chegou a ser incorporado no `ktu doctor` — e **caiu** com a
  [ADR 0021](../adr/0021-o-init-nao-escreve-no-agents-md.md), porque sem bloco gerado não há versão
  a ler. O que fica é o `ktu doctor` do projecto (layout e skill, com remédio).
- **O `ktu doctor` prometia o que não fazia** («layout e conhecimento» mas corria o diagnóstico do
  provider): passa a diagnosticar o **projecto** (layout, `AGENTS.md`, skill) e a **apontar** as
  outras portas (`ktu config doctor`, `ktu memory doctor`, `ktu providers doctor`); o diagnóstico
  da memória ganhou porta própria (`ktu memory doctor`, que sai de `doctor --memory`).
