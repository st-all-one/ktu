# MCP (Model Context Protocol)

- **Estado:** fora (YAGNI explícito).
- **Origem:** `plan.md` §0; katu `13-providers.md` (MCP como *seam*).
- **Âncora:** — (sem equivalente no ktu).

## O que é
Cliente MCP (stdio/HTTP) para expor **servidores de tools** externos ao agente, e/ou
expor o ktu como servidor MCP.

## O que o ktu tem hoje
- Tools internas (`read/write/edit/list/glob/grep/bash/memory/plan/subagent`).
- **Plugins externos** por processo (JSON-lines) que podem transformar/negar — o
  equivalente funcional mais próximo.
- Memória via porta `Memory` (knudge), sem MCP.

## Porque não entrou
Está na lista do "não-fazer" (§0). Os plugins cobrem a extensibilidade; MCP acrescentaria
um protocolo e um runtime sem consumidor.

## Pré-condições para reconsiderar
- Necessidade de integrar com um **ecossistema MCP** existente (tools de terceiros).
- Ou expor o ktu como ferramenta a outro agente.

## Custo e risco
- **Esforço:** médio-alto (protocolo + transporte + permissões).
- **Risco de não fazer:** baixo (plugins cobrem).
- **Risco de fazer (YAGNI):** superfície de segurança nova (tools externas).

## Decisão pendente
Manter fora. Se entrar, tratá-lo como **fonte de tools não confiável** (taint + guardrails),
nunca como caminho privilegiado.
