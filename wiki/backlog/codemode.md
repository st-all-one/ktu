# Codemode (compor tools com código em sandbox)

- **Estado:** candidata (do `pi`) — **avaliar** (alto valor, alto custo).
- **Origem:** `REF/pi/packages/coding-agent/docs/codemode.md`, `packages/codemode`;
  `wiki/revisao-pi.md` §2.6.
- **Âncora:** `crates/ktu-tools/`, `crates/ktu-core/src/tools.rs`.

## O que é
O modelo escreve **JavaScript** (QuickJS em sandbox, sem APIs de SO) que chama as outras
tools. **Só a saída** do script chega ao modelo — logo pode correr chamadas **em paralelo**
e **filtrar** resultados grandes antes de os mostrar. Globals: `tools.<name>(args)`,
`text`/`image`/`console`, `searchTools`/`describeTool`, `models` (classificadores/imagem),
`store`/`load`.

## Porque interessa
Ataca directamente a **performance terciária** (menos *round-trips*, menos tokens) e a
composição de tools. É uma **alternativa mais simples** ao "paralelismo de tools" (backlog
11): o paralelismo fica no script, não no kernel.

## O que o ktu tem hoje
Tools chamadas **uma a uma** pelo loop, sequencialmente na thread T2; sem composição.

## Pré-condições para reconsiderar
- **Número** que mostre o turno dominado por muitas tool calls independentes / resultados
  grandes.
- Um runtime JS seguro (QuickJS/WASM) + catálogo de tools tipado + limites
  (`max_output_tokens`, `timeout_ms`).

## Custo e risco
- **Esforço:** alto (runtime + sandbox + tipagem + limites + testes).
- **Risco de não fazer:** mais *round-trips* e tokens.
- **Risco de fazer:** superfície de execução nova; determinismo; dependência pesada.

## Decisão pendente
**Só com medição.** Se entrar, atrás de *flag* opt-in, com sandbox sem acesso a SO e
limites explícitos — e o script é **dado do modelo**, nunca privilégio.
