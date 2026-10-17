# Custom tools (tools definidas pelo utilizador)

- **Estado:** candidata (do `opencode`) — **avaliar**.
- **Origem:** `REF/opencode/.opencode/tool/*.ts`; `wiki/revisao-opencode.md` §2.10.
- **Âncora:** `crates/ktu-core/src/tools.rs`, `crates/ktu-plugin/`.

## O que é
Tools definidas pelo utilizador em ficheiros (`.opencode/tool/`), disponíveis ao modelo.

## Porque interessa
O ktu tem **plugins** (que transformam/negam) mas **não** tools definidas pelo
utilizador. Uma tool de utilizador é usabilidade objectiva (adicionar capacidades sem
mexer no kernel).

## O que o ktu tem hoje
Tools internas fixas (`read/write/edit/list/glob/grep/bash/memory/plan/subagent`);
plugins externos (processo) que **não** adicionam tools.

## Pré-condições para reconsiderar
- Necessidade de uma tool específica do projecto.
- Modelo de execução (o ktu é Rust → a tool seria um **processo**, como os plugins, não TS).

## Custo e risco
- **Esforço:** médio (manifesto + protocolo de invocação + guardrail).
- **Risco de não fazer:** pedidos de tools ad-hoc no loop.
- **Risco de fazer:** **superfície de execução** nova — tem de passar pelo `Guard` e por
  confinamento/timeout.

## Decisão pendente
Adiar. Se entrar, como **processo** com manifesto (à imagem dos plugins), entrando no
funil do `Guard` (I10) e com caps — nunca como caminho privilegiado.
