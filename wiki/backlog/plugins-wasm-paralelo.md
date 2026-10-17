# Plugins WASM/ABI + dispatch paralelo + paralelismo de tools

- **Estado:** rejeitado por medição (fechado).
- **Origem:** katu `12-plugins-e-abi.md`; `plan.md` §0; `decision_01e2183g`; `task_00ovthd1`.
- **Âncora:** `crates/ktu-plugin/`, `crates/ktu-core/src/plugin/`.

## O que é
- **Plugins em WASM** (sandbox de plugin) com ABI rica, em vez de processos externos
  JSON-lines.
- **Dispatch paralelo** de plugins e **paralelismo de tools** (executar N tools ao mesmo
  tempo em vez de sequencial).

## O que o ktu tem hoje
- Plugins **externos por processo** (JSON-lines stdio) com timeout e gate fail-closed.
- Hot reload por geração; dispatcher determinístico por nome.
- Tools executadas sequencialmente na thread T2.

## Porque não entrou
O benchmark do núcleo (`decision_01e2183g`) mede o loop em **microssegundos**; o gargalo
é o modelo. Não há número que exija paralelismo, e WASM/ABI é complexidade sem consumidor.
Fechado explicitamente como **não justificado**.

## Pré-condições para reconsiderar
- **Número** que mostre o loop dominado por tools/plugins (ex.: turno > limiar com muitas
  tools independentes).
- Um caso de uso de plugin que exija isolamento de SO (aí WASM ganha ao processo).

## Custo e risco
- **Esforço:** alto (WASM runtime/ABI) a médio (paralelismo de tools).
- **Risco de não fazer:** baixo hoje.
- **Risco de fazer (YAGNI):** reintroduzir o "fazer antes de precisar".

## Decisão pendente
**Reabrir só com medição.** A barra está fixada: um número, não uma intuição.
