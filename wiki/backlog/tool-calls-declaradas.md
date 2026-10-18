# Tool calls declaradas como texto

- **Estado:** **implementado** (H5; `wiki/implementation_phase2/H5-fiabilidade-loop.md`).
- **Origem:** `REF/katu/wiki/crates/DOSSIE_PENDENCIAS.md` §4; `wiki/revisao-katu.md` §3.4.
- **Âncora:** `crates/ktu-core/src/agent/mod.rs`, `crates/ktu-provider/src/wire.rs`.

## O que é
Modelos locais **não** emitem `tool_calls` nativas — escrevem a chamada no `content` (bloco JSON ou
`<tool_call>`). O katu decodifica a declaração **contra o catálogo fechado** quando o passo não traz
chamadas nativas; o texto declarado **não** entra no log como mensagem do assistente.

## Porque interessa
Sem isto, um modelo local emite zero chamadas e o JSON cru vira resposta final (`calls = 0`) — o
agente "responde" em vez de agir. É a lacuna prática mais comum com modelos locais.

## O que o ktu tem hoje
`declared` = 1 hit (um teste). O loop só reconhece `tool_calls` nativas.

## Pré-condições para reconsiderar
- Um modelo em uso que declare tool calls como texto.

## Custo e risco
- **Esforço:** médio (decodificar contra o catálogo fechado; não logar o texto declarado).
- **Risco de não fazer:** modelos locais inúteis.
- **Risco de fazer:** aceitar chamadas que o modelo não "quis" — validar contra o catálogo,
  fail-closed; nunca executar fora do catálogo.

## Decisão pendente
Promover. Decodificação contra o catálogo fechado; o texto declarado **não** entra no log; teste com
modelo que declara por texto.
